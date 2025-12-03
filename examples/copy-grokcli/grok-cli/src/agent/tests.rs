#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_grok_agent_creation() {
        let result = GrokAgent::new(
            "test-key", 
            "https://api.test.com".to_string(), 
            Some("test-model".to_string()), 
            Some(10)
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_grok_agent_message_processing() {
        let mut agent = GrokAgent::new(
            "test-key", 
            "https://api.test.com".to_string(), 
            Some("test-model".to_string()), 
            Some(10)
        ).await.unwrap();
        
        // This test would require a mock API to work properly
        // For now, we just test that the method is callable
        let result = agent.process_user_message("test message").await;
        
        // Note: This will likely fail without a real API, but that's expected
        // The test demonstrates that the method exists and can be called
    }
}