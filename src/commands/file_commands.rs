/// 文件操作命令处理
use crate::utils::code_file_handler::CodeFileHandler;

#[derive(Debug, Clone)]
pub enum FileCommand {
    /// 创建文件: /create-file <path> [content]
    CreateFile { path: String, content: Option<String> },
    /// 修改文件: /modify-file <path> <content>
    ModifyFile { path: String, content: String },
    /// 搜索替换修改: /modify-file-search <path> <search> <replace>
    ModifyFileSearch { path: String, search: String, replace: String },
    /// 确认修改: /confirm-modify
    ConfirmModify,
    /// 取消修改: /cancel-modify
    CancelModify,
    /// 删除文件: /delete-file <path>
    DeleteFile { path: String },
    /// 读取文件: /read-file <path>
    ReadFile { path: String },
    /// 列出目录: /list-dir <path>
    ListDir { path: String },
    /// 搜索文件: /search-files <dir> <pattern>
    SearchFiles { directory: String, pattern: String },
}

#[derive(Debug, Clone)]
pub struct FileCommandResult {
    pub success: bool,
    pub message: String,
    pub content: Option<String>,
    pub requires_confirmation: bool,
    pub diff: Option<FileDiff>,
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub file_path: String,
    pub old_content: String,
    pub new_content: String,
}

pub struct FileCommandHandler {
    file_handler: CodeFileHandler,
    yolo_mode: bool,
    pending_modification: Option<(String, String)>, // (path, new_content)
    confirmation_pending: bool,
    confirmation_selected: ConfirmationChoice, // 当前选择
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfirmationChoice {
    Confirm,
    Cancel,
}

impl FileCommandHandler {
    pub fn new() -> Self {
        Self {
            file_handler: CodeFileHandler::new(),
            yolo_mode: false,
            pending_modification: None,
            confirmation_pending: false,
            confirmation_selected: ConfirmationChoice::Confirm,
        }
    }

    /// 上下箭头导航确认选择
    pub fn move_confirmation_up(&mut self) {
        if self.confirmation_pending {
            self.confirmation_selected = match self.confirmation_selected {
                ConfirmationChoice::Confirm => ConfirmationChoice::Cancel,
                ConfirmationChoice::Cancel => ConfirmationChoice::Confirm,
            };
        }
    }

    /// 下箭头导航确认选择
    pub fn move_confirmation_down(&mut self) {
        if self.confirmation_pending {
            self.confirmation_selected = match self.confirmation_selected {
                ConfirmationChoice::Confirm => ConfirmationChoice::Cancel,
                ConfirmationChoice::Cancel => ConfirmationChoice::Confirm,
            };
        }
    }

    /// 获取当前确认选择
    pub fn get_confirmation_choice(&self) -> ConfirmationChoice {
        self.confirmation_selected
    }

    /// 是否有待确认的修改
    pub fn has_pending_confirmation(&self) -> bool {
        self.confirmation_pending
    }

    pub fn enable_yolo_mode(&mut self) {
        self.yolo_mode = true;
        self.file_handler.enable_yolo_mode();
    }

    pub fn disable_yolo_mode(&mut self) {
        self.yolo_mode = false;
        self.file_handler.disable_yolo_mode();
    }

    /// 获取文件处理器（用于直接文件操作）
    pub fn file_handler(&self) -> &CodeFileHandler {
        &self.file_handler
    }

    /// 解析命令字符串
    pub fn parse_command(input: &str) -> Option<FileCommand> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "/create-file" => {
                if parts.len() < 2 {
                    return None;
                }
                let path = parts[1].to_string();
                let content = if parts.len() > 2 {
                    Some(parts[2..].join(" "))
                } else {
                    None
                };
                Some(FileCommand::CreateFile { path, content })
            }
            "/modify-file" => {
                if parts.len() < 3 {
                    return None;
                }
                let path = parts[1].to_string();
                let content = parts[2..].join(" ");
                Some(FileCommand::ModifyFile { path, content })
            }
            "/modify-file-search" => {
                if parts.len() < 4 {
                    return None;
                }
                let path = parts[1].to_string();
                let search = parts[2].to_string();
                let replace = parts[3..].join(" ");
                Some(FileCommand::ModifyFileSearch { path, search, replace })
            }
            "/delete-file" => {
                if parts.len() < 2 {
                    return None;
                }
                let path = parts[1].to_string();
                Some(FileCommand::DeleteFile { path })
            }
            "/read-file" => {
                if parts.len() < 2 {
                    return None;
                }
                let path = parts[1].to_string();
                Some(FileCommand::ReadFile { path })
            }
            "/list-dir" => {
                if parts.len() < 2 {
                    return None;
                }
                let path = parts[1].to_string();
                Some(FileCommand::ListDir { path })
            }
            "/search-files" => {
                if parts.len() < 3 {
                    return None;
                }
                let directory = parts[1].to_string();
                let pattern = parts[2].to_string();
                Some(FileCommand::SearchFiles { directory, pattern })
            }
            _ => None,
        }
    }

    /// 执行文件命令
    pub fn execute(&mut self, command: FileCommand) -> FileCommandResult {
        match command {
            FileCommand::CreateFile { path, content } => {
                let content = content.unwrap_or_default();
                let result = self.file_handler.create_file(&path, &content);
                if result.success {
                    FileCommandResult {
                        success: true,
                        message: format!("✅ 文件已创建: {}", path),
                        content: None,
                        requires_confirmation: false,
                        diff: None,
                    }
                } else {
                    FileCommandResult {
                        success: false,
                        message: format!("❌ 创建文件失败: {}", result.message),
                        content: None,
                        requires_confirmation: false,
                        diff: None,
                    }
                }
            }
            FileCommand::ModifyFile { path, content } => {
                // 读取原始内容用于 diff
                let read_result = self.file_handler.read_file(&path);
                if read_result.success {
                    let old_content = read_result.data.unwrap_or_default();

                    // 如果启用 YOLO 模式，直接修改
                    if self.yolo_mode {
                        let write_result = self.file_handler.write_file(&path, &content);
                        if write_result.success {
                            FileCommandResult {
                                success: true,
                                message: format!("✅ 文件已修改: {}", path),
                                content: None,
                                requires_confirmation: false,
                                diff: None,
                            }
                        } else {
                            FileCommandResult {
                                success: false,
                                message: format!("❌ 修改文件失败: {}", write_result.message),
                                content: None,
                                requires_confirmation: false,
                                diff: None,
                            }
                        }
                    } else {
                        // 否则显示 diff，等待确认
                        self.pending_modification = Some((path.clone(), content.clone()));
                        self.confirmation_pending = true;
                        self.confirmation_selected = ConfirmationChoice::Confirm; // 默认选择确认
                        FileCommandResult {
                            success: true,
                            message: format!("📝 显示修改对比 (使用 ↑↓ 选择，Enter 确认)"),
                            content: None,
                            requires_confirmation: true,
                            diff: Some(FileDiff {
                                file_path: path,
                                old_content,
                                new_content: content,
                            }),
                        }
                    }
                } else {
                    FileCommandResult {
                        success: false,
                        message: format!("❌ 读取文件失败: {}", read_result.message),
                        content: None,
                        requires_confirmation: false,
                        diff: None,
                    }
                }
            }
            FileCommand::ModifyFileSearch { path, search, replace } => {
                // 使用搜索替换方式修改文件
                let modify_result = self.file_handler.modify_file(&path, &search, &replace);
                if modify_result.success {
                    let backup_info = if let Some(backup) = &modify_result.backup_path {
                        format!(" (备份: {})", backup.display())
                    } else {
                        String::new()
                    };
                    FileCommandResult {
                        success: true,
                        message: format!("✅ 文件已修改{}: {}", backup_info, path),
                        content: None,
                        requires_confirmation: false,
                        diff: None,
                    }
                } else {
                    FileCommandResult {
                        success: false,
                        message: format!("❌ 修改文件失败: {}", modify_result.message),
                        content: None,
                        requires_confirmation: false,
                        diff: None,
                    }
                }
            }
            FileCommand::ConfirmModify => {
                // 根据当前选择执行确认或取消
                match self.confirmation_selected {
                    ConfirmationChoice::Confirm => {
                        if let Some((path, content)) = self.pending_modification.take() {
                            self.confirmation_pending = false;
                            let result = self.file_handler.write_file(&path, &content);
                            if result.success {
                                FileCommandResult {
                                    success: true,
                                    message: format!("✅ 修改已确认并保存: {}", path),
                                    content: None,
                                    requires_confirmation: false,
                                    diff: None,
                                }
                            } else {
                                FileCommandResult {
                                    success: false,
                                    message: format!("❌ 保存文件失败: {}", result.message),
                                    content: None,
                                    requires_confirmation: false,
                                    diff: None,
                                }
                            }
                        } else {
                            FileCommandResult {
                                success: false,
                                message: "❌ 没有待确认的修改".to_string(),
                                content: None,
                                requires_confirmation: false,
                                diff: None,
                            }
                        }
                    }
                    ConfirmationChoice::Cancel => {
                        self.pending_modification = None;
                        self.confirmation_pending = false;
                        FileCommandResult {
                            success: true,
                            message: "✅ 修改已取消".to_string(),
                            content: None,
                            requires_confirmation: false,
                            diff: None,
                        }
                    }
                }
            }
            FileCommand::CancelModify => {
                self.pending_modification = None;
                self.confirmation_pending = false;
                FileCommandResult {
                    success: true,
                    message: "✅ 修改已取消".to_string(),
                    content: None,
                    requires_confirmation: false,
                    diff: None,
                }
            }
            FileCommand::DeleteFile { path } => {
                let result = self.file_handler.delete_file(&path, self.yolo_mode);
                if result.success {
                    FileCommandResult {
                        success: true,
                        message: format!("✅ 文件已删除: {}", path),
                        content: None,
                        requires_confirmation: false,
                        diff: None,
                    }
                } else {
                    FileCommandResult {
                        success: false,
                        message: format!("❌ 删除文件失败: {}", result.message),
                        content: None,
                        requires_confirmation: false,
                        diff: None,
                    }
                }
            }
            FileCommand::ReadFile { path } => {
                let result = self.file_handler.read_file(&path);
                if result.success {
                    FileCommandResult {
                        success: true,
                        message: format!("✅ 文件已读取: {}", path),
                        content: result.data,
                        requires_confirmation: false,
                        diff: None,
                    }
                } else {
                    FileCommandResult {
                        success: false,
                        message: format!("❌ 读取文件失败: {}", result.message),
                        content: None,
                        requires_confirmation: false,
                        diff: None,
                    }
                }
            }
            FileCommand::ListDir { path } => {
                let result = self.file_handler.list_directory(&path);
                if result.success {
                    FileCommandResult {
                        success: true,
                        message: format!("✅ 目录列表: {}", path),
                        content: result.data,
                        requires_confirmation: false,
                        diff: None,
                    }
                } else {
                    FileCommandResult {
                        success: false,
                        message: format!("❌ 列表失败: {}", result.message),
                        content: None,
                        requires_confirmation: false,
                        diff: None,
                    }
                }
            }
            FileCommand::SearchFiles { directory, pattern } => {
                let result = self.file_handler.search_files(&directory, &pattern);
                if result.success {
                    FileCommandResult {
                        success: true,
                        message: format!("✅ 搜索结果: {} 中匹配 {}", directory, pattern),
                        content: result.data,
                        requires_confirmation: false,
                        diff: None,
                    }
                } else {
                    FileCommandResult {
                        success: false,
                        message: format!("❌ 搜索失败: {}", result.message),
                        content: None,
                        requires_confirmation: false,
                        diff: None,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_file() {
        let cmd = FileCommandHandler::parse_command("/create-file test.txt hello");
        assert!(cmd.is_some());
    }

    #[test]
    fn test_parse_read_file() {
        let cmd = FileCommandHandler::parse_command("/read-file test.txt");
        assert!(cmd.is_some());
    }

    #[test]
    fn test_parse_delete_file() {
        let cmd = FileCommandHandler::parse_command("/delete-file test.txt");
        assert!(cmd.is_some());
    }
}
