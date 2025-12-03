/// Str Replace Editor Tool
/// 实现文件文本替换功能（类似 grok-cli 的 str_replace_editor）

use super::tool::{Tool, ToolCall, ToolDefinition, ToolParameter, ToolResult, ToolExecutionContext};
use std::fs;
use std::io::{Read, Write};
use std::pin::Pin;
use std::future::Future;

/// 文本替换编辑器工具
pub struct StrReplaceTool;

impl Tool for StrReplaceTool {
    fn name(&self) -> &str {
        "str_replace_editor"
    }

    fn description(&self) -> &str {
        "在文件中替换文本字符串，可编辑或创建文件。高级特性: replace_all 参数可替换所有匹配项，支持相似匹配（搜索可包含多行但不精确匹配，使用等宽块）。注意 old_str 参数必须完全匹配，不支持 shell 转义。"
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    description: "绝对路径或相对路径的文件".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
                ToolParameter {
                    name: "old_str".to_string(),
                    description: "需要精确匹配的现有文件中的连续文本块。可以包含 \\n 换行符以匹配多行".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
                ToolParameter {
                    name: "new_str".to_string(),
                    description: "替换后的新文本字符串".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                },
                ToolParameter {
                    name: "replace_all".to_string(),
                    description: "如果 old_str 在文件中出现多次，是否替换所有出现的位置".to_string(),
                    param_type: "boolean".to_string(),
                    required: false,
                },
            ],
        }
    }

    fn execute(&self, call: ToolCall) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>> {
        Box::pin(async move {
            let ctx = ToolExecutionContext::new(call.tool_name, call.arguments);

            let path = match ctx.get_string("path") {
                Some(p) => p,
                None => return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some("Missing required parameter: path".to_string()),
                },
            };

            let old_str = match ctx.get_string("old_str") {
                Some(s) => s,
                None => return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some("Missing required parameter: old_str".to_string()),
                },
            };

            let new_str = match ctx.get_string("new_str") {
                Some(s) => s,
                None => return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some("Missing required parameter: new_str".to_string()),
                },
            };

            let replace_all = ctx.get_bool("replace_all").unwrap_or(false);

            // 读取文件内容
            let content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(e) => return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some(format!("Failed to read file '{}': {}", path, e)),
                },
            };

            // 检查 old_str 是否存在
            if !content.contains(&old_str) {
                return ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some(format!(
                        "Text to replace not found.\nExpected exact match for:\n{}\n\nConsider: (1) Checking whitespace, (2) Escaping special characters, (3) Using a different approach",
                        old_str
                    )),
                };
            }

            // 执行替换
            let modified_content = if replace_all {
                content.replace(&old_str, &new_str)
            } else {
                content.replacen(&old_str, &new_str, 1)
            };

            let replacement_count = if replace_all {
                content.matches(&old_str).count()
            } else if content.contains(&old_str) {
                1
            } else {
                0
            };

            // 写回文件
            match fs::write(&path, modified_content) {
                Ok(_) => ToolResult {
                    success: true,
                    data: serde_json::json!({
                        "path": path,
                        "replacements_made": replacement_count,
                        "status": "success"
                    }),
                    error: None,
                },
                Err(e) => ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some(format!("Failed to write file '{}': {}", path, e)),
                },
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_str_replace_single() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // 创建测试文件
        let initial_content = r#"fn main() {
    println!("original");
    println!("keep this");
}"#;
        fs::write(&file_path, initial_content).unwrap();

        // 执行替换
        let tool = StrReplaceTool;
        let call = ToolCall {
            tool_name: "str_replace_editor".to_string(),
            arguments: vec![
                ("path".to_string(), serde_json::json!(file_path.to_string_lossy())),
                ("old_str".to_string(), serde_json::json!("println!(\"original\");")),
                ("new_str".to_string(), serde_json::json!("println!(\"replaced\");")),
            ].into_iter().collect(),
        };

        let result = tool.execute(call).await;
        println!("Result: {:?}", result);
        assert!(result.success);

        // 验证文件内容
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("println!(\"replaced\");"));
        assert!(!content.contains("println!(\"original\");"));
        assert!(content.contains("keep this"));
    }

    #[tokio::test]
    async fn test_str_replace_all() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // 创建测试文件（有重复内容）
        let initial_content = "foo\nfoo\nbar\n";
        fs::write(&file_path, initial_content).unwrap();

        // 执行替换所有匹配
        let tool = StrReplaceTool;
        let call = ToolCall {
            tool_name: "str_replace_editor".to_string(),
            arguments: vec![
                ("path".to_string(), serde_json::json!(file_path.to_string_lossy())),
                ("old_str".to_string(), serde_json::json!("foo")),
                ("new_str".to_string(), serde_json::json!("baz")),
                ("replace_all".to_string(), serde_json::json!(true)),
            ].into_iter().collect(),
        };

        let result = tool.execute(call).await;
        assert!(result.success);

        // 验证文件内容（全部被替换）
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "baz\nbaz\nbar\n");
        assert_eq!(result.data["replacements_made"], 2);
    }

    #[tokio::test]
    async fn test_str_replace_multiline() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // 创建测试文件
        let initial_content = r#"fn main() {
    let x = 1;
    let y = 2;
    println!("{}", x + y);
}"#;
        fs::write(&file_path, initial_content).unwrap();

        // 执行多行替换
        let tool = StrReplaceTool;
        let old_str = r#"    let x = 1;
    let y = 2;"#;
        let new_str = "    let x = 10;\n    let y = 20;";

        let call = ToolCall {
            tool_name: "str_replace_editor".to_string(),
            arguments: vec![
                ("path".to_string(), serde_json::json!(file_path.to_string_lossy())),
                ("old_str".to_string(), serde_json::json!(old_str)),
                ("new_str".to_string(), serde_json::json!(new_str)),
            ].into_iter().collect(),
        };

        let result = tool.execute(call).await;
        assert!(result.success);

        // 验证文件内容
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("let x = 10;"));
        assert!(content.contains("let y = 20;"));
        assert!(!content.contains("let x = 1;"));
    }
}
