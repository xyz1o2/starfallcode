# AI 代码修改确认工作流

## 概述

这是一个**用户确认优先**的 AI 代码修改系统。当 AI 在聊天中建议修改或创建代码时，系统会：

1. **自动检测** AI 回复中的代码修改指令
2. **生成 Diff 对比** 显示给用户
3. **等待用户确认** 后才执行修改

## 核心设计原则

✅ **用户优先** - 用户必须看到 Diff 再确认，绝不自动执行  
✅ **智能匹配** - 支持多层匹配策略（精确 → 空白不敏感 → 模糊）  
✅ **格式保留** - 保留原始代码的缩进和风格  
✅ **详细反馈** - 提供清晰的错误信息和匹配结果  

## 工作流程

### 1. AI 响应检测

当 AI 回复包含代码修改建议时，系统自动检测：

```
AI 回复:
"Modify `src/app.rs`:

```rust
pub fn new() -> Self {
    Self { /* ... */ }
}
```"

↓ 检测到 "Modify `src/app.rs`" 指令
↓ 提取代码块内容
↓ 生成修改操作
```

### 2. 支持的修改指令

#### 创建文件
```
create file `path/to/file.rs`
create `path/to/file.rs`
new file `path/to/file.rs`
```

#### 修改文件
```
modify `path/to/file.rs`
update `path/to/file.rs`
change `path/to/file.rs`
edit `path/to/file.rs`
```

#### 删除文件
```
delete `path/to/file.rs`
remove `path/to/file.rs`
```

### 3. 代码块提取

系统从 Markdown 代码块中提取内容：

```markdown
```rust
fn main() {
    println!("Hello");
}
```
```

提取的内容用于创建或修改文件。

### 4. Diff 生成和显示

对于修改操作，系统生成 Diff 对比：

```
--- src/app.rs (原始)
+++ src/app.rs (新版本)

- pub fn old_function() {
+ pub fn new_function() {
    // ...
  }
```

用户可以看到具体的改动。

### 5. 用户确认

显示确认对话：

```
⏳ 等待确认: 修改文件 src/app.rs

▶ 确认 (Confirm)
  取消 (Cancel)

按 ↑/↓ 切换选择，Enter 确认，Esc 取消
```

### 6. 执行修改

用户确认后，系统执行修改：

- **创建文件**: 创建新文件并写入内容
- **修改文件**: 查找并替换代码块
- **删除文件**: 删除指定文件

## 代码匹配策略

### 多层匹配算法

当修改文件时，系统使用多层匹配策略提高成功率：

#### 1. 精确匹配
直接搜索完全相同的代码块。

```rust
if old_content.contains(search) {
    // 精确匹配成功
}
```

#### 2. 空白不敏感匹配
忽略空白差异，比较规范化后的内容。

```rust
let search_normalized = normalize_whitespace(search);
let content_normalized = normalize_whitespace(&old_content);
if content_normalized.contains(&search_normalized) {
    // 空白不敏感匹配成功
}
```

#### 3. 模糊匹配
使用相似度算法找到最接近的代码块。

```rust
let similarity = string_similarity(search_line, content_line);
if similarity > 0.7 {
    // 模糊匹配成功
}
```

### 相似度计算

系统使用简化的相似度算法：

- **完全相同**: 1.0
- **规范化后相同**: 0.95
- **字符匹配率**: matches / max_len

## 实现细节

### 核心模块

**`src/ai/code_modification.rs`**

```rust
pub enum CodeModificationOp {
    Create { path: String, content: String },
    Modify { path: String, search: String, replace: String },
    Delete { path: String },
}

pub struct AICodeModificationDetector;
pub struct CodeMatcher;
```

### 检测流程

1. 提取 AI 响应中的所有代码块
2. 扫描响应文本查找修改指令
3. 匹配指令和代码块
4. 生成修改操作列表

### 应用流程

1. 读取目标文件
2. 尝试多层匹配找到搜索块
3. 生成 Diff 对比
4. 等待用户确认
5. 执行替换操作

## 使用示例

### 示例 1: 创建新文件

```
用户: 帮我创建一个 Rust 项目的 main.rs

AI: 我会为你创建一个基础的 main.rs 文件。

create file `src/main.rs`

```rust
fn main() {
    println!("Hello, world!");
}
```

系统响应:
⏳ 等待确认: 创建文件 src/main.rs

▶ 确认 (Confirm)
  取消 (Cancel)

用户按 Enter 确认
✅ 文件已创建: src/main.rs
```

### 示例 2: 修改现有文件

```
用户: 在 main 函数中添加一个变量

AI: 我会为你修改 main 函数。

modify `src/main.rs`

```rust
fn main() {
    let name = "Rust";
    println!("Hello, {}!", name);
}
```

系统响应:
⏳ 等待确认: 修改文件 src/main.rs

--- src/main.rs (原始)
+++ src/main.rs (新版本)

  fn main() {
+     let name = "Rust";
-     println!("Hello, world!");
+     println!("Hello, {}!", name);
  }

▶ 确认 (Confirm)
  取消 (Cancel)

用户按 Enter 确认
✅ 文件已修改: src/main.rs
```

## 错误处理

### 匹配失败

如果无法找到搜索块：

```
❌ 无法在文件中找到匹配的代码块:
fn old_function() {
    // ...
}

建议:
1. 检查代码块是否完全相同
2. 尝试使用更小的代码块
3. 检查缩进和空白
```

### 文件不存在

```
❌ 无法读取文件: src/app.rs (No such file or directory)
```

### 权限错误

```
❌ 无法写入文件: src/app.rs (Permission denied)
```

## 配置选项

### YOLO 模式

启用 YOLO 模式跳过确认（仅在完全信任 AI 时使用）：

```
/yolo-mode on
```

启用后，所有修改自动执行，不需要确认。

### 禁用 YOLO 模式

```
/yolo-mode off
```

## 最佳实践

### 对于 AI 模型

1. **清晰的指令**: 使用明确的修改指令
2. **完整的代码块**: 提供足够的上下文
3. **正确的缩进**: 保持原始文件的缩进风格
4. **单个操作**: 一次修改一个文件

### 对于用户

1. **仔细审查**: 总是检查 Diff 对比
2. **备份重要文件**: 修改前备份关键文件
3. **逐步修改**: 一次修改一个小改动
4. **验证结果**: 修改后验证代码是否正确

## 故障排除

### 问题: 修改失败，无法找到匹配

**原因**: 代码块与文件中的内容不完全相同

**解决方案**:
1. 复制文件中的确切代码
2. 保持相同的缩进和空白
3. 使用更小的代码块

### 问题: 修改了错误的代码

**原因**: 模糊匹配选择了不正确的代码块

**解决方案**:
1. 使用更具体的搜索块
2. 包含更多上下文
3. 禁用模糊匹配（仅精确匹配）

### 问题: 文件权限错误

**原因**: 没有写入文件的权限

**解决方案**:
1. 检查文件权限
2. 以管理员身份运行应用
3. 更改文件所有权

## 技术参考

### 相关文件

- `src/ai/code_modification.rs` - 核心实现
- `src/app.rs` - App 集成
- `src/events/handler.rs` - 事件处理
- `src/ui/mod.rs` - UI 显示

### 依赖

- `regex` - 正则表达式匹配
- `std::fs` - 文件操作

### 测试

运行单元测试：

```bash
cargo test code_modification
```

## 未来改进

- [ ] 支持多文件修改
- [ ] 撤销/重做功能
- [ ] 修改历史记录
- [ ] 自动备份
- [ ] 语法高亮 Diff
- [ ] 行号显示
- [ ] 更高级的模糊匹配算法
