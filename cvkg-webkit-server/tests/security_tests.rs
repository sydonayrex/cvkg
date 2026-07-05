// Security integration tests for cvkg-webkit-server
// Run with: cargo test --test security_tests

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;

use cvkg_webkit_server::router::{
    AppState, Config, create_router,
};

fn setup_test_state() -> Arc<AppState> {
    let config = Config {
        addr: "127.0.0.1:3000".parse().unwrap(),
        pkg_dir: "static".to_string(),
        assets_dir: "static".to_string(),
        static_dir: "static".to_string(),
        rate_limit_rps: 100,
        timeout_secs: 10,
        max_concurrent: 10,
    };
    let (hmr_tx, _) = tokio::sync::broadcast::channel(16);
    Arc::new(AppState::new(config, hmr_tx))
}

/// Security: Test path validation middleware rejects traversal sequences (M2)
#[tokio::test]
async fn test_path_validation_middleware_traversal() {
    let state = setup_test_state();
    let app = create_router(state, None);

    let test_paths = vec![
        "/cvkg-webkit-server/static/../../../etc/passwd",
        "/cvkg-webkit-server/static/..%2f..%2fetc/passwd",
        "/cvkg-webkit-server/static/..%252f..%252fetc/passwd",
        "/cvkg-webkit-server/static/..\\..\\etc/passwd",
        "/cvkg-webkit-server/static//etc/passwd",
    ];

    for path in test_paths {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Path '{}' should be blocked",
            path
        );
    }
}

/// Security: Test path validation middleware accepts normal paths (M2)
#[tokio::test]
async fn test_path_validation_middleware_valid() {
    let state = setup_test_state();
    let app = create_router(state, None);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health/liveness")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// Security: Test authentication controls gate POST endpoints (H1, M1)
#[tokio::test]
async fn test_auth_gating_on_post() {
    let state = setup_test_state();
    let token = state.auth_token.clone();
    let app = create_router(state, None);

    // 1. Unauthenticated request should fail
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/snapshot")
                .body(Body::from("test"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // 2. Request with invalid token should fail
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/snapshot")
                .header("Authorization", "Bearer invalid-token")
                .body(Body::from("test"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // 3. Request with valid token should succeed
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/snapshot")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from("<div>test</div>"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

/// Security: Test HTML snapshot sanitization removes script tags & inline events (H1)
#[tokio::test]
async fn test_html_sanitization() {
    let state = setup_test_state();
    let token = state.auth_token.clone();
    let app = create_router(state.clone(), None);

    let malicious_payload = "<div>Safe content</div><script>alert('xss')</script><img src='a.jpg' onload='alert(1)'>";
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/snapshot")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from(malicious_payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify stored snapshot is sanitized
    let snapshot_guard = state.last_vdom_snapshot.load();
    let stored = snapshot_guard.as_ref().as_ref().unwrap();
    
    assert!(!stored.contains("<script>"), "Should strip script tags");
    assert!(!stored.contains("onload="), "Should strip inline event handlers");
    assert!(stored.contains("<div>Safe content</div>"), "Should keep safe elements");
}

/// Security: Test that HTML snapshot endpoint rejects payload > 256KB (L9)
#[tokio::test]
async fn test_html_snapshot_size_limit() {
    let state = setup_test_state();
    let token = state.auth_token.clone();
    let app = create_router(state, None);

    // Create a payload larger than 256KB
    let large_payload = "A".repeat(256 * 1024 + 1);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/snapshot")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from(large_payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
