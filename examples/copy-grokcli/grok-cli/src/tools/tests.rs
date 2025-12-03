#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_text_editor_create_and_view() {
        let mut editor = TextEditorTool::new();
        
        // Create a temporary file
        let temp_path = format!("./temp_test_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        
        let content = "Hello, World!";
        let result = editor.create(&temp_path, content).await.unwrap();
        assert!(result.success);
        
        // View the file
        let view_result = editor.view(&temp_path, None).await.unwrap();
        assert!(view_result.success);
        assert!(view_result.output.unwrap().contains("Hello, World!"));
        
        // Clean up
        std::fs::remove_file(&temp_path).ok();
    }

    #[tokio::test]
    async fn test_bash_tool_execution() {
        let mut bash = BashTool::new();
        
        // Test a simple echo command
        let result = bash.execute("echo hello", None).await.unwrap();
        assert!(result.success);
        assert!(result.output.unwrap().contains("hello"));
    }
}