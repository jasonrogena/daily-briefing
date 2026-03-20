use super::*;
use axum::http::HeaderValue;

#[test]
fn test_check_basic_auth_valid() {
    let mut headers = HeaderMap::new();
    // "admin:secret" base64-encoded = "YWRtaW46c2VjcmV0"
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("Basic YWRtaW46c2VjcmV0"),
    );
    assert!(check_basic_auth(&headers, "admin", "secret"));
}

#[test]
fn test_check_basic_auth_wrong_password() {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("Basic YWRtaW46c2VjcmV0"),
    );
    assert!(!check_basic_auth(&headers, "admin", "wrong"));
}

#[test]
fn test_check_basic_auth_missing_header() {
    let headers = HeaderMap::new();
    assert!(!check_basic_auth(&headers, "admin", "secret"));
}

#[test]
fn test_check_basic_auth_malformed_header() {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("Bearer sometoken"),
    );
    assert!(!check_basic_auth(&headers, "admin", "secret"));
}

#[test]
fn test_markdown_to_html_basic() {
    let html = markdown_to_html("**bold** and *italic*");
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<em>italic</em>"));
}

#[test]
fn test_markdown_to_html_heading() {
    let html = markdown_to_html("# Hello");
    assert!(html.contains("<h1>"));
    assert!(html.contains("Hello"));
}

#[test]
fn test_render_page_empty_content() {
    let content = PageContent::empty();
    let html = render_page("My Title", &content);
    assert!(html.contains("My Title"));
    assert!(html.contains("No content yet"));
    assert!(html.contains("never"));
}

#[test]
fn test_render_page_with_content() {
    let content = PageContent::new("# Summary\n\nHello world.");
    let html = render_page("Daily", &content);
    assert!(html.contains("Daily"));
    assert!(html.contains("<h1>"));
    assert!(html.contains("Hello world."));
    assert!(!html.contains("No content yet"));
}

#[test]
fn test_page_content_new_sets_timestamp() {
    let content = PageContent::new("test");
    assert!(content.updated_at.is_some());
}

#[test]
fn test_page_content_empty_has_no_timestamp() {
    let content = PageContent::empty();
    assert!(content.updated_at.is_none());
}
