pub mod client;
pub mod commands;
pub mod config;
pub mod context;
pub mod fim;
pub mod streaming;
pub mod advanced_client;
pub mod tools;
pub mod code_modification;
pub mod prompt_builder;

pub use prompt_builder::{ PromptBuilder, Message, RulesCompressor };