/// AI 代码修改检测和处理
/// 基于 Aider 的 Search/Replace 块格式和模糊匹配策略

use regex::Regex;
use std::fs;

/// 代码修改操作
#[derive(Debug, Clone)]
pub enum CodeModificationOp {
    /// 创建文件: 路径, 内容
    Create { path: String, content: String },
    /// 修改文件: 路径, 搜索块, 替换块
    Modify { path: String, search: String, replace: String },
    /// 删除文件: 路径
    Delete { path: String },
}

/// 代码修改结果
#[derive(Debug, Clone)]
pub struct CodeModificationResult {
    pub op: CodeModificationOp,
    pub success: bool,
    pub message: String,
    /// Diff 对比（用于显示给用户）
    pub diff: Option<CodeDiff>,
    /// 是否需要用户确认
    pub requires_confirmation: bool,
}

/// 代码 Diff 对比
#[derive(Debug, Clone)]
pub struct CodeDiff {
    pub file_path: String,
    pub old_content: String,
    pub new_content: String,
}

/// AI 响应中的代码块
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub language: String,
    pub content: String,
}

/// AI 代码修改检测器
pub struct AICodeModificationDetector;

impl AICodeModificationDetector {
    /// 从 AI 响应中检测代码修改操作
    /// 
    /// 支持的格式：
    /// 1. 创建文件: "create file `path`" 或 "create `path`"
    /// 2. 修改文件: "modify `path`" 或 "update `path`"
    /// 3. 删除文件: "delete `path`" 或 "remove `path`"
    pub fn detect_modifications(response: &str) -> Vec<CodeModificationOp> {
        let mut operations = Vec::new();
        
        // 提取所有代码块
        let code_blocks = Self::extract_code_blocks(response);
        
        // 检测创建文件指令
        if let Some(creates) = Self::detect_create_instructions(response) {
            for (path, block_idx) in creates {
                if let Some(block) = code_blocks.get(block_idx) {
                    operations.push(CodeModificationOp::Create {
                        path,
                        content: block.content.clone(),
                    });
                }
            }
        }
        
        // 检测修改文件指令
        if let Some(modifies) = Self::detect_modify_instructions(response) {
            for (path, block_idx) in modifies {
                if let Some(block) = code_blocks.get(block_idx) {
                    operations.push(CodeModificationOp::Modify {
                        path,
                        search: String::new(), // 需要从文件读取
                        replace: block.content.clone(),
                    });
                }
            }
        }
        
        // 检测删除文件指令
        if let Some(deletes) = Self::detect_delete_instructions(response) {
            for path in deletes {
                operations.push(CodeModificationOp::Delete { path });
            }
        }
        
        operations
    }

    /// 提取代码块
    fn extract_code_blocks(response: &str) -> Vec<CodeBlock> {
        let mut blocks = Vec::new();
        let re = Regex::new(r"```(\w*)\n([\s\S]*?)```").unwrap();
        
        for cap in re.captures_iter(response) {
            let language = cap.get(1)
                .map(|m| m.as_str())
                .unwrap_or("")
                .to_string();
            let content = cap.get(2)
                .map(|m| m.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            
            blocks.push(CodeBlock {
                language: if language.is_empty() { "text".to_string() } else { language },
                content,
            });
        }
        
        blocks
    }

    /// 检测创建文件指令
    fn detect_create_instructions(response: &str) -> Option<Vec<(String, usize)>> {
        let re = Regex::new(r"(?:create|new)\s+(?:file\s+)?`([^`]+)`").unwrap();
        let mut results = Vec::new();
        let mut block_idx = 0;
        
        for cap in re.captures_iter(response) {
            if let Some(path_match) = cap.get(1) {
                results.push((path_match.as_str().to_string(), block_idx));
                block_idx += 1;
            }
        }
        
        if results.is_empty() { None } else { Some(results) }
    }

    /// 检测修改文件指令
    fn detect_modify_instructions(response: &str) -> Option<Vec<(String, usize)>> {
        let re = Regex::new(r"(?:modify|update|change|edit|replace)\s+(?:file\s+)?`([^`]+)`").unwrap();
        let mut results = Vec::new();
        let mut block_idx = 0;
        
        for cap in re.captures_iter(response) {
            if let Some(path_match) = cap.get(1) {
                results.push((path_match.as_str().to_string(), block_idx));
                block_idx += 1;
            }
        }
        
        if results.is_empty() { None } else { Some(results) }
    }
    
    /// 检测隐含的代码修改意图（当有代码块但没有明确指令时）
    pub fn detect_implicit_modifications(response: &str) -> Vec<CodeModificationOp> {
        let code_blocks = Self::extract_code_blocks(response);
        let mut operations = Vec::new();
        
        // 如果有代码块，检查是否有隐含的修改意图
        if !code_blocks.is_empty() {
            // 检查是否提到了文件名或路径（如 index.html, main.rs 等）
            let file_pattern = Regex::new(r"(?:file|path|save|write|create|add).*?([a-zA-Z0-9_\-./]+\.[a-zA-Z0-9]+)").unwrap();
            
            for cap in file_pattern.captures_iter(response) {
                if let Some(path_match) = cap.get(1) {
                    let path = path_match.as_str().to_string();
                    if let Some(block) = code_blocks.first() {
                        // 如果提到了文件名和有代码块，可能是想创建文件
                        operations.push(CodeModificationOp::Create {
                            path,
                            content: block.content.clone(),
                        });
                    }
                }
            }
        }
        
        operations
    }

    /// 检测删除文件指令
    fn detect_delete_instructions(response: &str) -> Option<Vec<String>> {
        let re = Regex::new(r"(?:delete|remove)\s+(?:file\s+)?`([^`]+)`").unwrap();
        let mut results = Vec::new();
        
        for cap in re.captures_iter(response) {
            if let Some(path_match) = cap.get(1) {
                results.push(path_match.as_str().to_string());
            }
        }
        
        if results.is_empty() { None } else { Some(results) }
    }
}

/// 代码匹配和应用
pub struct CodeMatcher;

impl CodeMatcher {
    /// 在文件中搜索代码块
    /// 使用多层匹配策略：
    /// 1. 精确匹配
    /// 2. 空白不敏感匹配
    /// 3. 模糊匹配（Levenshtein 距离）
    pub fn find_and_replace(
        file_path: &str,
        search: &str,
        replace: &str,
    ) -> Result<CodeDiff, String> {
        // 读取文件
        let old_content = fs::read_to_string(file_path)
            .map_err(|e| format!("无法读取文件: {}", e))?;

        // 尝试精确匹配
        if old_content.contains(search) {
            let new_content = old_content.replace(search, replace);
            return Ok(CodeDiff {
                file_path: file_path.to_string(),
                old_content,
                new_content,
            });
        }

        // 尝试空白不敏感匹配
        let search_normalized = Self::normalize_whitespace(search);
        let content_normalized = Self::normalize_whitespace(&old_content);
        
        if content_normalized.contains(&search_normalized) {
            // 找到匹配的位置，使用原始内容替换
            if let Some(pos) = Self::find_fuzzy_match(&old_content, search, 0.8) {
                let (start, end) = pos;
                let mut new_content = old_content.clone();
                new_content.replace_range(start..end, replace);
                
                return Ok(CodeDiff {
                    file_path: file_path.to_string(),
                    old_content,
                    new_content,
                });
            }
        }

        Err(format!(
            "无法在文件中找到匹配的代码块:\n{}",
            search
        ))
    }

    /// 规范化空白（用于比较）
    fn normalize_whitespace(s: &str) -> String {
        s.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 模糊匹配（简化版 Levenshtein 距离）
    /// 返回 (start, end) 位置
    fn find_fuzzy_match(content: &str, search: &str, threshold: f64) -> Option<(usize, usize)> {
        let search_lines: Vec<&str> = search.lines().collect();
        let content_lines: Vec<&str> = content.lines().collect();
        
        if search_lines.is_empty() {
            return None;
        }

        // 简单的行级匹配
        for i in 0..content_lines.len() {
            let mut match_score = 0.0;
            let mut matched_lines = 0;

            for (j, search_line) in search_lines.iter().enumerate() {
                if i + j < content_lines.len() {
                    let content_line = content_lines[i + j];
                    let similarity = Self::string_similarity(search_line, content_line);
                    
                    if similarity > 0.7 {
                        match_score += similarity;
                        matched_lines += 1;
                    }
                }
            }

            if matched_lines > 0 {
                let avg_score = match_score / matched_lines as f64;
                if avg_score >= threshold {
                    // 计算字符位置
                    let start = content_lines[..i].join("\n").len() + if i > 0 { 1 } else { 0 };
                    let end = start + content_lines[i..i + matched_lines].join("\n").len();
                    return Some((start, end));
                }
            }
        }

        None
    }

    /// 计算字符串相似度（简化版）
    fn string_similarity(a: &str, b: &str) -> f64 {
        let a_trimmed = a.trim();
        let b_trimmed = b.trim();
        
        if a_trimmed == b_trimmed {
            return 1.0;
        }

        let a_normalized = Self::normalize_whitespace(a_trimmed);
        let b_normalized = Self::normalize_whitespace(b_trimmed);
        
        if a_normalized == b_normalized {
            return 0.95;
        }

        // 简单的字符匹配率
        let mut matches = 0;
        let max_len = a_trimmed.len().max(b_trimmed.len());
        
        for (ca, cb) in a_trimmed.chars().zip(b_trimmed.chars()) {
            if ca == cb {
                matches += 1;
            }
        }

        if max_len == 0 {
            1.0
        } else {
            matches as f64 / max_len as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_create_instruction() {
        let response = "Create file `src/main.rs`:\n\n```rust\nfn main() {}\n```";
        let ops = AICodeModificationDetector::detect_modifications(response);
        
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            CodeModificationOp::Create { path, .. } => {
                assert_eq!(path, "src/main.rs");
            }
            _ => panic!("Expected Create operation"),
        }
    }

    #[test]
    fn test_detect_modify_instruction() {
        let response = "Modify `src/app.rs`:\n\n```rust\npub fn new() {}\n```";
        let ops = AICodeModificationDetector::detect_modifications(response);
        
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            CodeModificationOp::Modify { path, .. } => {
                assert_eq!(path, "src/app.rs");
            }
            _ => panic!("Expected Modify operation"),
        }
    }

    #[test]
    fn test_string_similarity() {
        assert_eq!(CodeMatcher::string_similarity("hello", "hello"), 1.0);
        assert!(CodeMatcher::string_similarity("hello", "hallo") > 0.7);
        assert!(CodeMatcher::string_similarity("hello", "world") < 0.5);
    }
}
