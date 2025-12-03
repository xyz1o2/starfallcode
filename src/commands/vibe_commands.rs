/// Vibe Coding 命令处理
/// 处理 /vibc 开头的 vibecoding 工作流命令

use crate::core::vibe_coding::VibeWorkflowManager;

#[derive(Debug, Clone)]
pub enum VibeCommand {
    /// 创建新项目: /vibc new <name> <description>
    NewProject { name: String, description: String },
    /// 显示当前状态: /vibc status
    ShowStatus,
    /// 进入下一阶段: /vibc next
    NextStage,
    /// 列出所有阶段: /vibc stages
    ListStages,
    /// 生成 PRD: /vibc generate-prd
    GeneratePRD,
    /// 生成技术设计文档: /vibc generate-design
    GenerateDesign,
}

#[derive(Debug, Clone)]
pub struct VibeCommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<String>,
}

pub struct VibeCommandHandler {
    workflow_manager: VibeWorkflowManager,
}

impl VibeCommandHandler {
    pub fn new() -> Self {
        Self {
            workflow_manager: VibeWorkflowManager::new(),
        }
    }

    /// 解析命令
    pub fn parse(input: &str) -> Result<VibeCommand, String> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        if parts.is_empty() || parts[0] != "/vibc" {
            return Err("Not a vibe command".to_string());
        }

        if parts.len() < 2 {
            return Err("Missing vibe command action. Use: new, status, next, stages, generate-prd, generate-design".to_string());
        }

        match parts[1] {
            "new" => {
                if parts.len() < 4 {
                    return Err("Usage: /vibc new <project_name> <description>".to_string());
                }
                let name = parts[2].to_string();
                let description = parts[3..].join(" ");
                Ok(VibeCommand::NewProject { name, description })
            }
            "status" => Ok(VibeCommand::ShowStatus),
            "next" => Ok(VibeCommand::NextStage),
            "stages" => Ok(VibeCommand::ListStages),
            "generate-prd" => Ok(VibeCommand::GeneratePRD),
            "generate-design" => Ok(VibeCommand::GenerateDesign),
            _ => Err(format!("Unknown vibe command: {}. Available: new, status, next, stages, generate-prd, generate-design", parts[1])),
        }
    }

    /// 执行命令
    pub fn execute(&mut self, command: VibeCommand) -> VibeCommandResult {
        match command {
            VibeCommand::NewProject { name, description } => {
                self.new_project(name, description)
            }
            VibeCommand::ShowStatus => self.show_status(),
            VibeCommand::NextStage => self.next_stage(),
            VibeCommand::ListStages => self.list_stages(),
            VibeCommand::GeneratePRD => self.generate_prd(),
            VibeCommand::GenerateDesign => self.generate_design(),
        }
    }

    fn new_project(&mut self, name: String, description: String) -> VibeCommandResult {
        match self.workflow_manager.create_project(name.clone(), description) {
            Ok(project) => {
                VibeCommandResult {
                    success: true,
                    message: format!("✅ 项目 '{}' 创建成功!", project.name),
                    data: Some(format!("Project ID: {}", project.id)),
                }
            }
            Err(e) => VibeCommandResult {
                success: false,
                message: format!("❌ 创建项目失败: {}", e),
                data: None,
            },
        }
    }

    fn show_status(&self) -> VibeCommandResult {
        let status = self.workflow_manager.get_status();
        let details = format!(
            "当前阶段: {}\n{}\n\n变更统计:\n  - 总计: {}\n  - 已完成: {}\n  - 进行中: {}",
            status.stage_name,
            "─".repeat(40),
            status.changes_count,
            status.completed_changes,
            status.changes_count.saturating_sub(status.completed_changes)
        );

        VibeCommandResult {
            success: true,
            message: "工作流状态查询成功".to_string(),
            data: Some(details),
        }
    }

    fn next_stage(&mut self) -> VibeCommandResult {
        match self.workflow_manager.advance_stage() {
            Ok(stage) => {
                let stage_name = stage.name();
                VibeCommandResult {
                    success: true,
                    message: format!("✅ 已进入下一阶段: {}", stage_name),
                    data: Some(format!("当前阶段: {}", stage.description())),
                }
            }
            Err(e) => VibeCommandResult {
                success: false,
                message: format!("❌ 无法进入下一阶段: {}", e),
                data: None,
            },
        }
    }

    fn list_stages(&self) -> VibeCommandResult {
        use crate::core::vibe_coding::VibeStage;

        let stages = vec![
            (VibeStage::Conceptualization, "概念化", "定义需求，创建产品需求文档"),
            (VibeStage::Generation, "生成", "AI生成全栈代码和初始构建"),
            (VibeStage::Iteration, "迭代", "交互式反馈循环，持续优化"),
            (VibeStage::Validation, "验证", "测试、错误修复和质量保证"),
            (VibeStage::Deployment, "部署", "部署到生产环境并监控"),
        ];

        let mut output = String::from("Vibe Coding 5阶段工作流:\n\n");
        for (i, (stage, name, desc)) in stages.into_iter().enumerate() {
            let current = if stage == self.workflow_manager.stage { " (当前)" } else { "" };
            output.push_str(&format!("{}. {}{}\n   {}\n\n", i + 1, name, current, desc));
        }

        VibeCommandResult {
            success: true,
            message: "阶段列表查询成功".to_string(),
            data: Some(output),
        }
    }

    fn generate_prd(&mut self) -> VibeCommandResult {
        VibeCommandResult {
            success: true,
            message: "PRD 生成命令已接收".to_string(),
            data: Some("请提供项目详细信息以便生成完整的产品需求文档".to_string()),
        }
    }

    fn generate_design(&mut self) -> VibeCommandResult {
        VibeCommandResult {
            success: true,
            message: "技术设计文档生成命令已接收".to_string(),
            data: Some("基于 PRD 生成技术设计文档...".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_new_project() {
        let cmd = VibeCommandHandler::parse("/vibc new myapp A simple web app");
        assert!(cmd.is_ok());

        if let Ok(VibeCommand::NewProject { name, description }) = cmd {
            assert_eq!(name, "myapp");
            assert_eq!(description, "A simple web app");
        }
    }

    #[test]
    fn test_parse_status() {
        let cmd = VibeCommandHandler::parse("/vibc status");
        assert!(matches!(cmd, Ok(VibeCommand::ShowStatus)));
    }

    #[test]
    fn test_parse_next() {
        let cmd = VibeCommandHandler::parse("/vibc next");
        assert!(matches!(cmd, Ok(VibeCommand::NextStage)));
    }

    #[test]
    fn test_parse_unknown_command() {
        let cmd = VibeCommandHandler::parse("/vibc unknown");
        assert!(cmd.is_err());
    }

    #[test]
    fn test_execute_status() {
        let mut handler = VibeCommandHandler::new();
        let result = handler.execute(VibeCommand::ShowStatus);

        assert!(result.success);
        assert!(result.data.is_some());
    }

    #[test]
    fn test_execute_list_stages() {
        let handler = VibeCommandHandler::new();
        let result = handler.execute(VibeCommand::ListStages);

        assert!(result.success);
        assert!(result.data.is_some());
        let data = result.data.unwrap();
        assert!(data.contains("Vibe Coding 5阶段工作流"));
        assert!(data.contains("概念化"));
        assert!(data.contains("部署"));
    }
}
