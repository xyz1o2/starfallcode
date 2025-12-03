/// Todo Management Tool
/// Implements create and update functionality for Todo lists (similar to grok-cli todo-tool)

use super::tool::{Tool, ToolCall, ToolDefinition, ToolParameter, ToolResult, ToolExecutionContext};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::future::Future;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: usize,
    pub task: String,
    pub status: TodoStatus,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
}

impl std::fmt::Display for TodoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoStatus::Pending => write!(f, "⏳"),
            TodoStatus::InProgress => write!(f, "🔄"),
            TodoStatus::Completed => write!(f, "✅"),
        }
    }
}

/// Todo list manager
pub struct TodoManager {
    todos: Vec<TodoItem>,
}

impl TodoManager {
    pub fn new() -> Self {
        Self {
            todos: Vec::new(),
        }
    }

    pub fn create_list(&mut self, todos: Vec<TodoItem>) -> String {
        self.todos = todos;
        self.render_list()
    }

    pub fn update_list(&mut self, updates: Vec<TodoUpdate>) -> String {
        for update in updates {
            if let Some(todo) = self.todos.iter_mut().find(|t| t.id == update.id) {
                if let Some(new_task) = update.task {
                    todo.task = new_task;
                }
                if let Some(new_status) = update.status {
                    todo.status = match new_status.as_str() {
                        "pending" => TodoStatus::Pending,
                        "in_progress" => TodoStatus::InProgress,
                        "completed" => TodoStatus::Completed,
                        _ => todo.status.clone(),
                    };
                }
                if let Some(new_priority) = update.priority {
                    todo.priority = new_priority;
                }
            }
        }
        self.render_list()
    }

    fn render_list(&self) -> String {
        let mut output = String::from("# Todo List\n\n");
        for todo in &self.todos {
            let status_icon = format!("{}", todo.status);
            let priority_text = match todo.priority.as_str() {
                "high" => "🔴 high",
                "medium" => "🟡 medium",
                "low" => "🟢 low",
                _ => &todo.priority,
            };
            output.push_str(&format!("{} ({}): {}\n", status_icon, priority_text, todo.task));
        }
        output
    }
}

impl Default for TodoManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TodoUpdate {
    pub id: usize,
    pub task: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
}

/// Create Todo List Tool
pub struct CreateTodoListTool {
    manager: std::sync::Arc<tokio::sync::Mutex<TodoManager>>,
}

impl CreateTodoListTool {
    pub fn new(manager: std::sync::Arc<tokio::sync::Mutex<TodoManager>>) -> Self {
        Self { manager }
    }
}

impl Tool for CreateTodoListTool {
    fn name(&self) -> &str {
        "create_todo_list"
    }

    fn description(&self) -> &str {
        "Creates a visual Todo list for planning and tracking tasks. Used for complex multi-step requests. This is a user-visible todo list. Should be used as the first tool to setup todos. Priority must be 'high' (🔴), 'medium' (🟡), or 'low' (🟢)."
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: vec![
                ToolParameter {
                    name: "todos".to_string(),
                    description: "Todo items array, each contains id, task, status and priority".to_string(),
                    param_type: "array".to_string(),
                    required: true,
                },
            ],
        }
    }

    fn execute(&self, call: ToolCall) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>> {
        let manager = self.manager.clone();
        Box::pin(async move {
            let ctx = ToolExecutionContext::new(call.tool_name, call.arguments);

            match ctx.arguments.get("todos") {
                Some(todos_value) => {
                    match serde_json::from_value::<Vec<TodoItem>>(todos_value.clone()) {
                        Ok(todos) => {
                            let mut manager = manager.lock().await;
                            let rendered = manager.create_list(todos);

                            ToolResult {
                                success: true,
                                data: serde_json::json!({
                                    "rendered": rendered,
                                    "item_count": manager.todos.len()
                                }),
                                error: None,
                            }
                        }
                        Err(e) => ToolResult {
                            success: false,
                            data: serde_json::json!(null),
                            error: Some(format!("Failed to parse todos: {}", e)),
                        },
                    }
                }
                None => ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some("Missing required parameter: todos ".to_string()),
                },
            }
        })
    }
}

/// Update Todo List Tool
pub struct UpdateTodoListTool {
    manager: std::sync::Arc<tokio::sync::Mutex<TodoManager>>,
}

impl UpdateTodoListTool {
    pub fn new(manager: std::sync::Arc<tokio::sync::Mutex<TodoManager>>) -> Self {
        Self { manager }
    }
}

impl Tool for UpdateTodoListTool {
    fn name(&self) -> &str {
        "update_todo_list"
    }

    fn description(&self) -> &str {
        "Update existing todo list items. Used to mark items as in progress, completed, etc. Can update multiple items at once."
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: vec![
                ToolParameter {
                    name: "updates".to_string(),
                    description: "Update operations array, each contains id and fields to update".to_string(),
                    param_type: "array".to_string(),
                    required: true,
                },
            ],
        }
    }

    fn execute(&self, call: ToolCall) -> Pin<Box<dyn Future<Output = ToolResult> + Send + '_>> {
        let manager = self.manager.clone();
        Box::pin(async move {
            let ctx = ToolExecutionContext::new(call.tool_name, call.arguments);

            match ctx.arguments.get("updates") {
                Some(updates_value) => {
                    match serde_json::from_value::<Vec<TodoUpdate>>(updates_value.clone()) {
                        Ok(updates) => {
                            let update_count = updates.len();
                            let mut manager = manager.lock().await;
                            let rendered = manager.update_list(updates);

                            ToolResult {
                                success: true,
                                data: serde_json::json!({
                                    "rendered": rendered,
                                    "updated_count": update_count
                                }),
                                error: None,
                            }
                        }
                        Err(e) => ToolResult {
                            success: false,
                            data: serde_json::json!(null),
                            error: Some(format!("Failed to parse updates: {}", e)),
                        },
                    }
                }
                None => ToolResult {
                    success: false,
                    data: serde_json::json!(null),
                    error: Some("Missing required parameter: updates ".to_string()),
                },
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_manager_basics() {
        let mut manager = TodoManager::new();
        let todos = vec![
            TodoItem {
                id: 1,
                task: "Implement core".to_string(),
                status: TodoStatus::Pending,
                priority: "high".to_string(),
            },
        ];
        let rendered = manager.create_list(todos);
        assert!(rendered.contains("Implement core"));
    }
}
