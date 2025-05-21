// Basic smoke test to ensure the library loads and initializes

#[test]
fn test_init() {
    transparent_classroom_photos_grabber_rs::init();
    // If we get here without panicking, the test passes
}

#[test]
fn test_version() {
    let version = transparent_classroom_photos_grabber_rs::version();
    assert!(!version.is_empty(), "Version should not be empty");
}
