use super::*;

#[test]
fn test_create_notification_request_serialization() {
    let req = CreateNotificationRequest {
        notification_id: "daily_briefing",
        title: "Daily Summary",
        message: "Today's briefing.",
    };
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["notification_id"], "daily_briefing");
    assert_eq!(json["title"], "Daily Summary");
    assert_eq!(json["message"], "Today's briefing.");
}

#[test]
fn test_base_url_trailing_slash_trimmed() {
    let client = HaClient::new("http://homeassistant.local:8123/", "token");
    assert_eq!(client.base_url, "http://homeassistant.local:8123");
}

#[test]
fn test_base_url_no_trailing_slash() {
    let client = HaClient::new("http://homeassistant.local:8123", "token");
    assert_eq!(client.base_url, "http://homeassistant.local:8123");
}
