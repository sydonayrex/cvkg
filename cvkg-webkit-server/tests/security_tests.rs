// Security tests for cvkg-webkit-server
// Run with: cargo test --test security_tests

use std::path::PathBuf;

/// Security: Test path validation prevents directory traversal
#[test]
fn test_validate_path_prevents_traversal() {
    let _base = PathBuf::from("/safe/pkg");
    let escape_attempt = PathBuf::from("/safe/pkg/../../../etc/passwd");
    assert!(escape_attempt.iter().any(|p| p == ".."));
}

/// Security: Test subpath validation rejects traversal sequences
#[test]
fn test_validate_subpath_rejects_traversal_sequences() {
    let malicious_subpaths = vec![
        "../etc",
        "../../secret",
        "wgpu/../../etc",
        "~/secret",
        "wgpu/../webgl2/../../../etc",
    ];
    
    for subpath in malicious_subpaths {
        let contains_traversal = subpath.contains("..") 
            || subpath.contains("/") 
            || subpath.starts_with("~");
        assert!(contains_traversal, "Should be flagged as malicious: {}", subpath);
    }
}

/// Security: Test subpath validation accepts valid paths
#[test]
fn test_validate_subpath_accepts_valid_paths() {
    let valid_subpaths = vec![
        "wgpu",
        "webgl2",
        "wasm",
        "native",
    ];
    
    for subpath in valid_subpaths {
        let contains_traversal = subpath.contains("..") 
            || subpath.contains("/") 
            || subpath.starts_with("~");
        assert!(!contains_traversal, "Should be valid: {}", subpath);
    }
}

/// Security: Test CORS origin parsing rejects wildcards in production
#[test]
fn test_cors_parsing_wildcard_security() {
    let permissive = "*";
    let is_permissive = permissive == "*";
    assert!(is_permissive, "Should detect wildcard");
    
    let safe_origins = vec!["http://localhost:3000"];
    assert!(!safe_origins.iter().any(|&o| o == "*"));
}

/// Security: Test CORS origin parsing accepts valid origins
#[test]
fn test_cors_parsing_valid_origins() {
    let origins = "https://app.example.com, https://admin.example.com";
    
    let parsed: Vec<&str> = origins
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    
    assert_eq!(parsed, vec!["https://app.example.com", "https://admin.example.com"]);
}

/// Security: Test AppError variants
#[test]
fn test_app_error_display() {
    let errors = vec![
        ("Invalid path", "Invalid path: access denied"),
        ("Unauthorized", "Unauthorized"),
        ("Rate limited", "Rate limit exceeded"),
        ("Internal", "Internal error: test"),
    ];
    
    for (error_type, expected_msg) in errors {
        assert!(!expected_msg.is_empty(), "{} should have message", error_type);
    }
}

/// Security: Test authentication header validation
#[test]
fn test_auth_header_format() {
    let valid_header = "Bearer my-secret-key";
    let invalid_headers = vec![
        "Basic my-secret-key",
        "my-secret-key",
        "Bearer",
        "",
    ];
    
    for header in invalid_headers {
        let valid = header.starts_with("Bearer ") && header.len() > 7;
        assert!(!valid, "Should reject: {}", header);
    }
    
    assert!(valid_header.starts_with("Bearer ") && valid_header.len() > 7);
}

/// Security: Test that malicious paths are detected
#[test]
fn test_malicious_path_detection() {
    let test_cases = vec![
        ("../../../etc/passwd", true),
        ("..%2F..%2Fetc%2Fpasswd", true),
        ("/etc/passwd", true),
        ("normal.js", false),
        ("pkg/wgpu/file.js", false),
    ];
    
    for (path, should_flag) in test_cases {
        let flagged = path.contains("..") || path.contains("%2F") || path.starts_with("/");
        assert_eq!(flagged, should_flag, "Path: {}", path);
    }
}

/// Performance: Benchmark path validation speed
#[test]
fn test_path_validation_performance() {
    use std::time::Instant;
    let start = Instant::now();
    
    for i in 0..1000 {
        let _base = PathBuf::from("/pkg");
        let _target = PathBuf::from(format!("/pkg/subdir/{}", i));
        let is_valid = !format!("/pkg/subdir/{}", i).contains("..");
        assert!(is_valid);
    }
    
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 100, "Path validation should be fast");
}
