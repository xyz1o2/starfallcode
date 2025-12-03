#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_grok_client_creation() {
        let client = GrokClient::new("test-key", Some("test-model".to_string()), Some("https://api.test.com".to_string()));
        assert_eq!(client.get_current_model(), "test-model");
    }

    #[tokio::test]
    async fn test_default_model() {
        let client = GrokClient::new("test-key", None, Some("https://api.test.com".to_string()));
        assert_eq!(client.get_current_model(), "grok-code-fast-1");
    }
}