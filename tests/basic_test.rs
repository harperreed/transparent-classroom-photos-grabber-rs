use transparent_classroom_photos_grabber_rs::error::AppError;

// Simple test that verifies our logging setup works
#[test]
fn test_logger_init() {
    // This should not panic
    transparent_classroom_photos_grabber_rs::init();
    transparent_classroom_photos_grabber_rs::init(); // Call twice to ensure Once works
}

// Simple test to verify the error type conversion works
#[test]
fn test_error_conversion() {
    let app_error = AppError::Generic("Test error".to_string());
    assert!(format!("{}", app_error).contains("Test error"));

    let app_error = AppError::Parse("Parse error".to_string());
    assert!(format!("{}", app_error).contains("Parse error"));
}
