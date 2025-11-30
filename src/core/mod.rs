pub mod buffer;
pub mod cursor;
pub mod history;
pub mod message;
pub mod context_optimizer;
pub mod integration;
pub mod conversation_engine;

pub use conversation_engine::{
    ConversationEngine, IntentRecognizer, ContextManager, ResponseProcessor,
    UserIntent, ConversationContext, ProcessedResponse, CodeModification,
    ModificationOperation, FileContent,
};