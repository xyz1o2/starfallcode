use crate::core::context_optimizer::{ContextWindowOptimizer, ContextConfig};
use crate::ai::tools::PairProgrammingTools;
use crate::utils::code_file_handler::CodeFileHandler;

/// 集成管理器 - 统一管理三个核心模块
pub struct IntegrationManager {
    pub context_optimizer: ContextWindowOptimizer,
    pub tools: PairProgrammingTools,
    pub file_handler: CodeFileHandler,
}

impl IntegrationManager {
    /// 创建新的集成管理器
    pub fn new() -> Self {
        Self {
            context_optimizer: ContextWindowOptimizer::new(ContextConfig::default()),
            tools: PairProgrammingTools::new(),
            file_handler: CodeFileHandler::new(),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(context_config: ContextConfig) -> Self {
        Self {
            context_optimizer: ContextWindowOptimizer::new(context_config),
            tools: PairProgrammingTools::new(),
            file_handler: CodeFileHandler::new(),
        }
    }

    /// 启用 YOLO 模式（跳过确认）
    pub fn enable_yolo_mode(&mut self) {
        self.tools.enable_yolo_mode();
        self.file_handler.enable_yolo_mode();
    }

    /// 禁用 YOLO 模式
    pub fn disable_yolo_mode(&mut self) {
        self.tools.disable_yolo_mode();
        self.file_handler.disable_yolo_mode();
    }

    /// 获取状态信息
    pub fn get_status(&self) -> String {
        format!(
            "Integration Manager Status:\n\
             - Context Optimizer: Ready\n\
             - Tools: {} available\n\
             - File Handler: Ready\n\
             - YOLO Mode: {}",
            self.tools.get_available_tools().len(),
            if self.tools.is_yolo_mode() { "ON" } else { "OFF" }
        )
    }
}

impl Default for IntegrationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_manager_creation() {
        let manager = IntegrationManager::new();
        assert!(!manager.tools.get_available_tools().is_empty());
    }

    #[test]
    fn test_yolo_mode_toggle() {
        let mut manager = IntegrationManager::new();
        assert!(!manager.tools.is_yolo_mode());
        manager.enable_yolo_mode();
        assert!(manager.tools.is_yolo_mode());
        manager.disable_yolo_mode();
        assert!(!manager.tools.is_yolo_mode());
    }
}
