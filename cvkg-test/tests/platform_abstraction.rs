use cvkg_platform_core::Platform;

#[test]
fn native_and_web_platforms_expose_distinct_capabilities() {
    let native = cvkg_render_native::NativePlatform::default();
    let web = cvkg_platform_web::WebPlatform::default();

    assert_eq!(Platform::name(&native), "native");
    assert_eq!(Platform::name(&web), "web");
    assert!(!native.is_headless());
    assert!(web.is_headless());
}
