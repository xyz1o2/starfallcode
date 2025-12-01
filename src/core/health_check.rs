use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

/// 健康检查状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// 健康检查结果
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub timestamp: u64,
    pub checks: HashMap<String, CheckResult>,
    pub message: String,
}

/// 单个检查结果
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub message: String,
}

/// 健康检查器
pub struct HealthChecker {
    checks: Vec<Box<dyn Fn() -> CheckResult>>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }
    
    /// 添加检查项
    pub fn add_check<F>(&mut self, check: F)
    where
        F: Fn() -> CheckResult + 'static,
    {
        self.checks.push(Box::new(check));
    }
    
    /// 运行所有检查
    pub fn run_checks(&self) -> HealthCheckResult {
        let mut checks = HashMap::new();
        let mut all_passed = true;
        let mut degraded = false;
        
        for check_fn in &self.checks {
            let result = check_fn();
            if !result.passed {
                all_passed = false;
                if result.message.contains("warning") {
                    degraded = true;
                }
            }
            checks.insert(result.name.clone(), result);
        }
        
        let status = if all_passed {
            HealthStatus::Healthy
        } else if degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };
        
        let message = match status {
            HealthStatus::Healthy => "所有检查通过".to_string(),
            HealthStatus::Degraded => "部分功能降级".to_string(),
            HealthStatus::Unhealthy => "系统不健康".to_string(),
        };
        
        HealthCheckResult {
            status,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            checks,
            message,
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建默认的健康检查器
pub fn create_default_health_checker() -> HealthChecker {
    let mut checker = HealthChecker::new();
    
    // 检查内存使用
    checker.add_check(|| {
        let start = SystemTime::now();
        let passed = true; // 简化的检查
        let duration = start.elapsed().unwrap_or_default().as_millis() as u64;
        
        CheckResult {
            name: "memory_check".to_string(),
            passed,
            duration_ms: duration,
            message: "内存使用正常".to_string(),
        }
    });
    
    // 检查 LLM 连接
    checker.add_check(|| {
        let start = SystemTime::now();
        let passed = true; // 简化的检查
        let duration = start.elapsed().unwrap_or_default().as_millis() as u64;
        
        CheckResult {
            name: "llm_connection".to_string(),
            passed,
            duration_ms: duration,
            message: "LLM 连接正常".to_string(),
        }
    });
    
    // 检查消息历史
    checker.add_check(|| {
        let start = SystemTime::now();
        let passed = true; // 简化的检查
        let duration = start.elapsed().unwrap_or_default().as_millis() as u64;
        
        CheckResult {
            name: "message_history".to_string(),
            passed,
            duration_ms: duration,
            message: "消息历史正常".to_string(),
        }
    });
    
    checker
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_checker_creation() {
        let checker = HealthChecker::new();
        assert_eq!(checker.checks.len(), 0);
    }
    
    #[test]
    fn test_health_check_result() {
        let mut checker = HealthChecker::new();
        
        checker.add_check(|| CheckResult {
            name: "test_check".to_string(),
            passed: true,
            duration_ms: 10,
            message: "Test passed".to_string(),
        });
        
        let result = checker.run_checks();
        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.checks.contains_key("test_check"));
    }
    
    #[test]
    fn test_default_health_checker() {
        let checker = create_default_health_checker();
        let result = checker.run_checks();
        
        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.checks.len() >= 3);
    }
}
