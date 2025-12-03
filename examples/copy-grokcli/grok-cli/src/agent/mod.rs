use crate::grok::client::GrokClient;
use crate::types::{ChatEntry, ChatEntryType, GrokMessage, GrokTool, GrokToolCall, GrokToolCallFunction, ToolResult, StreamingChunk, StreamingChunkType};
use crate::tools::{TextEditorTool, BashTool, TodoTool, SearchTool, ConfirmationTool, MorphEditorTool};
use std::collections::HashMap;
use std::pin::Pin;
use futures::Stream;

#[derive(Clone)]
pub struct GrokAgent {
    grok_client: GrokClient,
    text_editor: TextEditorTool,
    bash: BashTool,
    todo_tool: TodoTool,
    search: SearchTool,
    confirmation_tool: ConfirmationTool,
    morph_editor: Option<MorphEditorTool>,
    chat_history: Vec<ChatEntry>,
    messages: Vec<GrokMessage>,
    max_tool_rounds: u32,
}

impl GrokAgent {
    pub async fn new(
        api_key: &str,
        base_url: String,
        model: Option<String>,
        max_tool_rounds: Option<u32>,
        is_openai_compatible: Option<bool>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Adaptive tool rounds configuration (inspired by LangGraph and industry best practices)
        // Priority: explicit parameter > environment variable > model-based default > global default
        let tool_rounds = max_tool_rounds
            .or_else(|| std::env::var("GROK_MAX_TOOL_ROUNDS").ok().and_then(|v| v.parse().ok()))
            .or_else(|| {
                // Model-based defaults: complex models get more rounds
                let model_name = model.as_deref().unwrap_or("grok-code-fast-1");
                match model_name {
                    m if m.contains("claude") => Some(15),      // Claude: more reasoning
                    m if m.contains("gpt-4") => Some(12),       // GPT-4: good balance
                    m if m.contains("grok") => Some(10),        // Grok: efficient
                    m if m.contains("qwen") => Some(8),         // Qwen: lightweight
                    _ => Some(5),                                // Unknown: conservative
                }
            })
            .unwrap_or(5);
        let client = GrokClient::new(api_key, model, Some(base_url), is_openai_compatible);
        let text_editor = TextEditorTool::new();
        let bash = BashTool::new();
        let todo_tool = TodoTool::new();
        let search = SearchTool::new();
        let confirmation_tool = ConfirmationTool::new();
        let morph_editor = if let Ok(_) = std::env::var("MORPH_API_KEY") {
            Some(MorphEditorTool::new(None))
        } else {
            None
        };

        // Initialize with system message
        let system_message = GrokMessage {
            role: "system".to_string(),
            content: Some("You are Grok CLI, an AI assistant that helps with file editing, coding tasks, and system operations.

You have access to these tools:
- view_file: View file contents or directory listings
- create_file: Create new files with content (ONLY use this for files that don't exist yet)
- str_replace_editor: Replace text in existing files (ALWAYS use this to edit or update existing files)
- bash: Execute bash commands (use for searching, file discovery, navigation, and system operations)
- search: Unified search tool for finding text content or files (similar to Cursor's search functionality)
- create_todo_list: Create a visual todo list for planning and tracking tasks
- update_todo_list: Update existing todos in your todo list
- request_confirmation: Request user confirmation for operations
- check_session_acceptance: Check which operations are accepted for this session
- edit_file: High-speed file editing with Morph Fast Apply (4,500+ tokens/sec with 98% accuracy)

REAL-TIME INFORMATION:
You have access to real-time web search and X (Twitter) data. When users ask for current information, latest news, or recent events, you automatically have access to up-to-date information from the web and social media.

IMPORTANT TOOL USAGE RULES:
- NEVER use create_file on files that already exist - this will overwrite them completely
- ALWAYS use str_replace_editor to modify existing files, even for small changes
- Before editing a file, use view_file to see its current contents
- Use create_file ONLY when creating entirely new files that don't exist

USER CONFIRMATION SYSTEM:
File operations (create_file, str_replace_editor) and bash commands will automatically request user confirmation before execution. The confirmation system will show users the actual content or command before they decide. Users can choose to approve individual operations or approve all operations of that type for the session.

If a user rejects an operation, the tool will return an error and you should not proceed with that specific operation.

SEARCHING AND EXPLORATION:
- Use search for fast, powerful text search across files or finding files by name (unified search tool)
- Examples: search for text content like \"import.*react\", search for files like \"component.tsx\"
- Use bash with commands like 'find', 'grep', 'rg', 'ls' for complex file operations and navigation
- view_file is best for reading specific files you already know exist

When a user asks you to edit, update, modify, or change an existing file:
1. First use view_file to see the current contents
2. Then use str_replace_editor to make the specific changes
3. Never use create_file for existing files
4. Use edit_file (Morph Fast Apply) for complex edits requiring full context

TASK PLANNING WITH TODO LISTS:
- For complex requests with multiple steps, ALWAYS create a todo list first to plan your approach
- Use create_todo_list to break down tasks into manageable items with priorities
- Mark tasks as 'in_progress' when you start working on them (only one at a time)
- Mark tasks as 'completed' immediately when finished
- Use update_todo_list to track your progress throughout the task
- Todo lists provide visual feedback with colors: ✅ Green (completed), 🔄 Cyan (in progress), ⏳ Yellow (pending)
- Always create todos with priorities: 'high' (🔴), 'medium' (🟡), 'low' (🟢)

Be helpful, direct, and efficient. Always explain what you're doing and show the results.

IMPORTANT RESPONSE GUIDELINES:
- After using tools, do NOT respond with pleasantries like \"Thanks for...\" or \"Great!\"
- Only provide necessary explanations or next steps if relevant to the task
- Keep responses concise and focused on the actual work being done
- If a tool execution completes the user's request, you can remain silent or give a brief confirmation

Current working directory: ".to_string() + &std::env::current_dir()?.to_string_lossy()),
            tool_calls: None,
            tool_call_id: None,
        };

        Ok(GrokAgent {
            grok_client: client,
            text_editor,
            bash,
            todo_tool,
            search,
            confirmation_tool,
            morph_editor,
            chat_history: Vec::new(),
            messages: vec![system_message],
            max_tool_rounds: tool_rounds,
        })
    }

    pub async fn process_user_message(&mut self, message: &str) -> Result<Vec<ChatEntry>, Box<dyn std::error::Error>> {
        // Add user message to conversation
        let user_entry = ChatEntry {
            entry_type: ChatEntryType::User,
            content: message.to_string(),
            timestamp: chrono::Utc::now(),
            tool_calls: None,
            tool_call: None,
            tool_result: None,
            is_streaming: None,
        };
        self.chat_history.push(user_entry.clone());
        self.messages.push(GrokMessage {
            role: "user".to_string(),
            content: Some(message.to_string()),
            tool_calls: None,
            tool_call_id: None,
        });

        let mut new_entries = vec![user_entry.clone()];
        let mut tool_rounds = 0;
        let mut last_tool_signature: String = String::new();  // Track tool name + arguments for loop detection
        let mut repeated_calls = 0;  // Count repeated identical tool calls

        // Get all available tools
        let tools = self.get_all_tools().await;

        let mut current_response = match self.grok_client.chat(
            self.messages.clone(),
            Some(tools),
            None,
            None,
        ).await {
            Ok(response) => response,
            Err(e) => {
                if e.to_string().contains("No API key set") {
                    let error_entry = ChatEntry {
                        entry_type: ChatEntryType::Assistant,
                        content: "No API key configured. Please set your API key in settings before proceeding with chat functionality.".to_string(),
                        timestamp: chrono::Utc::now(),
                        tool_calls: None,
                        tool_call: None,
                        tool_result: None,
                        is_streaming: None,
                    };
                    self.chat_history.push(error_entry.clone());
                    return Ok(vec![user_entry, error_entry]);
                } else {
                    return Err(e);
                }
            }
        };

        // Agent loop - continue until no more tool calls or max rounds reached
        while tool_rounds < self.max_tool_rounds {
            let assistant_message = match current_response.choices.first() {
                Some(choice) => &choice.message,
                None => break,
            };

            // Check if there are tool calls (must be non-empty)
            if let Some(tool_calls) = &assistant_message.tool_calls {
                if tool_calls.is_empty() {
                    // Empty tool_calls array - treat as no tool calls
                    let final_entry = ChatEntry {
                        entry_type: ChatEntryType::Assistant,
                        content: assistant_message.content.clone().unwrap_or_else(|| "I understand, but I don't have a specific response.".to_string()),
                        timestamp: chrono::Utc::now(),
                        tool_calls: None,
                        tool_call: None,
                        tool_result: None,
                        is_streaming: None,
                    };
                    self.chat_history.push(final_entry.clone());
                    new_entries.push(final_entry);

                    self.messages.push(GrokMessage {
                        role: "assistant".to_string(),
                        content: assistant_message.content.clone(),
                        tool_calls: None,
                        tool_call_id: None,
                    });
                    break; // Exit the loop
                }
                
                tool_rounds += 1;

                // Loop detection: check if same tools with same arguments are being called repeatedly
                // Create a signature of tool name + arguments for comparison
                let current_signature = tool_calls
                    .iter()
                    .map(|tc| format!("{}({})", tc.function.name, tc.function.arguments))
                    .collect::<Vec<_>>()
                    .join(";");
                
                if current_signature == last_tool_signature && !current_signature.is_empty() {
                    repeated_calls += 1;
                    if repeated_calls >= 2 {
                        // Same tool call with same arguments 3 times in a row - infinite loop
                        let tool_desc = if current_signature.is_empty() {
                            "unknown tool".to_string()
                        } else {
                            current_signature.clone()
                        };
                        
                        let warning_entry = ChatEntry {
                            entry_type: ChatEntryType::Assistant,
                            content: format!(
                                "⚠️ Infinite loop detected: {} called {} times with identical parameters. This suggests the tool is not making progress. Stopping to prevent infinite loops.",
                                tool_desc,
                                repeated_calls + 1
                            ),
                            timestamp: chrono::Utc::now(),
                            tool_calls: None,
                            tool_call: None,
                            tool_result: None,
                            is_streaming: None,
                        };
                        self.chat_history.push(warning_entry.clone());
                        new_entries.push(warning_entry);
                        break;
                    }
                } else {
                    repeated_calls = 0;
                    last_tool_signature = current_signature;
                }

                // Add assistant message with tool calls
                let assistant_entry = ChatEntry {
                    entry_type: ChatEntryType::Assistant,
                    content: assistant_message.content.clone().unwrap_or_else(|| "Using tools to help you...".to_string()),
                    timestamp: chrono::Utc::now(),
                    tool_calls: Some(tool_calls.clone()),
                    tool_call: None,
                    tool_result: None,
                    is_streaming: None,
                };
                self.chat_history.push(assistant_entry.clone());
                new_entries.push(assistant_entry);

                // Add assistant message to conversation
                self.messages.push(assistant_message.clone());

                // Execute tool calls
                for tool_call in tool_calls {
                    let result = self.execute_tool(tool_call).await?;
                    let result_content = if result.success {
                        result.output.clone().unwrap_or_else(|| "Success".to_string())
                    } else {
                        result.error.clone().unwrap_or_else(|| "Error occurred".to_string())
                    };

                    // Add tool result entry
                    let tool_result_entry = ChatEntry {
                        entry_type: ChatEntryType::ToolResult,
                        content: result_content.clone(),
                        timestamp: chrono::Utc::now(),
                        tool_calls: None,
                        tool_call: Some(tool_call.clone()),
                        tool_result: Some(result),
                        is_streaming: None,
                    };
                    self.chat_history.push(tool_result_entry.clone());
                    new_entries.push(tool_result_entry);

                    // Add tool result to messages with proper format (needed for AI context)
                    self.messages.push(GrokMessage {
                        role: "tool".to_string(),
                        content: Some(result_content),
                        tool_calls: None,
                        tool_call_id: Some(tool_call.id.clone()),
                    });
                }

                // Get next response - this might contain more tool calls
                current_response = match self.grok_client.chat(
                    self.messages.clone(),
                    Some(self.get_all_tools().await),
                    None,
                    None,
                ).await {
                    Ok(response) => response,
                    Err(e) => {
                        if e.to_string().contains("No API key set") {
                            let error_entry = ChatEntry {
                                entry_type: ChatEntryType::Assistant,
                                content: "No API key configured. Please set your API key in settings before proceeding with chat functionality.".to_string(),
                                timestamp: chrono::Utc::now(),
                                tool_calls: None,
                                tool_call: None,
                                tool_result: None,
                                is_streaming: None,
                            };
                            self.chat_history.push(error_entry.clone());
                            new_entries.push(error_entry);
                            break; // Exit the loop
                        } else {
                            return Err(e);
                        }
                    }
                };
            } else {
                // No more tool calls, add final response
                let final_entry = ChatEntry {
                    entry_type: ChatEntryType::Assistant,
                    content: assistant_message.content.clone().unwrap_or_else(|| "I understand, but I don't have a specific response.".to_string()),
                    timestamp: chrono::Utc::now(),
                    tool_calls: None,
                    tool_call: None,
                    tool_result: None,
                    is_streaming: None,
                };
                self.chat_history.push(final_entry.clone());
                new_entries.push(final_entry);

                self.messages.push(GrokMessage {
                    role: "assistant".to_string(),
                    content: assistant_message.content.clone(),
                    tool_calls: None,
                    tool_call_id: None,
                });
                break; // Exit the loop
            }
        }

        if tool_rounds >= self.max_tool_rounds {
            let warning_entry = ChatEntry {
                entry_type: ChatEntryType::Assistant,
                content: "Maximum tool execution rounds reached. Stopping to prevent infinite loops.".to_string(),
                timestamp: chrono::Utc::now(),
                tool_calls: None,
                tool_call: None,
                tool_result: None,
                is_streaming: None,
            };
            self.chat_history.push(warning_entry.clone());
            new_entries.push(warning_entry);
        }

        Ok(new_entries)
    }

    async fn execute_tool(&mut self, tool_call: &GrokToolCall) -> Result<ToolResult, Box<dyn std::error::Error>> {
        let args: HashMap<String, serde_json::Value> = serde_json::from_str(&tool_call.function.arguments)?;

        match tool_call.function.name.as_str() {
            "view_file" => {
                let path = args.get("path").and_then(|v| v.as_str()).ok_or("Missing 'path' argument")?;
                let start_line = args.get("start_line").and_then(|v| v.as_u64()).map(|v| v as usize);
                let end_line = args.get("end_line").and_then(|v| v.as_u64()).map(|v| v as usize);

                let view_range = if let (Some(start), Some(end)) = (start_line, end_line) {
                    Some((start, end))
                } else {
                    None
                };

                match self.text_editor.view(path, view_range).await {
                    Ok(result) => Ok(result),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        data: None,
                    }),
                }
            },
            "create_file" => {
                let path = args.get("path").and_then(|v| v.as_str()).ok_or("Missing 'path' argument")?;
                let content = args.get("content").and_then(|v| v.as_str()).ok_or("Missing 'content' argument")?;

                match self.text_editor.create(path, content).await {
                    Ok(result) => Ok(result),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        data: None,
                    }),
                }
            },
            "str_replace_editor" => {
                let path = args.get("path").and_then(|v| v.as_str()).ok_or("Missing 'path' argument")?;
                let old_str = args.get("old_str").and_then(|v| v.as_str()).ok_or("Missing 'old_str' argument")?;
                let new_str = args.get("new_str").and_then(|v| v.as_str()).ok_or("Missing 'new_str' argument")?;
                let replace_all = args.get("replace_all").and_then(|v| v.as_bool()).unwrap_or(false);

                match self.text_editor.str_replace(path, old_str, new_str, replace_all).await {
                    Ok(result) => Ok(result),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        data: None,
                    }),
                }
            },
            "bash" => {
                let command = args.get("command").and_then(|v| v.as_str()).ok_or("Missing 'command' argument")?;

                match self.bash.execute(command, None).await {
                    Ok(result) => Ok(result),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        data: None,
                    }),
                }
            },
            "create_todo_list" => {
                let todos_value = args.get("todos").ok_or("Missing 'todos' argument")?;

                // Deserialize the todos array from the JSON value
                let todos: Vec<crate::tools::TodoItem> = serde_json::from_value(todos_value.clone())?;

                match self.todo_tool.create_todo_list(todos).await {
                    Ok(result) => Ok(result),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        data: None,
                    }),
                }
            },
            "update_todo_list" => {
                let updates_value = args.get("updates").ok_or("Missing 'updates' argument")?;

                // Deserialize the updates array from the JSON value
                let updates: Vec<crate::tools::TodoUpdate> = serde_json::from_value(updates_value.clone())?;

                match self.todo_tool.update_todo_list(updates).await {
                    Ok(result) => Ok(result),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        data: None,
                    }),
                }
            },
            "search" => {
                let query = args.get("query").and_then(|v| v.as_str()).ok_or("Missing 'query' argument")?;
                let search_type = args.get("search_type").and_then(|v| v.as_str()).map(|s| s.to_string());
                let include_pattern = args.get("include_pattern").and_then(|v| v.as_str()).map(|s| s.to_string());
                let exclude_pattern = args.get("exclude_pattern").and_then(|v| v.as_str()).map(|s| s.to_string());
                let case_sensitive = args.get("case_sensitive").and_then(|v| v.as_bool());
                let whole_word = args.get("whole_word").and_then(|v| v.as_bool());
                let regex = args.get("regex").and_then(|v| v.as_bool());
                let max_results = args.get("max_results").and_then(|v| v.as_u64()).map(|v| v as u32);
                let file_types_value = args.get("file_types");
                let exclude_files_value = args.get("exclude_files");
                let include_hidden = args.get("include_hidden").and_then(|v| v.as_bool());

                let file_types = if let Some(types_value) = file_types_value {
                    Some(serde_json::from_value(types_value.clone())?)
                } else {
                    None
                };

                let exclude_files = if let Some(files_value) = exclude_files_value {
                    Some(serde_json::from_value(files_value.clone())?)
                } else {
                    None
                };

                match self.search.search(
                    query,
                    search_type,
                    include_pattern,
                    exclude_pattern,
                    case_sensitive,
                    whole_word,
                    regex,
                    max_results,
                    file_types,
                    exclude_files,
                    include_hidden,
                ).await {
                    Ok(result) => Ok(result),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        data: None,
                    }),
                }
            },
            "request_confirmation" => {
                let operation = args.get("operation").and_then(|v| v.as_str()).ok_or("Missing 'operation' argument")?;
                let filename = args.get("filename").and_then(|v| v.as_str()).ok_or("Missing 'filename' argument")?;
                let description = args.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                let show_vscode_open = args.get("show_vscode_open").and_then(|v| v.as_bool());
                let auto_accept = args.get("auto_accept").and_then(|v| v.as_bool());

                let request = crate::tools::ConfirmationRequest {
                    operation: operation.to_string(),
                    filename: filename.to_string(),
                    description,
                    show_vscode_open,
                    auto_accept,
                };

                match self.confirmation_tool.request_confirmation(request).await {
                    Ok(result) => Ok(result),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        data: None,
                    }),
                }
            },
            "check_session_acceptance" => {
                match self.confirmation_tool.check_session_acceptance().await {
                    Ok(result) => Ok(result),
                    Err(e) => Ok(ToolResult {
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        data: None,
                    }),
                }
            },
            "edit_file" => {
                let target_file = args.get("target_file").and_then(|v| v.as_str()).ok_or("Missing 'target_file' argument")?;
                let instructions = args.get("instructions").and_then(|v| v.as_str()).ok_or("Missing 'instructions' argument")?;
                let code_edit = args.get("code_edit").and_then(|v| v.as_str()).ok_or("Missing 'code_edit' argument")?;

                if let Some(ref morph_editor) = self.morph_editor {
                    match morph_editor.edit_file(target_file, instructions, code_edit).await {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            success: false,
                            output: None,
                            error: Some(e.to_string()),
                            data: None,
                        }),
                    }
                } else {
                    Ok(ToolResult {
                        success: false,
                        output: None,
                        error: Some("Morph Fast Apply not available. Please set MORPH_API_KEY environment variable to use this feature.".to_string()),
                        data: None,
                    })
                }
            },
            _ => Ok(ToolResult {
                success: false,
                output: None,
                error: Some(format!("Unknown tool: {}", tool_call.function.name)),
                data: None,
            }),
        }
    }

    async fn get_all_tools(&self) -> Vec<GrokTool> {
        vec![
            // view_file tool
            GrokTool {
                tool_type: "function".to_string(),
                function: crate::types::GrokToolFunction {
                    name: "view_file".to_string(),
                    description: "View contents of a file or list directory contents".to_string(),
                    parameters: crate::types::GrokToolParameters {
                        param_type: "object".to_string(),
                        properties: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("path".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "Path to the file or directory to view"
                            }));
                            props.insert("start_line".to_string(), serde_json::json!({
                                "type": "integer",
                                "description": "Start line number for viewing a specific range (optional)"
                            }));
                            props.insert("end_line".to_string(), serde_json::json!({
                                "type": "integer",
                                "description": "End line number for viewing a specific range (optional)"
                            }));
                            props
                        },
                        required: vec!["path".to_string()],
                    },
                },
            },
            // create_file tool
            GrokTool {
                tool_type: "function".to_string(),
                function: crate::types::GrokToolFunction {
                    name: "create_file".to_string(),
                    description: "Create a new file with the specified content".to_string(),
                    parameters: crate::types::GrokToolParameters {
                        param_type: "object".to_string(),
                        properties: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("path".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "Path where the file should be created"
                            }));
                            props.insert("content".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "Content to write to the file"
                            }));
                            props
                        },
                        required: vec!["path".to_string(), "content".to_string()],
                    },
                },
            },
            // str_replace_editor tool
            GrokTool {
                tool_type: "function".to_string(),
                function: crate::types::GrokToolFunction {
                    name: "str_replace_editor".to_string(),
                    description: "Replace text in an existing file using exact string matching".to_string(),
                    parameters: crate::types::GrokToolParameters {
                        param_type: "object".to_string(),
                        properties: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("path".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "Path to the file to edit"
                            }));
                            props.insert("old_str".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "Original string to be replaced"
                            }));
                            props.insert("new_str".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "New string to replace with"
                            }));
                            props.insert("replace_all".to_string(), serde_json::json!({
                                "type": "boolean",
                                "description": "Whether to replace all occurrences (default: false)"
                            }));
                            props
                        },
                        required: vec!["path".to_string(), "old_str".to_string(), "new_str".to_string()],
                    },
                },
            },
            // bash tool
            GrokTool {
                tool_type: "function".to_string(),
                function: crate::types::GrokToolFunction {
                    name: "bash".to_string(),
                    description: "Execute bash commands (use for searching, file discovery, navigation, and system operations)".to_string(),
                    parameters: crate::types::GrokToolParameters {
                        param_type: "object".to_string(),
                        properties: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("command".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "The bash command to execute"
                            }));
                            props
                        },
                        required: vec!["command".to_string()],
                    },
                },
            },
            // create_todo_list tool
            GrokTool {
                tool_type: "function".to_string(),
                function: crate::types::GrokToolFunction {
                    name: "create_todo_list".to_string(),
                    description: "Create a visual todo list for planning and tracking tasks".to_string(),
                    parameters: crate::types::GrokToolParameters {
                        param_type: "object".to_string(),
                        properties: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("todos".to_string(), serde_json::json!({
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "id": {"type": "string", "description": "Unique identifier for the todo"},
                                        "content": {"type": "string", "description": "Description of the todo"},
                                        "status": {"type": "string", "enum": ["pending", "in_progress", "completed"]},
                                        "priority": {"type": "string", "enum": ["high", "medium", "low"]}
                                    },
                                    "required": ["id", "content", "status", "priority"]
                                },
                                "description": "List of todo items to create"
                            }));
                            props
                        },
                        required: vec!["todos".to_string()],
                    },
                },
            },
            // update_todo_list tool
            GrokTool {
                tool_type: "function".to_string(),
                function: crate::types::GrokToolFunction {
                    name: "update_todo_list".to_string(),
                    description: "Update existing todos in your todo list".to_string(),
                    parameters: crate::types::GrokToolParameters {
                        param_type: "object".to_string(),
                        properties: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("updates".to_string(), serde_json::json!({
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "id": {"type": "string", "description": "ID of the todo to update"},
                                        "status": {"type": "string", "enum": ["pending", "in_progress", "completed"], "description": "New status for the todo"},
                                        "content": {"type": "string", "description": "New content for the todo"},
                                        "priority": {"type": "string", "enum": ["high", "medium", "low"], "description": "New priority for the todo"}
                                    },
                                    "required": ["id"]
                                },
                                "description": "List of updates to apply to todo items"
                            }));
                            props
                        },
                        required: vec!["updates".to_string()],
                    },
                },
            },
            // search tool
            GrokTool {
                tool_type: "function".to_string(),
                function: crate::types::GrokToolFunction {
                    name: "search".to_string(),
                    description: "Unified search tool for finding text content or files (similar to Cursor's search functionality)".to_string(),
                    parameters: crate::types::GrokToolParameters {
                        param_type: "object".to_string(),
                        properties: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("query".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "The search query to execute"
                            }));
                            props.insert("search_type".to_string(), serde_json::json!({
                                "type": "string",
                                "enum": ["text", "files", "both"],
                                "description": "Type of search to perform: 'text' for content search, 'files' for file name search, 'both' for both",
                                "default": "both"
                            }));
                            props.insert("include_pattern".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "Glob pattern to include in search"
                            }));
                            props.insert("exclude_pattern".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "Glob pattern to exclude from search"
                            }));
                            props.insert("case_sensitive".to_string(), serde_json::json!({
                                "type": "boolean",
                                "description": "Whether to perform case-sensitive search",
                                "default": false
                            }));
                            props.insert("whole_word".to_string(), serde_json::json!({
                                "type": "boolean",
                                "description": "Whether to match whole words only",
                                "default": false
                            }));
                            props.insert("regex".to_string(), serde_json::json!({
                                "type": "boolean",
                                "description": "Whether to treat query as a regular expression",
                                "default": false
                            }));
                            props.insert("max_results".to_string(), serde_json::json!({
                                "type": "integer",
                                "description": "Maximum number of results to return",
                                "minimum": 1,
                                "maximum": 100,
                                "default": 50
                            }));
                            props.insert("file_types".to_string(), serde_json::json!({
                                "type": "array",
                                "items": {
                                    "type": "string"
                                },
                                "description": "File types to include in search (e.g., ['js', 'ts', 'py'])"
                            }));
                            props.insert("exclude_files".to_string(), serde_json::json!({
                                "type": "array",
                                "items": {
                                    "type": "string"
                                },
                                "description": "File patterns to exclude from search"
                            }));
                            props.insert("include_hidden".to_string(), serde_json::json!({
                                "type": "boolean",
                                "description": "Whether to include hidden files in search",
                                "default": false
                            }));
                            props
                        },
                        required: vec!["query".to_string()],
                    },
                },
            },
            // request_confirmation tool
            GrokTool {
                tool_type: "function".to_string(),
                function: crate::types::GrokToolFunction {
                    name: "request_confirmation".to_string(),
                    description: "Request user confirmation for operations".to_string(),
                    parameters: crate::types::GrokToolParameters {
                        param_type: "object".to_string(),
                        properties: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("operation".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "The operation to confirm"
                            }));
                            props.insert("filename".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "The filename associated with the operation"
                            }));
                            props.insert("description".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "Optional description of the operation"
                            }));
                            props.insert("show_vscode_open".to_string(), serde_json::json!({
                                "type": "boolean",
                                "description": "Whether to open the file in VS Code for review",
                                "default": false
                            }));
                            props.insert("auto_accept".to_string(), serde_json::json!({
                                "type": "boolean",
                                "description": "Whether to auto-accept the confirmation",
                                "default": false
                            }));
                            props
                        },
                        required: vec!["operation".to_string(), "filename".to_string()],
                    },
                },
            },
            // check_session_acceptance tool
            GrokTool {
                tool_type: "function".to_string(),
                function: crate::types::GrokToolFunction {
                    name: "check_session_acceptance".to_string(),
                    description: "Check which operations are accepted for this session".to_string(),
                    parameters: crate::types::GrokToolParameters {
                        param_type: "object".to_string(),
                        properties: std::collections::HashMap::new(),
                        required: vec![],
                    },
                },
            },
            // edit_file tool (Morph Fast Apply)
            GrokTool {
                tool_type: "function".to_string(),
                function: crate::types::GrokToolFunction {
                    name: "edit_file".to_string(),
                    description: "High-speed file editing with Morph Fast Apply (4,500+ tokens/sec with 98% accuracy)".to_string(),
                    parameters: crate::types::GrokToolParameters {
                        param_type: "object".to_string(),
                        properties: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("target_file".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "The file to edit"
                            }));
                            props.insert("instructions".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "Instructions for the edit"
                            }));
                            props.insert("code_edit".to_string(), serde_json::json!({
                                "type": "string",
                                "description": "The code edit to apply with Morph Fast Apply syntax (using // ... existing code ... comments for context)"
                            }));
                            props
                        },
                        required: vec!["target_file".to_string(), "instructions".to_string(), "code_edit".to_string()],
                    },
                },
            },
        ]
    }

    pub async fn process_user_message_stream(
        &mut self,
        message: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamingChunk, Box<dyn std::error::Error + Send>>> + Send>>, Box<dyn std::error::Error + Send>> {
        // Add user message to conversation
        self.messages.push(GrokMessage {
            role: "user".to_string(),
            content: Some(message.to_string()),
            tool_calls: None,
            tool_call_id: None,
        });

        let user_entry = ChatEntry {
            entry_type: ChatEntryType::User,
            content: message.to_string(),
            timestamp: chrono::Utc::now(),
            tool_calls: None,
            tool_call: None,
            tool_result: None,
            is_streaming: Some(true),
        };
        self.chat_history.push(user_entry);

        // Get all available tools
        let tools = self.get_all_tools().await;

        // Get streaming response from the client
        let stream = self.grok_client.chat_stream(
            self.messages.clone(),
            Some(tools),
            None,
            None,
        ).await?;

        use async_stream::stream;
        use futures::stream::StreamExt;

        let stream = Box::pin(stream! {
            let mut stream_pinned = std::pin::pin!(stream);
            let mut accumulated_content = String::new();
            let mut accumulated_tool_calls: Vec<GrokToolCall> = Vec::new();
            let mut current_tool_call_index: Option<usize> = None;
            
            while let Some(result) = stream_pinned.next().await {
                match result {
                    Ok(json) => {
                        // Parse the streaming response
                        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                            for choice in choices {
                                if let Some(delta) = choice.get("delta") {
                                    // Handle content streaming
                                    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                        if !content.is_empty() {
                                            accumulated_content.push_str(content);
                                            
                                            // Emit content chunk
                                            yield Ok(StreamingChunk {
                                                chunk_type: StreamingChunkType::Content,
                                                content: Some(content.to_string()),
                                                tool_calls: None,
                                                tool_call: None,
                                                tool_result: None,
                                                token_count: None,
                                            });
                                        }
                                    }

                                    // Handle tool calls
                                    if let Some(tool_calls) = delta.get("tool_calls").and_then(|tc| tc.as_array()) {
                                        for (idx, tool_call) in tool_calls.iter().enumerate() {
                                            if let Some(index) = tool_call.get("index").and_then(|i| i.as_u64()) {
                                                current_tool_call_index = Some(index as usize);
                                                
                                                if accumulated_tool_calls.len() <= index as usize {
                                                    accumulated_tool_calls.resize(index as usize + 1, GrokToolCall {
                                                        id: uuid::Uuid::new_v4().to_string(),
                                                        call_type: "function".to_string(),
                                                        function: GrokToolCallFunction {
                                                            name: String::new(),
                                                            arguments: String::new(),
                                                        },
                                                    });
                                                }

                                                let tool_call_ref = &mut accumulated_tool_calls[index as usize];

                                                if let Some(id) = tool_call.get("id").and_then(|i| i.as_str()) {
                                                    tool_call_ref.id = id.to_string();
                                                }

                                                if let Some(function) = tool_call.get("function") {
                                                    if let Some(name) = function.get("name").and_then(|n| n.as_str()) {
                                                        tool_call_ref.function.name = name.to_string();
                                                    }
                                                    if let Some(arguments) = function.get("arguments").and_then(|a| a.as_str()) {
                                                        tool_call_ref.function.arguments.push_str(arguments);
                                                    }
                                                }

                                                // Emit tool call chunk
                                                yield Ok(StreamingChunk {
                                                    chunk_type: StreamingChunkType::ToolCalls,
                                                    content: None,
                                                    tool_calls: Some(vec![tool_call_ref.clone()]),
                                                    tool_call: None,
                                                    tool_result: None,
                                                    token_count: None,
                                                });
                                            }
                                        }
                                    }
                                }

                                // Check for finish_reason
                                if let Some(finish_reason) = choice.get("finish_reason").and_then(|fr| fr.as_str()) {
                                    if finish_reason == "stop" || finish_reason == "tool_calls" {
                                        // Emit done chunk
                                        yield Ok(StreamingChunk {
                                            chunk_type: StreamingChunkType::Done,
                                            content: Some(accumulated_content.clone()),
                                            tool_calls: if accumulated_tool_calls.is_empty() { None } else { Some(accumulated_tool_calls.clone()) },
                                            tool_call: None,
                                            tool_result: None,
                                            token_count: None,
                                        });
                                    }
                                }
                            }
                        }

                        // Check for usage (token count)
                        if let Some(usage) = json.get("usage") {
                            if let Some(total_tokens) = usage.get("total_tokens").and_then(|t| t.as_u64()) {
                                yield Ok(StreamingChunk {
                                    chunk_type: StreamingChunkType::TokenCount,
                                    content: None,
                                    tool_calls: None,
                                    tool_call: None,
                                    tool_result: None,
                                    token_count: Some(total_tokens as u32),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(e);
                        break;
                    }
                }
            }
        });

        Ok(stream)
    }

    pub fn get_chat_history(&self) -> &Vec<ChatEntry> {
        &self.chat_history
    }
}