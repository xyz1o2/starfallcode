use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

/// 代码文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFileInfo {
    pub path: PathBuf,
    pub name: String,
    pub extension: String,
    pub size: u64,
    pub lines: usize,
    pub language: String,
}

/// 代码上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    pub file_info: CodeFileInfo,
    pub functions: Vec<FunctionInfo>,
    pub imports: Vec<String>,
    pub classes: Vec<String>,
    pub summary: String,
}

/// 函数信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub signature: String,
}

/// 文件操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperationResult {
    pub success: bool,
    pub message: String,
    pub data: Option<String>,
}

impl FileOperationResult {
    pub fn success(message: String, data: Option<String>) -> Self {
        Self {
            success: true,
            message,
            data,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message,
            data: None,
        }
    }
}

/// 代码文件处理器
pub struct CodeFileHandler {
    yolo_mode: bool,
}

impl CodeFileHandler {
    pub fn new() -> Self {
        Self { yolo_mode: false }
    }

    /// 启用 YOLO 模式
    pub fn enable_yolo_mode(&mut self) {
        self.yolo_mode = true;
    }

    /// 禁用 YOLO 模式
    pub fn disable_yolo_mode(&mut self) {
        self.yolo_mode = false;
    }

    /// 读取文件
    pub fn read_file(&self, path: &str) -> FileOperationResult {
        match fs::read_to_string(path) {
            Ok(content) => FileOperationResult::success(
                format!("File read successfully: {}", path),
                Some(content),
            ),
            Err(e) => FileOperationResult::error(format!("Failed to read file: {}", e)),
        }
    }

    /// 写入文件
    pub fn write_file(&self, path: &str, content: &str) -> FileOperationResult {
        match fs::write(path, content) {
            Ok(_) => FileOperationResult::success(
                format!("File written successfully: {}", path),
                None,
            ),
            Err(e) => FileOperationResult::error(format!("Failed to write file: {}", e)),
        }
    }

    /// 创建文件
    pub fn create_file(&self, path: &str, content: &str) -> FileOperationResult {
        let path_obj = Path::new(path);

        // 创建父目录
        if let Some(parent) = path_obj.parent() {
            if parent != Path::new("") {
                if let Err(e) = fs::create_dir_all(parent) {
                    return FileOperationResult::error(format!("Failed to create directory: {}", e));
                }
            }
        }

        // 检查文件是否已存在
        if path_obj.exists() {
            return FileOperationResult::error(format!("File already exists: {}", path));
        }

        self.write_file(path, content)
    }

    /// 删除文件（需要确认）
    pub fn delete_file(&self, path: &str, confirmed: bool) -> FileOperationResult {
        if !self.yolo_mode && !confirmed {
            return FileOperationResult::error(
                "Deletion requires confirmation. Use confirmed=true or enable YOLO mode".to_string(),
            );
        }

        match fs::remove_file(path) {
            Ok(_) => FileOperationResult::success(format!("File deleted: {}", path), None),
            Err(e) => FileOperationResult::error(format!("Failed to delete file: {}", e)),
        }
    }

    /// 获取文件信息
    pub fn get_file_info(&self, path: &str) -> FileOperationResult {
        let path_obj = Path::new(path);

        match fs::metadata(path) {
            Ok(metadata) => {
                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => String::new(),
                };

                let lines = content.lines().count();
                let extension = path_obj
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let language = self.detect_language(&extension);

                let info = CodeFileInfo {
                    path: path_obj.to_path_buf(),
                    name: path_obj
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    extension,
                    size: metadata.len(),
                    lines,
                    language,
                };

                FileOperationResult::success(
                    "File info retrieved".to_string(),
                    Some(serde_json::to_string_pretty(&info).unwrap_or_default()),
                )
            }
            Err(e) => FileOperationResult::error(format!("Failed to get file info: {}", e)),
        }
    }

    /// 列出目录内容
    pub fn list_directory(&self, path: &str) -> FileOperationResult {
        match fs::read_dir(path) {
            Ok(entries) => {
                let mut files = Vec::new();
                let mut dirs = Vec::new();

                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        let name = entry
                            .file_name()
                            .into_string()
                            .unwrap_or_default();

                        if metadata.is_dir() {
                            dirs.push(format!("[DIR] {}", name));
                        } else {
                            let size = metadata.len();
                            files.push(format!("{} ({}B)", name, size));
                        }
                    }
                }

                dirs.sort();
                files.sort();

                let mut result = dirs;
                result.extend(files);

                FileOperationResult::success(
                    format!("Directory listing: {}", path),
                    Some(result.join("\n")),
                )
            }
            Err(e) => FileOperationResult::error(format!("Failed to list directory: {}", e)),
        }
    }

    /// 搜索文件
    pub fn search_files(&self, directory: &str, pattern: &str) -> FileOperationResult {
        let mut results = Vec::new();

        if let Ok(entries) = fs::read_dir(directory) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let name = entry
                        .file_name()
                        .into_string()
                        .unwrap_or_default();

                    if name.contains(pattern) {
                        if metadata.is_dir() {
                            results.push(format!("[DIR] {}", name));
                        } else {
                            results.push(name);
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            FileOperationResult::error(format!("No files found matching: {}", pattern))
        } else {
            FileOperationResult::success(
                format!("Found {} matches", results.len()),
                Some(results.join("\n")),
            )
        }
    }

    /// 获取代码上下文
    pub fn get_code_context(&self, path: &str) -> FileOperationResult {
        match self.read_file(path) {
            result if !result.success => result,
            result => {
                let content = result.data.unwrap_or_default();
                let file_info = self.extract_file_info(path, &content);
                let functions = self.extract_functions(&content);
                let imports = self.extract_imports(&content);
                let classes = self.extract_classes(&content);
                let summary = self.generate_summary(&file_info, &functions, &classes);

                let context = CodeContext {
                    file_info,
                    functions,
                    imports,
                    classes,
                    summary,
                };

                FileOperationResult::success(
                    "Code context extracted".to_string(),
                    Some(serde_json::to_string_pretty(&context).unwrap_or_default()),
                )
            }
        }
    }

    /// 提取文件信息
    fn extract_file_info(&self, path: &str, content: &str) -> CodeFileInfo {
        let path_obj = Path::new(path);
        let extension = path_obj
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown")
            .to_string();

        CodeFileInfo {
            path: path_obj.to_path_buf(),
            name: path_obj
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            extension: extension.clone(),
            size: content.len() as u64,
            lines: content.lines().count(),
            language: self.detect_language(&extension),
        }
    }

    /// 提取函数
    fn extract_functions(&self, content: &str) -> Vec<FunctionInfo> {
        let mut functions = Vec::new();
        let mut line_num = 0;

        for line in content.lines() {
            line_num += 1;

            // 简单的函数检测（支持 Rust, Python, JavaScript）
            if line.trim().starts_with("fn ")
                || line.trim().starts_with("def ")
                || line.trim().starts_with("function ")
                || line.trim().starts_with("async fn ")
            {
                let signature = line.trim().to_string();
                let name = signature
                    .split('(')
                    .next()
                    .unwrap_or("")
                    .replace("fn ", "")
                    .replace("def ", "")
                    .replace("function ", "")
                    .replace("async fn ", "")
                    .trim()
                    .to_string();

                functions.push(FunctionInfo {
                    name,
                    line_start: line_num,
                    line_end: line_num,
                    signature,
                });
            }
        }

        functions
    }

    /// 提取导入
    fn extract_imports(&self, content: &str) -> Vec<String> {
        let mut imports = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ")
                || trimmed.starts_with("import ")
                || trimmed.starts_with("from ")
            {
                imports.push(trimmed.to_string());
            }
        }

        imports
    }

    /// 提取类
    fn extract_classes(&self, content: &str) -> Vec<String> {
        let mut classes = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("pub struct ")
                || trimmed.starts_with("struct ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("pub enum ")
                || trimmed.starts_with("enum ")
            {
                let name = trimmed
                    .split_whitespace()
                    .nth(2)
                    .unwrap_or("")
                    .split('{')
                    .next()
                    .unwrap_or("")
                    .to_string();

                if !name.is_empty() {
                    classes.push(name);
                }
            }
        }

        classes
    }

    /// 生成摘要
    fn generate_summary(
        &self,
        file_info: &CodeFileInfo,
        functions: &[FunctionInfo],
        classes: &[String],
    ) -> String {
        format!(
            "File: {} | Language: {} | Lines: {} | Functions: {} | Classes/Structs: {}",
            file_info.name,
            file_info.language,
            file_info.lines,
            functions.len(),
            classes.len()
        )
    }

    /// 检测编程语言
    fn detect_language(&self, extension: &str) -> String {
        match extension {
            "rs" => "Rust",
            "py" => "Python",
            "js" | "jsx" => "JavaScript",
            "ts" | "tsx" => "TypeScript",
            "go" => "Go",
            "java" => "Java",
            "cpp" | "cc" | "cxx" => "C++",
            "c" => "C",
            "rb" => "Ruby",
            "php" => "PHP",
            "swift" => "Swift",
            "kt" => "Kotlin",
            "cs" => "C#",
            "scala" => "Scala",
            "sh" | "bash" => "Bash",
            "sql" => "SQL",
            "html" => "HTML",
            "css" => "CSS",
            "json" => "JSON",
            "yaml" | "yml" => "YAML",
            "xml" => "XML",
            "md" => "Markdown",
            _ => "Unknown",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        let handler = CodeFileHandler::new();
        assert_eq!(handler.detect_language("rs"), "Rust");
        assert_eq!(handler.detect_language("py"), "Python");
        assert_eq!(handler.detect_language("js"), "JavaScript");
    }

    #[test]
    fn test_yolo_mode() {
        let mut handler = CodeFileHandler::new();
        assert!(!handler.yolo_mode);
        handler.enable_yolo_mode();
        assert!(handler.yolo_mode);
    }
}
