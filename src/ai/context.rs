use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Language, Parser, Query, QueryCursor};
use std::fs;

pub struct ProjectContext {
    pub symbols: Vec<Symbol>,
    pub file_contents: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub code: String,
}

pub struct RAGContextBuilder;

impl RAGContextBuilder {
    pub fn scan_project(project_path: &str) -> Result<ProjectContext, Box<dyn std::error::Error>> {
        let mut symbols = Vec::new();
        let mut file_contents = HashMap::new();
        
        // Walk through project files, skipping target/, .git/, etc.
        let walker = ignore::WalkBuilder::new(project_path)
            .filter_entry(|entry| {
                let path = entry.path();
                let name = path.file_name().unwrap_or_default().to_str().unwrap_or_default();
                
                // Skip common directories that shouldn't be scanned
                if name == "target" || name == ".git" || name == "node_modules" || name == ".vscode" {
                    return false;
                }
                
                // Only include source files
                if path.is_file() {
                    let ext = path.extension().unwrap_or_default().to_str().unwrap_or_default();
                    if matches!(ext, "rs" | "py" | "js" | "ts" | "go" | "java" | "cpp" | "c" | "h" | "hpp") {
                        return true;
                    }
                    return false;
                }
                
                true
            })
            .build();
        
        for entry in walker {
            let entry = entry?;
            if entry.file_type().map_or(false, |t| t.is_file()) {
                let file_path = entry.path().to_string_lossy().to_string();
                if let Ok(content) = fs::read_to_string(&file_path) {
                    file_contents.insert(file_path.clone(), content.clone());
                    
                    // Extract symbols using tree-sitter based on file extension
                    if let Some(language) = get_language_for_file(&file_path) {
                        let file_symbols = Self::extract_symbols(&content, &file_path, language)?;
                        symbols.extend(file_symbols);
                    }
                }
            }
        }
        
        Ok(ProjectContext {
            symbols,
            file_contents,
        })
    }
    
    fn extract_symbols(content: &str, file_path: &str, language: Language) -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
        let mut parser = Parser::new();
        parser.set_language(language)?;
        
        let tree = parser.parse(content, None).unwrap();
        let mut symbols = Vec::new();
        
        // Get appropriate query for the language to extract symbols
        let query_text = match language {
            lang if std::ptr::eq(lang, tree_sitter_rust::LANGUAGE.into()) => {
                r#"
                    (function_item
                        name: (identifier) @function_name)
                    (struct_item
                        name: (type_identifier) @struct_name)
                    (impl_item
                        type: (_) @impl_type)
                    (trait_item
                        name: (type_identifier) @trait_name)
                    (enum_item
                        name: (type_identifier) @enum_name)
                    (const_item
                        name: (identifier) @const_name)
                    (macro_definition
                        name: (identifier) @macro_name)
                "#
            },
            lang if std::ptr::eq(lang, tree_sitter_python::LANGUAGE.into()) => {
                r#"
                    (function_definition
                        name: (identifier) @function_name)
                    (class_definition
                        name: (identifier) @class_name)
                    (decorator) @decorator
                "#
            },
            _ => return Ok(vec![]), // Return empty for unsupported languages
        };
        
        let query = Query::new(language, query_text).map_err(|e| format!("Query error: {:?}", e))?;
        
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
        
        for match_result in matches {
            for capture in match_result.captures {
                let node = capture.node;
                let start = node.start_position();
                let name = &content[node.start_byte()..node.end_byte()];
                
                let symbol = Symbol {
                    name: name.to_string(),
                    kind: capture.index.to_string(), // This would need mapping to actual names
                    file_path: file_path.to_string(),
                    line: start.row,
                    column: start.column,
                    code: node.utf8_text(content.as_bytes()).unwrap_or("").to_string(),
                };
                
                symbols.push(symbol);
            }
        }
        
        Ok(symbols)
    }
    
    pub fn build_context_for_file(file_path: &str, project_context: &ProjectContext, max_context_tokens: usize) -> String {
        let mut context = String::new();
        
        // Find related symbols based on file path
        let file_name = Path::new(file_path)
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        
        // Add symbols that might be relevant to the current file
        for symbol in &project_context.symbols {
            if symbol.file_path.contains(file_name) || 
               symbol.name.to_lowercase().contains(&file_name.to_lowercase()) {
                context.push_str(&format!(
                    "File: {} | Line: {} | {} {}: {}\n",
                    symbol.file_path,
                    symbol.line + 1,
                    symbol.kind,
                    symbol.name,
                    symbol.code
                ));
            }
        }
        
        // Limit context size if needed
        if context.len() > max_context_tokens {
            context.truncate(max_context_tokens);
        }
        
        context
    }
}

// Helper function to get the appropriate tree-sitter language for a file
fn get_language_for_file(file_path: &str) -> Option<Language> {
    let ext = std::path::Path::new(file_path)
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_lowercase();
    
    match ext.as_str() {
        "rs" => Some(tree_sitter_rust::LANGUAGE.into()),
        "py" => Some(tree_sitter_python::LANGUAGE.into()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_language_for_file() {
        assert!(get_language_for_file("test.rs").is_some());
        assert!(get_language_for_file("test.py").is_some());
        assert!(get_language_for_file("test.js").is_none());
    }
}