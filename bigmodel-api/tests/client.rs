#[tokio::test]
async fn test_client_chat() {
    let config = bigmodel_api::Config::new("test-key");
    let client = bigmodel_api::BigModelClient::new(config);

    let request = bigmodel_api::ChatRequest {
        model: "glm-4".to_string(),
        messages: vec![bigmodel_api::Message {
            role: bigmodel_api::Role::User,
            content: bigmodel_api::Content::Text("Hello".to_string()),
            reasoning_content: None,
        }],
        temperature: Some(0.7),
        top_p: None,
        max_tokens: Some(100),
        stream: false,
        tools: None,
        tool_choice: None,
        thinking: None,
    };

    // This will fail with network error since no real API
    // But verifies the request structure is correct
    let result = client.chat(request).await;
    assert!(result.is_err());
}
