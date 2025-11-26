# AI 代码修改 - 快速开始指南

## 核心概念

当 AI 建议修改代码时，系统会：
1. 检测修改指令
2. 显示 Diff 对比
3. 等待用户确认

**绝不自动执行修改** - 用户必须确认！

## 支持的指令格式

### 创建文件

```
create file `src/main.rs`
create `src/main.rs`
new file `src/main.rs`
```

### 修改文件

```
modify `src/app.rs`
update `src/app.rs`
change `src/app.rs`
edit `src/app.rs`
```

### 删除文件

```
delete `src/main.rs`
remove `src/main.rs`
```

## 工作流

### 第 1 步: AI 提供代码

```
AI: 我会为你创建一个 Rust 项目。

create file `src/main.rs`

```rust
fn main() {
    println!("Hello, world!");
}
```
```

### 第 2 步: 系统检测和显示

```
⏳ 等待确认: 创建文件 src/main.rs

▶ 确认 (Confirm)
  取消 (Cancel)
```

### 第 3 步: 用户确认

- 按 **↑** 或 **↓** 切换选择
- 按 **Enter** 确认
- 按 **Esc** 取消

### 第 4 步: 执行修改

```
✅ 文件已创建: src/main.rs
```

## 修改文件的 Diff 显示

对于修改操作，系统显示 Diff 对比：

```
--- src/app.rs (原始)
+++ src/app.rs (新版本)

  fn main() {
-     println!("Hello");
+     println!("Hello, world!");
  }
```

- **-** 表示删除的行
- **+** 表示添加的行
- 其他行表示上下文

## 常见场景

### 场景 1: 创建新文件

```
用户: 创建一个 Rust 项目结构

AI: 我会为你创建 main.rs 和 lib.rs

create file `src/main.rs`

```rust
mod lib;

fn main() {
    lib::hello();
}
```

create file `src/lib.rs`

```rust
pub fn hello() {
    println!("Hello from lib!");
}
```

系统: 显示两个确认对话，用户逐个确认
```

### 场景 2: 修改现有文件

```
用户: 在 main 函数中添加错误处理

AI: 我会为你改进 main 函数

modify `src/main.rs`

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    lib::hello()?;
    Ok(())
}
```

系统: 显示 Diff，用户确认修改
```

### 场景 3: 删除文件

```
用户: 删除不需要的文件

AI: 我会删除 old.rs

delete `src/old.rs`

系统: 显示确认对话
```

## 代码匹配策略

系统使用 **3 层匹配策略**：

### 1️⃣ 精确匹配
完全相同的代码块 → 直接替换

### 2️⃣ 空白不敏感匹配
忽略空白差异 → 替换

### 3️⃣ 模糊匹配
相似度 > 70% → 替换

**示例**:
```
搜索块:
fn main() {
    println!("Hello");
}

文件中的代码:
fn main() {
  println!("Hello");  // 缩进不同，但能匹配
}

结果: ✅ 模糊匹配成功
```

## 错误处理

### ❌ 无法找到匹配

```
❌ 无法在文件中找到匹配的代码块:
fn old_function() {
    // ...
}
```

**解决方案**:
1. 使用文件中的确切代码
2. 保持相同的缩进
3. 包含更多上下文

### ❌ 文件不存在

```
❌ 无法读取文件: src/app.rs
```

**解决方案**:
1. 检查文件路径
2. 确保文件存在
3. 检查文件权限

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| ↑ | 选择"确认" |
| ↓ | 选择"取消" |
| Enter | 执行选择 |
| Esc | 取消修改 |

## 配置

### 启用 YOLO 模式（自动执行）

```
/yolo-mode on
```

⚠️ **警告**: 仅在完全信任 AI 时使用！

### 禁用 YOLO 模式

```
/yolo-mode off
```

## 最佳实践

✅ **DO**
- 仔细审查 Diff 对比
- 一次修改一个文件
- 备份重要文件
- 验证修改结果

❌ **DON'T**
- 不要盲目确认
- 不要同时修改多个文件
- 不要在生产环境直接应用
- 不要启用 YOLO 模式除非必要

## 故障排除

### 问题: 修改了错误的代码

**原因**: 模糊匹配选择了不同的代码块

**解决方案**:
1. 使用更具体的搜索块
2. 包含更多上下文
3. 手动修改文件

### 问题: 缩进错误

**原因**: 代码块的缩进与文件不同

**解决方案**:
1. 复制文件中的确切代码
2. 保持相同的缩进风格
3. 使用 Tab 或空格一致

### 问题: 权限被拒绝

**原因**: 没有写入文件的权限

**解决方案**:
1. 检查文件权限: `ls -l src/app.rs`
2. 更改权限: `chmod 644 src/app.rs`
3. 以管理员身份运行

## 示例工作流

### 完整示例: 创建 Rust 项目

```
用户: 帮我创建一个简单的 Rust CLI 项目

AI: 我会为你创建项目结构。

create file `src/main.rs`

```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <name>", args[0]);
        return;
    }
    
    println!("Hello, {}!", args[1]);
}
```

系统:
⏳ 等待确认: 创建文件 src/main.rs

▶ 确认 (Confirm)
  取消 (Cancel)

用户: 按 Enter 确认

系统:
✅ 文件已创建: src/main.rs

用户: cargo run -- World
输出: Hello, World!
```

## 获取帮助

- 查看完整文档: `AI_CODE_MODIFICATION_WORKFLOW.md`
- 查看命令列表: `/help`
- 查看当前状态: `/status`

## 下一步

1. 尝试让 AI 创建一个简单的文件
2. 审查 Diff 对比
3. 确认修改
4. 验证文件内容
5. 逐步增加复杂度

祝您使用愉快！🚀
