use ignore::WalkBuilder;
use std::path::PathBuf;

/// 文件搜索引擎 - 支持实时全文检索和模糊匹配
#[derive(Debug, Clone)]
pub struct FileSearchEngine {
    pub query: String,
    pub results: Vec<String>,
    pub selected_index: usize,
    pub root_path: PathBuf,
    pub cache: Vec<PathBuf>,  // 缓存所有文件
    pub cache_built: bool,
}

impl FileSearchEngine {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            selected_index: 0,
            root_path: PathBuf::from("."),
            cache: Vec::new(),
            cache_built: false,
        }
    }

    /// 设置根路径（项目根目录）
    pub fn set_root(&mut self, root: PathBuf) {
        self.root_path = root;
        self.cache_built = false; // 需要重新构建缓存
    }

    /// 构建文件缓存 - 应用启动时调用一次
    /// 递归遍历整个项目树，包含所有子目录
    /// 这样做的好处：
    /// - 应用启动时一次性加载，后续查询快速
    /// - 类似 Gemini CLI 的 list_directory 工具
    /// - 避免输入时卡顿
    pub fn build_cache(&mut self) {
        if self.cache_built {
            return;
        }

        self.cache.clear();

        // 使用 ignore crate 遍历文件，自动跳过 .gitignore 中的文件
        // 这与 Gemini CLI 的 list_directory 工具类似
        let walker = WalkBuilder::new(&self.root_path)
            .hidden(true)            // 隐藏隐藏文件（.git, .env 等）
            .ignore(true)            // 尊重 .gitignore
            .git_ignore(true)        // 尊重 .gitignore
            .max_depth(None)         // 无限深度 - 递归遍历整个树
            .build();

        for result in walker {
            if let Ok(entry) = result {
                let path = entry.path().to_path_buf();
                let path_str = path.to_string_lossy();

                // 跳过 target 和 .git 目录中的内容
                if !path_str.contains("target/")
                    && !path_str.contains("target\\")
                    && !path_str.contains(".git/")
                    && !path_str.contains(".git\\") {
                    // 同时保留文件和目录
                    self.cache.push(path);
                }
            }
        }

        // 排序缓存
        self.cache.sort();
        self.cache_built = true;
    }

    /// 更新查询并执行搜索
    pub fn update_query(&mut self, query: String) {
        self.query = query;
        self.selected_index = 0;
        self.search();
    }

    /// 执行搜索（支持全文检索和模糊匹配）
    /// 支持多种搜索模式：
    /// - @src -> 查找包含 "src" 的文件
    /// - @src/main -> 查找路径中包含 "src" 和 "main" 的文件
    /// - @.rs -> 查找扩展名为 .rs 的文件
    fn search(&mut self) {
        self.results.clear();

        // 移除 @ 符号
        let search_query = self.query.trim_start_matches('@').trim();

        if search_query.is_empty() {
            // 空查询 - 返回所有文件（限制数量）
            self.results = self
                .cache
                .iter()
                .take(20)
                .map(|p| format!("@{}", p.display()))
                .collect();
            return;
        }

        // 分割查询为多个关键词（仅在有空格或 / 时分割）
        let keywords: Vec<&str> = if search_query.contains(' ') || search_query.contains('/') {
            // 有空格或 / 时，按这些分隔符分割
            search_query
                .split(|c| c == ' ' || c == '/')
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            // 没有空格或 / 时，作为单个关键词
            vec![search_query]
        };

        // 执行多关键词搜索
        let mut matches: Vec<(String, usize)> = self
            .cache
            .iter()
            .filter_map(|path| {
                let path_str = path.to_string_lossy().to_lowercase();
                
                // 检查所有关键词是否都匹配
                let all_match = keywords.iter().all(|kw| {
                    let kw_lower = kw.to_lowercase();
                    path_str.contains(&kw_lower)
                });

                if all_match {
                    // 参考 Everything 搜索算法的排序策略
                    let mut score = 0usize;
                    
                    // 获取文件名
                    let file_name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    
                    // 优先级 1: 完整文件名匹配（Everything 的最高优先级）
                    if keywords.len() == 1 {
                        let kw_lower = keywords[0].to_lowercase();
                        if file_name == kw_lower {
                            // 精确匹配文件名
                            score += 1000000;
                        } else if file_name.starts_with(&kw_lower) {
                            // 文件名以关键词开头
                            score += 500000;
                        } else if file_name.contains(&kw_lower) {
                            // 文件名包含关键词
                            score += 100000;
                        }
                    }
                    
                    // 优先级 2: 路径中的匹配（Everything 的第二优先级）
                    for kw in &keywords {
                        let kw_lower = kw.to_lowercase();
                        
                        // 路径中的位置越前面得分越高
                        if let Some(pos) = path_str.find(&kw_lower) {
                            // 位置越前，得分越高
                            let position_score = 10000 - (pos as usize / 10).min(10000);
                            score += position_score;
                        }
                    }
                    
                    // 优先级 3: 目录优先级（src/ 目录下的文件优先）
                    if path_str.starts_with("src/") || path_str.starts_with("src\\") {
                        score += 50000;
                    }
                    
                    // 优先级 4: 路径深度（浅层文件优先）
                    let depth = path_str.matches('/').count() + path_str.matches('\\').count();
                    let depth_score = 1000 - (depth as usize * 100).min(1000);
                    score += depth_score;

                    Some((format!("@{}", path.display()), score))
                } else {
                    None
                }
            })
            .collect();

        // 按得分排序（降序）
        matches.sort_by(|a, b| b.1.cmp(&a.1));

        // 提取结果并限制数量
        self.results = matches
            .into_iter()
            .take(20)  // 增加显示数量
            .map(|(path, _)| path)
            .collect();
    }

    /// 向上选择
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if !self.results.is_empty() {
            self.selected_index = self.results.len() - 1;
        }
    }

    /// 向下选择
    pub fn select_next(&mut self) {
        if self.selected_index < self.results.len().saturating_sub(1) {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    /// 获取当前选中的结果
    pub fn get_selected(&self) -> Option<String> {
        self.results.get(self.selected_index).cloned()
    }

    /// 清空搜索
    pub fn clear(&mut self) {
        self.query.clear();
        self.results.clear();
        self.selected_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_search_engine() {
        let mut engine = FileSearchEngine::new();
        engine.build_cache();
        
        // 测试空查询
        engine.update_query("@".to_string());
        assert!(!engine.results.is_empty());

        // 测试单关键词搜索
        engine.update_query("@src".to_string());
        let has_src = engine.results.iter().any(|r| r.contains("src"));
        assert!(has_src || engine.results.is_empty());

        // 测试多关键词搜索
        engine.update_query("@src main".to_string());
        // 结果应该包含同时包含 "src" 和 "main" 的文件
    }
}
