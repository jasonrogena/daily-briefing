use super::*;
use chrono::TimeZone;

#[test]
fn test_markdown_to_html_basic() {
    let html = markdown_to_html("**bold** and *italic*");
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<em>italic</em>"));
}

#[test]
fn test_markdown_to_html_heading() {
    let html = markdown_to_html("# Title");
    assert!(html.contains("<h1>"));
    assert!(html.contains("Title"));
}

#[test]
fn test_render_page_with_content() {
    let ts = Utc.with_ymd_and_hms(2026, 3, 20, 8, 0, 0).unwrap();
    let html = render_page("My Briefing", "## Hello\n\nWorld", ts);
    assert!(html.contains("<title>My Briefing</title>"));
    assert!(html.contains("<h2>"));
    assert!(html.contains("Hello"));
    assert!(html.contains("2026-03-20 08:00 UTC"));
}

#[test]
fn test_render_page_empty_content() {
    let ts = Utc.with_ymd_and_hms(2026, 3, 20, 8, 0, 0).unwrap();
    let html = render_page("My Briefing", "", ts);
    assert!(html.contains("No content yet"));
}
