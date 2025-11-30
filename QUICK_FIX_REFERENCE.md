# 🔧 编译错误修复快速参考

## 问题 1: 多重可变借用错误 (E0499)

### 症状
```
error[E0499]: cannot borrow `*analysis` as mutable more than once at a time
```

### 原因
在同一作用域内同时获取多个可变引用。

### ❌ 错误做法
```rust
let a = obj["a"].as_array_mut().unwrap();
let b = obj["b"].as_array_mut().unwrap();
// 错误：同时持有两个可变引用
```

### ✅ 正确做法 - 方案 1: 嵌套 if let
```rust
if let Some(a) = obj["a"].as_array_mut() {
    if let Some(b) = obj["b"].as_array_mut() {
        // 正确：每个引用的作用域被限制
    }
}
```

### ✅ 正确做法 - 方案 2: 在 match 分支中获取
```rust
match condition {
    Case1 => {
        if let Some(a) = obj["a"].as_array_mut() {
            // 使用 a
        }
    }
    Case2 => {
        if let Some(b) = obj["b"].as_array_mut() {
            // 使用 b
        }
    }
}
```

---

## 问题 2: 未使用变量警告

### 症状
```
warning: unused variable: `call`
```

### ❌ 错误做法
```rust
fn execute(&self, call: ToolCall) -> Result {
    // call 未使用
}
```

### ✅ 正确做法
```rust
fn execute(&self, _call: ToolCall) -> Result {
    // 使用下划线前缀表示故意不使用
}
```

---

## 问题 3: 值移动后借用错误

### 症状
```
error[E0382]: use of moved value: `content`
```

### ❌ 错误做法
```rust
match fs::write(&path, content) {
    Ok(_) => {
        // content 已被移动到 fs::write
        let len = content.len();  // 错误！
    }
}
```

### ✅ 正确做法
```rust
let len = content.len();  // 先计算
match fs::write(&path, content) {
    Ok(_) => {
        // 使用之前计算的 len
    }
}
```

---

## 问题 4: Result 未解包错误

### 症状
```
error[E0599]: no method named `path` found for enum `Result`
```

### ❌ 错误做法
```rust
for entry in fs::read_dir(path)? {
    entry.path()  // 错误：entry 是 Result
}
```

### ✅ 正确做法
```rust
for entry in fs::read_dir(path)? {
    let entry = entry?;  // 解包 Result
    entry.path()  // 正确
}
```

---

## 问题 5: Async/Await 缺失

### 症状
```
error[E0308]: mismatched types
expected `ToolResult`, found `impl Future`
```

### ❌ 错误做法
```rust
match async_function(&path) {  // 返回 Future
    Ok(result) => { ... }
}
```

### ✅ 正确做法
```rust
match async_function(&path).await {  // await Future
    Ok(result) => { ... }
}
```

---

## 快速检查清单

在修复编译错误时，检查以下几点：

- [ ] 是否同时持有多个可变引用？→ 使用嵌套作用域
- [ ] 是否有未使用的变量？→ 添加下划线前缀
- [ ] 是否在值被移动后使用它？→ 提前计算或克隆
- [ ] 是否忘记解包 Result？→ 添加 `?` 或 `match`
- [ ] 是否忘记 await？→ 添加 `.await`

---

## 相关资源

- [Rust Book - Ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
- [Rust Book - References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
- [Rust Book - Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
- [Rust Error Codes](https://doc.rust-lang.org/error-index.html)

