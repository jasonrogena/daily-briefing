use super::*;

#[test]
fn test_request_body_serialization() {
    let body = RequestBody {
        model: "gpt-4o",
        max_tokens: 1024,
        messages: vec![
            Message {
                role: "system",
                content: "You are a helpful assistant.",
            },
            Message {
                role: "user",
                content: "Hello, world!",
            },
        ],
    };

    let json = serde_json::to_value(&body).unwrap();
    assert_eq!(json["model"], "gpt-4o");
    assert_eq!(json["max_tokens"], 1024);
    assert_eq!(json["messages"][0]["role"], "system");
    assert_eq!(json["messages"][1]["role"], "user");
    assert_eq!(json["messages"][1]["content"], "Hello, world!");
}

#[test]
fn test_response_parsing_success() {
    let json = serde_json::json!({
        "choices": [
            {
                "message": {
                    "role": "assistant",
                    "content": "This is the summary."
                },
                "finish_reason": "stop"
            }
        ]
    });

    let resp: ResponseBody = serde_json::from_value(json).unwrap();
    let text = resp.choices.into_iter().next().and_then(|c| c.message.content);
    assert_eq!(text, Some("This is the summary.".to_string()));
}

#[test]
fn test_response_parsing_api_error() {
    let json = serde_json::json!({
        "choices": [],
        "error": {
            "message": "Invalid API key",
            "type": "invalid_request_error",
            "code": "invalid_api_key"
        }
    });

    let resp: ResponseBody = serde_json::from_value(json).unwrap();
    assert!(resp.error.is_some());
    assert_eq!(resp.error.unwrap().message, "Invalid API key");
}

#[test]
fn test_response_parsing_empty_choices() {
    let json = serde_json::json!({ "choices": [] });
    let resp: ResponseBody = serde_json::from_value(json).unwrap();
    let text = resp.choices.into_iter().next().and_then(|c| c.message.content);
    assert!(text.is_none());
}

#[test]
fn test_client_default_base_url() {
    let client = OpenAiClient::new(None, None);
    assert_eq!(client.base_url, DEFAULT_BASE_URL);
}

#[test]
fn test_client_custom_base_url_strips_trailing_slash() {
    let client = OpenAiClient::new(Some("http://192.168.10.3:8080/v1/"), None);
    assert_eq!(client.base_url, "http://192.168.10.3:8080/v1");
}
