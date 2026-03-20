use super::*;

#[test]
fn test_api_key_computation() {
    // md5("user@example.com:secret") should be deterministic
    let client = FeverClient::new("http://localhost/fever/", "user@example.com", "secret");
    let raw = "user@example.com:secret";
    let expected = format!("{:x}", md5::compute(raw.as_bytes()));
    assert_eq!(client.api_key, expected);
}

#[test]
fn test_strip_html_basic() {
    let html = "<p>Hello <b>world</b>!</p>";
    assert_eq!(super::strip_html(html), "Hello world!");
}

#[test]
fn test_strip_html_preserves_text() {
    let html = "<div><p>First paragraph.</p><p>Second paragraph.</p></div>";
    let result = super::strip_html(html);
    assert!(result.contains("First paragraph."));
    assert!(result.contains("Second paragraph."));
}

#[test]
fn test_strip_html_empty() {
    assert_eq!(super::strip_html(""), "");
}

#[test]
fn test_strip_html_no_tags() {
    let text = "Plain text without tags.";
    assert_eq!(super::strip_html(text), text);
}

#[test]
fn test_unread_ids_parsing() {
    // Simulate comma-separated string parsing
    let raw = "1,2,3,42,100";
    let ids: Vec<u64> = raw
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.trim().parse::<u64>().ok())
        .collect();
    assert_eq!(ids, vec![1, 2, 3, 42, 100]);
}

#[test]
fn test_unread_ids_empty_string() {
    let raw = "";
    let ids: Vec<u64> = raw
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.trim().parse::<u64>().ok())
        .collect();
    assert!(ids.is_empty());
}
