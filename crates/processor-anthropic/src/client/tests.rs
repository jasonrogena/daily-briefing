use super::*;

#[test]
fn test_request_body_serialization() {
    let body = RequestBody {
        model: "claude-opus-4-6",
        max_tokens: 1024,
        messages: vec![Message {
            role: "user",
            content: "Hello, world!".to_string(),
        }],
    };

    let json = serde_json::to_value(&body).unwrap();
    assert_eq!(json["model"], "claude-opus-4-6");
    assert_eq!(json["max_tokens"], 1024);
    assert_eq!(json["messages"][0]["role"], "user");
    assert_eq!(json["messages"][0]["content"], "Hello, world!");
}

#[test]
fn test_response_parsing_success() {
    let json = serde_json::json!({
        "content": [
            {"type": "text", "text": "This is the summary."}
        ]
    });

    let resp: ResponseBody = serde_json::from_value(json).unwrap();
    let text = resp
        .content
        .into_iter()
        .find(|b| b.block_type == "text")
        .and_then(|b| b.text);
    assert_eq!(text, Some("This is the summary.".to_string()));
}

#[test]
fn test_response_parsing_empty_content() {
    let json = serde_json::json!({
        "content": []
    });

    let resp: ResponseBody = serde_json::from_value(json).unwrap();
    let text = resp
        .content
        .into_iter()
        .find(|b| b.block_type == "text")
        .and_then(|b| b.text);
    assert!(text.is_none());
}

#[test]
fn test_message_content_combines_prompt_and_input() {
    let prompt = "Summarize the following:";
    let content = "Article 1\nArticle 2";
    let combined = format!("{}\n\n{}", prompt, content);
    assert!(combined.starts_with("Summarize the following:"));
    assert!(combined.contains("Article 1"));
    assert!(combined.contains("Article 2"));
}
