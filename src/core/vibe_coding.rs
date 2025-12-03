/// Vibe Coding 核心工作流实现
/// 5阶段工作流：概念化 → 生成 → 迭代 → 验证 → 部署
///
/// 参考：
/// - https://medium.com/@ryan.kent/vibe-coding-how-ai-transformed-my-development-workflow-in-2025-e4982db19741
/// - https://vibecoding.app/blog/vibecoding-complete-guide
/// - https://www.freecodecamp.org/news/how-to-use-vibe-coding-effectively-as-a-dev/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Local};
use crate::utils::code_file_handler::CodeFileHandler;

/// 生成唯一 ID
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("vibe_{}", timestamp)
}

/// Vibe Coding 项目元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeProject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tech_stack: Vec<String>,
    pub created_at: DateTime<Local>,
    pub current_stage: VibeStage,
    pub metadata: HashMap<String, String>,
}

impl VibeProject {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: generate_id(),
            name,
            description,
            tech_stack: Vec::new(),
            created_at: Local::now(),
            current_stage: VibeStage::Conceptualization,
            metadata: HashMap::new(),
        }
    }
}

/// Vibe Coding 5 个阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VibeStage {
    /// Stage 1: 概念化 & 提示工程
    Conceptualization,
    /// Stage 2: AI 生成 & 初始构建
    Generation,
    /// Stage 3: 迭代优化循环
    Iteration,
    /// Stage 4: 验证 & 质量保证
    Validation,
    /// Stage 5: 部署 & 监控
    Deployment,
}

impl VibeStage {
    pub fn name(&self) -> &'static str {
        match self {
            VibeStage::Conceptualization => "概念化",
            VibeStage::Generation => "生成",
            VibeStage::Iteration => "迭代",
            VibeStage::Validation => "验证",
            VibeStage::Deployment => "部署",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            VibeStage::Conceptualization => "定义需求，创建详细的产品需求文档",
            VibeStage::Generation => "AI生成全栈代码和初始构建",
            VibeStage::Iteration => "交互式反馈循环，持续优化",
            VibeStage::Validation => "测试、错误修复和质量保证",
            VibeStage::Deployment => "部署到生产环境并监控",
        }
    }

    pub fn next(&self) -> Option<VibeStage> {
        match self {
            VibeStage::Conceptualization => Some(VibeStage::Generation),
            VibeStage::Generation => Some(VibeStage::Iteration),
            VibeStage::Iteration => Some(VibeStage::Validation),
            VibeStage::Validation => Some(VibeStage::Deployment),
            VibeStage::Deployment => None,
        }
    }
}

/// 产品需求文档 (PRD)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRequirementsDoc {
    pub project: VibeProject,
    pub version: String,
    pub created_at: DateTime<Local>,
    pub last_updated: DateTime<Local>,
    pub sections: HashMap<String, String>,
}

impl ProductRequirementsDoc {
    pub fn new(project: VibeProject) -> Self {
        let now = Local::now();
        let mut sections = HashMap::new();

        // 默认 PRD 章节
        sections.insert("概述".to_string(), String::new());
        sections.insert("目标用户".to_string(), String::new());
        sections.insert("核心功能".to_string(), String::new());
        sections.insert("技术要求".to_string(), String::new());
        sections.insert("验收标准".to_string(), String::new());
        sections.insert("时间线".to_string(), String::new());

        Self {
            project,
            version: "0.1.0".to_string(),
            created_at: now,
            last_updated: now,
            sections,
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut md = format!("# {}\n\n", self.project.name);
        md.push_str(&format!("**版本**: {}  **创建时间**: {}\n\n", self.version, self.created_at.format("%Y-%m-%d %H:%M")));
        md.push_str(&format!("**描述**: {}\n\n", self.project.description));

        if !self.project.tech_stack.is_empty() {
            md.push_str(&format!("**技术栈**: {}\n\n", self.project.tech_stack.join(", ")));
        }

        for (section, content) in &self.sections {
            md.push_str(&format!("## {}\n\n", section));
            if content.is_empty() {
                md.push_str("待填写...\n\n");
            } else {
                md.push_str(&format!("{}\n\n", content));
            }
        }

        md
    }
}

/// 技术设计文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDesignDoc {
    pub prd_id: String,
    pub version: String,
    pub created_at: DateTime<Local>,
    pub last_updated: DateTime<Local>,
    pub architecture: HashMap<String, String>,
    pub components: Vec<ComponentDesign>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDesign {
    pub name: String,
    pub description: String,
    pub file_path: String,
    pub dependencies: Vec<String>,
}

impl TechnicalDesignDoc {
    pub fn new(prd_id: String) -> Self {
        let now = Local::now();
        Self {
            prd_id,
            version: "0.1.0".to_string(),
            created_at: now,
            last_updated: now,
            architecture: HashMap::new(),
            components: Vec::new(),
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut md = String::from("# 技术设计文档\n\n");
        md.push_str(&format!("**版本**: {}  **创建时间**: {}\n\n", self.version, self.created_at.format("%Y-%m-%d %H:%M")));

        if !self.architecture.is_empty() {
            md.push_str("## 架构概述\n\n");
            for (key, value) in &self.architecture {
                md.push_str(&format!("### {}\n\n{}\n\n", key, value));
            }
        }

        if !self.components.is_empty() {
            md.push_str("## 组件设计\n\n");
            for component in &self.components {
                md.push_str(&format!("### {}\n\n", component.name));
                md.push_str(&format!("- **描述**: {}\n", component.description));
                md.push_str(&format!("- **文件路径**: `{}`\n", component.file_path));
                if !component.dependencies.is_empty() {
                    md.push_str(&format!("- **依赖**: {}\n", component.dependencies.join(", ")));
                }
                md.push_str("\n");
            }
        }

        md
    }
}

/// 代码变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub id: String,
    pub file_path: String,
    pub description: String,
    pub timestamp: DateTime<Local>,
    pub change_type: ChangeType,
    pub status: ChangeStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Create,
    Modify,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl CodeChange {
    pub fn new(file_path: String, description: String, change_type: ChangeType) -> Self {
        Self {
            id: generate_id(),
            file_path,
            description,
            timestamp: Local::now(),
            change_type,
            status: ChangeStatus::Pending,
        }
    }
}

/// Vibe Coding 工作流管理器
pub struct VibeWorkflowManager {
    project: Option<VibeProject>,
    pub stage: VibeStage,
    file_handler: CodeFileHandler,
    changes: Vec<CodeChange>,
}

impl VibeWorkflowManager {
    pub fn new() -> Self {
        Self {
            project: None,
            stage: VibeStage::Conceptualization,
            file_handler: CodeFileHandler::new(),
            changes: Vec::new(),
        }
    }

    /// Stage 1: 创建项目并生成 PRD
    pub fn create_project(&mut self, name: String, description: String) -> Result<VibeProject, String> {
        let project = VibeProject::new(name, description);
        let prd = ProductRequirementsDoc::new(project.clone());

        // 保存 PRD 文件
        let prd_path = PathBuf::from(format!("docs/prd_{}.md", project.id));
        if let Some(parent) = prd_path.parent() {
            let _ = self.file_handler.create_file(
                parent.to_str().unwrap(),
                "# Vibe Project\n\nThis directory was created using Vibe Coding workflow.",
            );
        }

        let result = self.file_handler.create_file(
            prd_path.to_str().unwrap(),
            &prd.to_markdown(),
        );

        match result.success {
            true => {
                self.project = Some(project.clone());
                Ok(project)
            }
            false => Err(format!("Failed to create PRD: {}", result.message)),
        }
    }

    /// Stage 2: 生成技术设计文档
    pub fn generate_technical_design(&mut self, prd: ProductRequirementsDoc) -> Result<TechnicalDesignDoc, String> {
        let mut design = TechnicalDesignDoc::new(prd.project.id.clone());

        // 基础架构设计
        design.architecture.insert(
            "整体架构".to_string(),
            "基于模块化的软件设计，包含前端、后端和数据库层。".to_string(),
        );

        let result = self.file_handler.create_file(
            &format!("docs/technical_design_{}.md", prd.project.id),
            &design.to_markdown(),
        );

        match result.success {
            true => Ok(design),
            false => Err(format!("Failed to create technical design: {}", result.message)),
        }
    }

    /// Stage 3: 记录代码变更
    pub fn record_change(&mut self, file_path: String, description: String, change_type: ChangeType) -> String {
        let change = CodeChange::new(file_path, description, change_type);
        let change_id = change.id.clone();
        self.changes.push(change);
        change_id
    }

    /// Stage 4: 获取当前状态
    pub fn get_status(&self) -> VibeStatus {
        VibeStatus {
            stage: self.stage,
            stage_name: self.stage.name().to_string(),
            stage_description: self.stage.description().to_string(),
            changes_count: self.changes.len(),
            completed_changes: self.changes.iter().filter(|c| c.status == ChangeStatus::Completed).count(),
        }
    }

    /// Stage 5: 进入下一阶段
    pub fn advance_stage(&mut self) -> Result<VibeStage, String> {
        if let Some(next_stage) = self.stage.next() {
            self.stage = next_stage;
            Ok(next_stage)
        } else {
            Err("Already at final stage".to_string())
        }
    }
}

/// Vibe Coding 状态报告
#[derive(Debug, Clone)]
pub struct VibeStatus {
    pub stage: VibeStage,
    pub stage_name: String,
    pub stage_description: String,
    pub changes_count: usize,
    pub completed_changes: usize,
}

impl VibeStatus {
    pub fn to_string(&self) -> String {
        format!(
            "阶段: {} ({})\n  {}",
            self.stage_name,
            self.stage_description,
            self.stage_description
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let mut manager = VibeWorkflowManager::new();
        let project = manager.create_project(
            "Test Project".to_string(),
            "A test project for vibe coding".to_string(),
        );

        assert!(project.is_ok());
        assert_eq!(manager.stage, VibeStage::Conceptualization);
    }

    #[test]
    fn test_stage_transitions() {
        let mut manager = VibeWorkflowManager::new();

        assert_eq!(manager.stage, VibeStage::Conceptualization);

        let next = manager.advance_stage();
        assert!(next.is_ok());
        assert_eq!(manager.stage, VibeStage::Generation);

        let next = manager.advance_stage();
        assert!(next.is_ok());
        assert_eq!(manager.stage, VibeStage::Iteration);
    }

    #[test]
    fn test_prd_generation() {
        let project = VibeProject::new("Test PRD".to_string(), "Test description".to_string());
        let prd = ProductRequirementsDoc::new(project);
        let markdown = prd.to_markdown();

        assert!(markdown.contains("Test PRD"));
        assert!(markdown.contains("概述"));
        assert!(markdown.contains("目标用户"));
    }
}
