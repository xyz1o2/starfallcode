/// 日志和遥测系统
/// 
/// 提供结构化日志记录和性能遥测功能
/// 支持多种日志级别和输出格式

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Local};

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// 日志条目
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
    pub context: HashMap<String, String>,
}

impl LogEntry {
    pub fn new(level: LogLevel, module: String, message: String) -> Self {
        Self {
            timestamp: Local::now(),
            level,
            module,
            message,
            context: HashMap::new(),
        }
    }

    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }

    pub fn format_json(&self) -> String {
        let context_str = self
            .context
            .iter()
            .map(|(k, v)| format!(r#""{}":"{}""#, k, v))
            .collect::<Vec<_>>()
            .join(",");

        format!(
            r#"{{"timestamp":"{}","level":"{}","module":"{}","message":"{}","context":{{{}}}}}"#,
            self.timestamp.to_rfc3339(),
            self.level,
            self.module,
            self.message,
            context_str
        )
    }

    pub fn format_text(&self) -> String {
        let context_str = if self.context.is_empty() {
            String::new()
        } else {
            let ctx = self
                .context
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(" ");
            format!(" [{}]", ctx)
        };

        format!(
            "[{}] {} [{}] {}{}\n",
            self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            self.level,
            self.module,
            self.message,
            context_str
        )
    }
}

/// 日志记录器
pub struct Logger {
    entries: Arc<Mutex<Vec<LogEntry>>>,
    min_level: LogLevel,
    max_entries: usize,
}

impl Logger {
    pub fn new(min_level: LogLevel) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            min_level,
            max_entries: 10000,
        }
    }

    pub fn log(&self, entry: LogEntry) {
        if entry.level < self.min_level {
            return;
        }

        if let Ok(mut entries) = self.entries.lock() {
            entries.push(entry);

            // 保持日志大小在限制内
            let len = entries.len();
            if len > self.max_entries {
                entries.drain(0..len - self.max_entries);
            }
        }
    }

    pub fn trace(&self, module: &str, message: &str) {
        self.log(LogEntry::new(LogLevel::Trace, module.to_string(), message.to_string()));
    }

    pub fn debug(&self, module: &str, message: &str) {
        self.log(LogEntry::new(LogLevel::Debug, module.to_string(), message.to_string()));
    }

    pub fn info(&self, module: &str, message: &str) {
        self.log(LogEntry::new(LogLevel::Info, module.to_string(), message.to_string()));
    }

    pub fn warn(&self, module: &str, message: &str) {
        self.log(LogEntry::new(LogLevel::Warn, module.to_string(), message.to_string()));
    }

    pub fn error(&self, module: &str, message: &str) {
        self.log(LogEntry::new(LogLevel::Error, module.to_string(), message.to_string()));
    }

    pub fn get_entries(&self) -> Vec<LogEntry> {
        self.entries.lock().ok().map(|e| e.clone()).unwrap_or_default()
    }

    pub fn get_entries_by_level(&self, level: LogLevel) -> Vec<LogEntry> {
        self.get_entries()
            .into_iter()
            .filter(|e| e.level == level)
            .collect()
    }

    pub fn get_entries_by_module(&self, module: &str) -> Vec<LogEntry> {
        self.get_entries()
            .into_iter()
            .filter(|e| e.module == module)
            .collect()
    }

    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }

    pub fn export_json(&self) -> String {
        let entries = self.get_entries();
        let json_entries = entries
            .iter()
            .map(|e| e.format_json())
            .collect::<Vec<_>>()
            .join(",");
        format!("[{}]", json_entries)
    }

    pub fn export_text(&self) -> String {
        self.get_entries()
            .iter()
            .map(|e| e.format_text())
            .collect::<Vec<_>>()
            .join("")
    }
}

/// 性能遥测
#[derive(Debug, Clone)]
pub struct PerformanceMetric {
    pub name: String,
    pub duration_ms: f64,
    pub timestamp: DateTime<Local>,
    pub tags: HashMap<String, String>,
}

impl PerformanceMetric {
    pub fn new(name: String, duration_ms: f64) -> Self {
        Self {
            name,
            duration_ms,
            timestamp: Local::now(),
            tags: HashMap::new(),
        }
    }

    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }
}

/// 遥测收集器
pub struct Telemetry {
    metrics: Arc<Mutex<Vec<PerformanceMetric>>>,
    max_metrics: usize,
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            max_metrics: 10000,
        }
    }

    pub fn record(&self, metric: PerformanceMetric) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.push(metric);

            let len = metrics.len();
            if len > self.max_metrics {
                metrics.drain(0..len - self.max_metrics);
            }
        }
    }

    pub fn get_metrics(&self) -> Vec<PerformanceMetric> {
        self.metrics.lock().ok().map(|m| m.clone()).unwrap_or_default()
    }

    pub fn get_average_duration(&self, name: &str) -> f64 {
        let metrics = self.get_metrics();
        let matching: Vec<_> = metrics.iter().filter(|m| m.name == name).collect();

        if matching.is_empty() {
            return 0.0;
        }

        let sum: f64 = matching.iter().map(|m| m.duration_ms).sum();
        sum / matching.len() as f64
    }

    pub fn get_percentile(&self, name: &str, percentile: f64) -> f64 {
        let mut metrics = self.get_metrics();
        metrics.retain(|m| m.name == name);
        metrics.sort_by(|a, b| a.duration_ms.partial_cmp(&b.duration_ms).unwrap());

        if metrics.is_empty() {
            return 0.0;
        }

        let index = ((percentile / 100.0) * metrics.len() as f64) as usize;
        metrics[index.min(metrics.len() - 1)].duration_ms
    }

    pub fn clear(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.clear();
        }
    }

    pub fn export_summary(&self) -> String {
        let metrics = self.get_metrics();
        let mut summary = HashMap::new();

        for metric in metrics {
            let entry = summary.entry(metric.name.clone()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += metric.duration_ms;
        }

        let mut result = String::from("Performance Summary:\n");
        for (name, (count, total)) in summary {
            let avg = total / count as f64;
            result.push_str(&format!("  {}: count={}, avg={:.2}ms, total={:.2}ms\n", name, count, avg, total));
        }

        result
    }
}

impl Default for Telemetry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(LogLevel::Info, "test".to_string(), "message".to_string());
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.module, "test");
        assert_eq!(entry.message, "message");
    }

    #[test]
    fn test_log_entry_with_context() {
        let entry = LogEntry::new(LogLevel::Info, "test".to_string(), "message".to_string())
            .with_context("key".to_string(), "value".to_string());
        assert_eq!(entry.context.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_logger_filtering() {
        let logger = Logger::new(LogLevel::Info);
        logger.trace("test", "trace message");
        logger.info("test", "info message");
        logger.error("test", "error message");

        let entries = logger.get_entries();
        assert_eq!(entries.len(), 2); // trace should be filtered out
    }

    #[test]
    fn test_logger_by_level() {
        let logger = Logger::new(LogLevel::Trace);
        logger.info("test", "info");
        logger.error("test", "error");
        logger.warn("test", "warn");

        let errors = logger.get_entries_by_level(LogLevel::Error);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_logger_by_module() {
        let logger = Logger::new(LogLevel::Trace);
        logger.info("module1", "msg1");
        logger.info("module2", "msg2");
        logger.info("module1", "msg3");

        let module1 = logger.get_entries_by_module("module1");
        assert_eq!(module1.len(), 2);
    }

    #[test]
    fn test_telemetry_average() {
        let telemetry = Telemetry::new();
        telemetry.record(PerformanceMetric::new("test".to_string(), 10.0));
        telemetry.record(PerformanceMetric::new("test".to_string(), 20.0));
        telemetry.record(PerformanceMetric::new("test".to_string(), 30.0));

        let avg = telemetry.get_average_duration("test");
        assert!((avg - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_telemetry_percentile() {
        let telemetry = Telemetry::new();
        for i in 1..=100 {
            telemetry.record(PerformanceMetric::new("test".to_string(), i as f64));
        }

        let p50 = telemetry.get_percentile("test", 50.0);
        assert!(p50 > 40.0 && p50 < 60.0);
    }

    #[test]
    fn test_log_export_json() {
        let logger = Logger::new(LogLevel::Trace);
        logger.info("test", "message");

        let json = logger.export_json();
        assert!(json.contains("\"level\":\"INFO\""));
        assert!(json.contains("\"message\":\"message\""));
    }

    #[test]
    fn test_telemetry_summary() {
        let telemetry = Telemetry::new();
        telemetry.record(PerformanceMetric::new("op1".to_string(), 10.0));
        telemetry.record(PerformanceMetric::new("op1".to_string(), 20.0));
        telemetry.record(PerformanceMetric::new("op2".to_string(), 15.0));

        let summary = telemetry.export_summary();
        assert!(summary.contains("op1"));
        assert!(summary.contains("op2"));
    }
}
