# 命令提示调试指南

## 问题诊断

如果输入 `/` 后没有看到命令提示，请按以下步骤调试：

### 步骤 1: 编译验证
```bash
cargo build
```

检查是否有编译错误。如果有错误，修复后重新编译。

### 步骤 2: 运行应用
```bash
cargo run
```

### 步骤 3: 测试命令提示

1. **激活提示**
   - 在输入框中输入 `/`
   - 观察是否有提示面板出现

2. **检查过滤**
   - 继续输入 `h` (变成 `/h`)
   - 应该只显示 `/help` 命令

3. **测试导航**
   - 按 `↑` 或 `↓` 箭头
   - 应该看到高亮项改变

4. **测试自动完成**
   - 按 `Tab`
   - 输入框应该填充为 `/help`

5. **测试执行**
   - 按 `Enter`
   - 应该执行 `/help` 命令

## 常见问题

### 问题 1: 提示面板不显示

**可能原因：**
- 终端窗口太小
- 输入框高度不足
- 命令提示未被激活

**解决方案：**
1. 扩大终端窗口
2. 检查 `app.command_hints.visible` 是否为 `true`
3. 验证 `activate()` 方法是否被调用

### 问题 2: 提示面板显示但没有命令

**可能原因：**
- 过滤逻辑有问题
- 命令列表为空

**解决方案：**
1. 检查 `get_filtered_hints()` 返回的列表
2. 验证命令列表初始化是否正确

### 问题 3: 导航不工作

**可能原因：**
- 事件处理器未正确处理箭头键
- 命令提示不可见时处理了箭头键

**解决方案：**
1. 检查 `handle_chat_event()` 中的条件
2. 验证 `app.command_hints.visible` 的值

### 问题 4: 自动完成不工作

**可能原因：**
- Tab 键未被正确处理
- 没有选中的命令

**解决方案：**
1. 检查 `get_selected()` 是否返回有效的命令
2. 验证 Tab 键事件处理

## 调试技巧

### 1. 添加日志输出

在 `command_hints.rs` 中添加调试输出：

```rust
pub fn activate(&mut self, input: &str) {
    eprintln!("DEBUG: activate called with input: {}", input);
    if input.starts_with('/') {
        eprintln!("DEBUG: Command detected, activating hints");
        self.visible = true;
        self.filter = input[1..].to_lowercase();
        self.selected_index = 0;
    } else {
        eprintln!("DEBUG: Not a command, hiding hints");
        self.visible = false;
    }
}
```

### 2. 检查渲染条件

在 `mod.rs` 中添加调试输出：

```rust
if app.command_hints.visible && input_chunks[1].height > 3 {
    eprintln!("DEBUG: Rendering hints, height: {}", input_chunks[1].height);
    // ... render code
} else {
    eprintln!("DEBUG: Not rendering hints, visible: {}, height: {}", 
        app.command_hints.visible, input_chunks[1].height);
}
```

### 3. 验证事件处理

在 `handler.rs` 中添加调试输出：

```rust
if app.command_hints.visible {
    eprintln!("DEBUG: Command hints visible, handling navigation");
    match key_event.code {
        KeyCode::Up => {
            eprintln!("DEBUG: Up arrow pressed");
            app.command_hints.select_previous();
            return true;
        }
        // ... other cases
    }
}
```

## 完整测试流程

1. **启动应用**
   ```bash
   cargo run 2>&1 | tee debug.log
   ```

2. **输入测试序列**
   - 输入: `/`
   - 输入: `h`
   - 按: `↑`
   - 按: `Tab`
   - 按: `Enter`

3. **查看日志**
   ```bash
   cat debug.log | grep DEBUG
   ```

4. **分析结果**
   - 检查每个步骤的日志输出
   - 确认函数是否被调用
   - 验证条件是否满足

## 预期行为

### 输入 `/` 时
```
输入框: /|
提示面板:
┌─ 💡 命令提示 ─┐
│ /help - 显示帮助信息    │
│ /clear - 清除聊天历史   │
│ /status - 显示应用状态  │
│ ...                    │
└──────────────────┘
```

### 输入 `/h` 时
```
输入框: /h|
提示面板:
┌─ 💡 命令提示 ─┐
│ /help - 显示帮助信息    │
│ /history - 显示历史    │
└──────────────────┘
```

### 按 Tab 时
```
输入框: /help|
提示面板: (关闭)
```

## 性能检查

- 命令提示激活时间: < 1ms
- 过滤时间: < 1ms
- 渲染时间: < 5ms
- 导航响应: 即时

## 最后检查清单

- [ ] 编译无错误
- [ ] 应用启动正常
- [ ] 输入 `/` 显示提示
- [ ] 继续输入过滤命令
- [ ] 上下箭头导航工作
- [ ] Tab 自动完成工作
- [ ] Enter 执行命令
- [ ] Esc 关闭提示

如果所有项都通过，命令提示系统工作正常！
