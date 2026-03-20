use super::*;
use axum::http::{HeaderMap, HeaderValue, header};
use base64::{engine::general_purpose::STANDARD, Engine};

fn auth_header(username: &str, password: &str) -> HeaderMap {
    let encoded = STANDARD.encode(format!("{}:{}", username, password));
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Basic {}", encoded)).unwrap(),
    );
    headers
}

#[test]
fn test_check_basic_auth_valid() {
    let headers = auth_header("alice", "secret");
    assert!(check_basic_auth(&headers, "alice", "secret"));
}

#[test]
fn test_check_basic_auth_wrong_password() {
    let headers = auth_header("alice", "wrong");
    assert!(!check_basic_auth(&headers, "alice", "secret"));
}

#[test]
fn test_check_basic_auth_missing_header() {
    assert!(!check_basic_auth(&HeaderMap::new(), "alice", "secret"));
}

#[test]
fn test_check_basic_auth_malformed_header() {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("NotBasic abc"),
    );
    assert!(!check_basic_auth(&headers, "alice", "secret"));
}
