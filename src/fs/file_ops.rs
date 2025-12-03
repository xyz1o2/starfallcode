//! 安全的文件操作模块
//! 提供带备份、确认和Git集成的文件操作

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use chrono::Local;

/// 文件操作结果
#[derive(Debug, Clone)]
pub struct FileOpResult {
    pub success: bool,
    pub message: String,
    pub backup_path: Option<PathBuf>,
}

impl FileOpResult {
    pub fn success(message: String, backup: Option<PathBuf>) -> Self {
        Self {
            success: true,
            message,
            backup_path: backup,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message,
            backup_path: None,
        }
    }
}

/// 文件操作管理器
pub struct SafeFileOps {
    enable_backups: bool,
    enable_git: bool,
}

impl SafeFileOps {
    pub fn new(enable_backups: bool, enable_git: bool) -> Self {
        Self {
            enable_backups,
            enable_git,
        }
    }

    /// 创建带备份的文件写入
    pub fn write_file(&self, path: &str, content: &str) -> io::Result<FileOpResult> {
        let path_buf = PathBuf::from(path);

        // 检查父目录是否存在
        if let Some(parent) = path_buf.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let backup = if path_buf.exists() && self.enable_backups {
            Some(self.create_backup(&path_buf)?)
        } else {
            None
        };

        fs::write(&path_buf, content)?;

        // Git add if enabled
        if self.enable_git && self.is_git_repo(path) {
            let _ = self.git_add(path);
        }

        Ok(FileOpResult::success(
            format!("File written: {}", path),
            backup,
        ))
    }

    /// 修改文件（带搜索替换）并创建备份
    pub fn modify_file(&self, path: &str, search: &str, replace: &str) -> io::Result<FileOpResult> {
        let path_buf = PathBuf::from(path);

        if !path_buf.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {}", path),
            ));
        }

        // 检查文件是否只读
        if self.is_readonly(&path_buf) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("File is read-only: {}", path),
            ));
        }

        let content = fs::read_to_string(&path_buf)?;

        // 创建备份
        let backup = if self.enable_backups {
            Some(self.create_backup(&path_buf)?)
        } else {
            None
        };

        // 执行替换
        if content.contains(search) {
            let new_content = content.replace(search, replace);
            fs::write(&path_buf, new_content)?;

            // Git add if enabled
            if self.enable_git && self.is_git_repo(path) {
                let _ = self.git_add(path);
            }

            Ok(FileOpResult::success(
                format!("File modified: {}", path),
                backup,
            ))
        } else {
            // 搜索内容不存在，恢复备份
            if let Some(ref backup_path) = backup {
                let _ = fs::copy(backup_path, &path_buf);
            }
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Search text not found in file: {}", search),
            ))
        }
    }

    /// 删除文件（先备份）
    pub fn delete_file(&self, path: &str) -> io::Result<FileOpResult> {
        let path_buf = PathBuf::from(path);

        if !path_buf.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {}", path),
            ));
        }

        // 备份文件（即使删除也要备份以防误删）
        let backup = if self.enable_backups {
            Some(self.create_backup(&path_buf)?)
        } else {
            None
        };

        fs::remove_file(&path_buf)?;

        Ok(FileOpResult::success(
            format!("File deleted: {}", path),
            backup,
        ))
    }

    /// 创建备份
    fn create_backup(&self, path: &PathBuf) -> io::Result<PathBuf> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!(
            "{}.bak.{}_{}",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file"),
            timestamp,
            rand::random::<u32>()
        );

        let backup_path = path.with_file_name(&backup_name);
        fs::copy(path, &backup_path)?;

        Ok(backup_path)
    }

    /// 检查文件是否为只读
    fn is_readonly(&self, path: &PathBuf) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(path) {
                let permissions = metadata.permissions();
                return permissions.mode() & 0o200 == 0;
            }
        }
        false
    }

    /// 检查是否在git仓库中
    fn is_git_repo(&self, path: &str) -> bool {
        let path_buf = PathBuf::from(path);
        let dir = if path_buf.is_file() {
            path_buf.parent().unwrap_or(&path_buf)
        } else {
            &path_buf
        };

        if let Ok(output) = std::process::Command::new("git")
            .arg("rev-parse")
            .arg("--git-dir")
            .current_dir(dir)
            .output()
        {
            output.status.success()
        } else {
            false
        }
    }

    /// 执行git add
    fn git_add(&self, path: &str) -> io::Result<()> {
        let path_buf = PathBuf::from(path);
        let dir = if path_buf.is_file() {
            path_buf.parent().unwrap_or(&path_buf)
        } else {
            &path_buf
        };

        let result = std::process::Command::new("git")
            .arg("add")
            .arg(path)
            .current_dir(dir)
            .output()?;

        if !result.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Git add failed",
            ));
        }

        Ok(())
    }

    /// 从备份恢复文件
    pub fn restore_backup(&self, backup_path: &PathBuf, original_path: &str) -> io::Result<()> {
        fs::copy(backup_path, original_path)?;
        Ok(())
    }

    /// 清理旧备份（保留最近N个）
    pub fn cleanup_backups(path: &str, keep_count: usize) -> io::Result<()> {
        let path_buf = PathBuf::from(path);
        let parent = path_buf.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "No parent directory")
        })?;

        let file_name = path_buf.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid path")
            })?;

        let backup_pattern = format!("{}.bak.*", file_name);
        let mut backups: Vec<_> = fs::read_dir(parent)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with(&format!("{}.bak.", file_name))
            })
            .collect();

        // 按时间排序（最新的在前）
        backups.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));
        backups.reverse();

        // 删除旧的备份
        for old_backup in backups.iter().skip(keep_count) {
            let _ = fs::remove_file(old_backup.path());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_write_file_with_backup() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_str = file_path.to_str().unwrap();

        let ops = SafeFileOps::new(true, false);
        let result = ops.write_file(file_str, "test content").unwrap();

        assert!(result.success);
        assert!(fs::read_to_string(file_str).unwrap() == "test content");
    }

    #[test]
    fn test_modify_file_with_backup() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_str = file_path.to_str().unwrap();

        fs::write(file_str, "hello world").unwrap();

        let ops = SafeFileOps::new(true, false);
        let result = ops.modify_file(file_str, "world", "rust").unwrap();

        assert!(result.success);
        assert!(fs::read_to_string(file_str).unwrap() == "hello rust");
        assert!(result.backup_path.is_some());
    }

    #[test]
    fn test_delete_file_with_backup() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let file_str = file_path.to_str().unwrap();

        fs::write(file_str, "test").unwrap();

        let ops = SafeFileOps::new(true, false);
        let result = ops.delete_file(file_str).unwrap();

        assert!(result.success);
        assert!(!file_path.exists());
        assert!(result.backup_path.is_some());
    }
}
