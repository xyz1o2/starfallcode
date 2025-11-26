use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 工具类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolType {
    /// 文件操作工具
    FileOps,
    /// 代码分析工具
    CodeAnalysis,
    /// 搜索工具
    Search,
    /// Git 工具
    Git,
    /// 执行工具
    Execute,
}

impl ToString for ToolType {
    fn to_string(&self) -> String {
        match self {
            ToolType::FileOps => "file_ops".to_string(),
            ToolType::CodeAnalysis => "code_analysis".to_string(),
            ToolType::Search => "search".to_string(),
            ToolType::Git => "git".to_string(),
            ToolType::Execute => "execute".to_string(),
        }
    }
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub tool_type: String,
    pub description: String,
    pub enabled: bool,
    pub priority: u8,
}

/// 工具参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParams {
    pub params: HashMap<String, String>,
}

impl ToolParams {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.params.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

impl ToolResult {
    pub fn success(output: String) -> Self {
        Self {
            success: true,
            output,
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error),
        }
    }
}

/// 配对编程工具集
pub struct PairProgrammingTools {
    tools: HashMap<String, Tool>,
    yolo_mode: bool,
}

impl PairProgrammingTools {
    pub fn new() -> Self {
        let mut tools = HashMap::new();

        // 文件操作工具
        tools.insert(
            "file_read".to_string(),
            Tool {
                name: "file_read".to_string(),
                tool_type: ToolType::FileOps.to_string(),
                description: "Read file contents".to_string(),
                enabled: true,
                priority: 10,
            },
        );

        tools.insert(
            "file_write".to_string(),
            Tool {
                name: "file_write".to_string(),
                tool_type: ToolType::FileOps.to_string(),
                description: "Write or create file".to_string(),
                enabled: true,
                priority: 10,
            },
        );

        tools.insert(
            "file_delete".to_string(),
            Tool {
                name: "file_delete".to_string(),
                tool_type: ToolType::FileOps.to_string(),
                description: "Delete file (requires confirmation)".to_string(),
                enabled: true,
                priority: 8,
            },
        );

        tools.insert(
            "file_list".to_string(),
            Tool {
                name: "file_list".to_string(),
                tool_type: ToolType::FileOps.to_string(),
                description: "List files in directory".to_string(),
                enabled: true,
                priority: 9,
            },
        );

        // 代码分析工具
        tools.insert(
            "code_analyze".to_string(),
            Tool {
                name: "code_analyze".to_string(),
                tool_type: ToolType::CodeAnalysis.to_string(),
                description: "Analyze code structure and quality".to_string(),
                enabled: true,
                priority: 9,
            },
        );

        // 搜索工具
        tools.insert(
            "search_code".to_string(),
            Tool {
                name: "search_code".to_string(),
                tool_type: ToolType::Search.to_string(),
                description: "Search code in repository".to_string(),
                enabled: true,
                priority: 8,
            },
        );

        // Git 工具
        tools.insert(
            "git_status".to_string(),
            Tool {
                name: "git_status".to_string(),
                tool_type: ToolType::Git.to_string(),
                description: "Get git repository status".to_string(),
                enabled: true,
                priority: 7,
            },
        );

        Self {
            tools,
            yolo_mode: false,
        }
    }

    /// 启用 YOLO 模式（跳过确认）
    pub fn enable_yolo_mode(&mut self) {
        self.yolo_mode = true;
    }

    /// 禁用 YOLO 模式
    pub fn disable_yolo_mode(&mut self) {
        self.yolo_mode = false;
    }

    /// 检查是否启用 YOLO 模式
    pub fn is_yolo_mode(&self) -> bool {
        self.yolo_mode
    }

    /// 获取所有可用工具
    pub fn get_available_tools(&self) -> Vec<Tool> {
        self.tools
            .values()
            .filter(|t| t.enabled)
            .cloned()
            .collect()
    }

    /// 按优先级排序工具
    pub fn get_tools_by_priority(&self) -> Vec<Tool> {
        let mut tools = self.get_available_tools();
        tools.sort_by(|a, b| b.priority.cmp(&a.priority));
        tools
    }

    /// 获取特定类型的工具
    pub fn get_tools_by_type(&self, tool_type: &str) -> Vec<Tool> {
        self.tools
            .values()
            .filter(|t| t.enabled && t.tool_type == tool_type)
            .cloned()
            .collect()
    }

    /// 启用工具
    pub fn enable_tool(&mut self, name: &str) -> bool {
        if let Some(tool) = self.tools.get_mut(name) {
            tool.enabled = true;
            true
        } else {
            false
        }
    }

    /// 禁用工具
    pub fn disable_tool(&mut self, name: &str) -> bool {
        if let Some(tool) = self.tools.get_mut(name) {
            tool.enabled = false;
            true
        } else {
            false
        }
    }

    /// 执行工具
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        params: ToolParams,
    ) -> Result<ToolResult, String> {
        let tool = self
            .tools
            .get(tool_name)
            .ok_or_else(|| format!("Tool not found: {}", tool_name))?;

        if !tool.enabled {
            return Err(format!("Tool is disabled: {}", tool_name));
        }

        match tool_name {
            "file_read" => self.execute_file_read(params).await,
            "file_write" => self.execute_file_write(params).await,
            "file_delete" => self.execute_file_delete(params).await,
            "file_list" => self.execute_file_list(params).await,
            "code_analyze" => self.execute_code_analyze(params).await,
            "search_code" => self.execute_search_code(params).await,
            "git_status" => self.execute_git_status(params).await,
            _ => Err(format!("Unknown tool: {}", tool_name)),
        }
    }

    async fn execute_file_read(&self, params: ToolParams) -> Result<ToolResult, String> {
        let path = params
            .get("path")
            .ok_or("Missing 'path' parameter")?
            .clone();

        match std::fs::read_to_string(&path) {
            Ok(content) => Ok(ToolResult::success(content)),
            Err(e) => Ok(ToolResult::error(format!("Failed to read file: {}", e))),
        }
    }

    async fn execute_file_write(&self, params: ToolParams) -> Result<ToolResult, String> {
        let path = params
            .get("path")
            .ok_or("Missing 'path' parameter")?
            .clone();
        let content = params
            .get("content")
            .ok_or("Missing 'content' parameter")?
            .clone();

        match std::fs::write(&path, &content) {
            Ok(_) => Ok(ToolResult::success(format!("File written: {}", path))),
            Err(e) => Ok(ToolResult::error(format!("Failed to write file: {}", e))),
        }
    }

    async fn execute_file_delete(&self, params: ToolParams) -> Result<ToolResult, String> {
        let path = params
            .get("path")
            .ok_or("Missing 'path' parameter")?
            .clone();

        // 如果不是 YOLO 模式，需要确认
        if !self.yolo_mode {
            let confirmed = params
                .get("confirmed")
                .map(|s| s == "true")
                .unwrap_or(false);

            if !confirmed {
                return Ok(ToolResult::error(
                    "Deletion requires confirmation. Use confirmed=true or enable YOLO mode".to_string(),
                ));
            }
        }

        match std::fs::remove_file(&path) {
            Ok(_) => Ok(ToolResult::success(format!("File deleted: {}", path))),
            Err(e) => Ok(ToolResult::error(format!("Failed to delete file: {}", e))),
        }
    }

    async fn execute_file_list(&self, params: ToolParams) -> Result<ToolResult, String> {
        let path = params
            .get("path")
            .map(|s| s.as_str())
            .unwrap_or(".");

        match std::fs::read_dir(path) {
            Ok(entries) => {
                let files: Vec<String> = entries
                    .filter_map(|e| {
                        e.ok().and_then(|entry| {
                            entry.file_name().into_string().ok().map(|name| {
                                let metadata = entry.metadata().ok();
                                let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                                if is_dir {
                                    format!("[DIR] {}", name)
                                } else {
                                    name
                                }
                            })
                        })
                    })
                    .collect();

                Ok(ToolResult::success(files.join("\n")))
            }
            Err(e) => Ok(ToolResult::error(format!("Failed to list directory: {}", e))),
        }
    }

    async fn execute_code_analyze(&self, _params: ToolParams) -> Result<ToolResult, String> {
        Ok(ToolResult::success(
            "Code analysis: Ready to analyze code structure".to_string(),
        ))
    }

    async fn execute_search_code(&self, params: ToolParams) -> Result<ToolResult, String> {
        let query = params
            .get("query")
            .ok_or("Missing 'query' parameter")?
            .clone();

        Ok(ToolResult::success(format!(
            "Search results for: {}",
            query
        )))
    }

    async fn execute_git_status(&self, _params: ToolParams) -> Result<ToolResult, String> {
        match std::process::Command::new("git")
            .arg("status")
            .arg("--short")
            .output()
        {
            Ok(output) => {
                let status = String::from_utf8_lossy(&output.stdout).to_string();
                Ok(ToolResult::success(status))
            }
            Err(e) => Ok(ToolResult::error(format!("Git command failed: {}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tools_creation() {
        let tools = PairProgrammingTools::new();
        assert!(!tools.get_available_tools().is_empty());
    }

    #[test]
    fn test_yolo_mode() {
        let mut tools = PairProgrammingTools::new();
        assert!(!tools.is_yolo_mode());
        tools.enable_yolo_mode();
        assert!(tools.is_yolo_mode());
    }

    #[test]
    fn test_get_tools_by_type() {
        let tools = PairProgrammingTools::new();
        let file_tools = tools.get_tools_by_type("file_ops");
        assert!(!file_tools.is_empty());
    }
}
