use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorCommand {
    pub command: EditorCommandType,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_str: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_str: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_line: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorCommandType {
    View,
    StrReplace,
    Create,
    Insert,
    UndoEdit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatEntryType {
    User,
    Assistant,
    ToolResult,
    ToolCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatEntry {
    #[serde(rename = "type")]
    pub entry_type: ChatEntryType,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<GrokToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call: Option<GrokToolCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_result: Option<ToolResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_streaming: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrokMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<GrokToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrokTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: GrokToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrokToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: GrokToolParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrokToolParameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrokToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: GrokToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrokToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingChunk {
    pub chunk_type: StreamingChunkType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<GrokToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call: Option<GrokToolCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_result: Option<ToolResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamingChunkType {
    Content,
    ToolCalls,
    ToolResult,
    Done,
    TokenCount,
}