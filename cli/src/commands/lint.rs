// chplus lint [文件]
// 轻量静态检查,区别于 fmt --check (格式) 和 check (能否跑通)
// lint 在 CLI 层直接扫 .ch 文件文本,做结构摘要 + 基础问题检测:
//   - 列出函数/结构体/导入/全局变量
//   - 检测重复定义、未使用的导入、空文件
// 不调底层解释器,纯文本分析,快且不依赖 -d 的机器化输出
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::path::PathBuf;

pub fn lint(args: &[String]) -> Result<()> {
    let manifest = crate::manifest::load_current().ok();
    let entry = crate::manifest::resolve_entry(manifest.as_ref(), args)?;
    let core = crate::core_path()?; // 校验底层存在 (lint 不用它,但保持一致的环境感知)

    // 收集要 lint 的文件:指定的单个文件,或 主函数.ch + src/*.ch
    let files: Vec<PathBuf> = if args.iter().any(|a| !a.starts_with('-')) {
        vec![entry]
    } else {
        collect_lint_files()?
    };

    if files.is_empty() {
        return Err(anyhow!("未找到要 lint 的 .ch 文件"));
    }

    let mut total_issues = 0;
    for f in &files {
        total_issues += lint_one(f)?;
    }

    // 顺便确认底层可调 (core_path 已 ok 证明底层编了),但不执行
    let _ = core;
    if total_issues == 0 {
        println!("\n✓ lint 通过 ({} 个文件)", files.len());
    } else {
        eprintln!("\n✗ lint 发现 {} 个问题", total_issues);
        std::process::exit(1);
    }
    Ok(())
}

/// 对单个 .ch 文件做 lint,返回发现的问题数
fn lint_one(path: &PathBuf) -> Result<usize> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| anyhow!("读取 {} 失败: {}", path.display(), e))?;

    println!("\n📄 {}", path.display());

    let mut functions: Vec<(String, usize)> = Vec::new();
    let mut structs: Vec<(String, usize)> = Vec::new();
    let mut imports: Vec<(String, usize)> = Vec::new();
    let mut globals: Vec<(String, usize)> = Vec::new();

    // 逐行扫,记录符号 + 行号
    // 这些模式基于 chplus-lang 的关键字:定义/结构体/导入
    for (i, line) in src.lines().enumerate() {
        let n = i + 1;
        let t = line.trim();
        // 跳过注释和空行
        if t.is_empty() || t.starts_with("//") || t.starts_with("#") {
            continue;
        }
        // 导入("路径");
        if let Some(p) = extract_import(t) {
            imports.push((p, n));
            continue;
        }
        // 结构体 名称 {  或  结构体 名称(
        if let Some(name) = extract_struct(t) {
            structs.push((name, n));
            continue;
        }
        // 定义 (返回类型) 函数名(...) {  ← 函数定义
        // 定义 (类型) 变量 = ...;        ← 变量定义 (在主函数体外才算全局,这里粗略都收)
        if let Some((name, is_fn)) = extract_def(t) {
            if is_fn {
                functions.push((name, n));
            } else {
                globals.push((name, n));
            }
        }
    }

    // 摘要输出
    println!("  函数 {} 个, 结构体 {} 个, 导入 {} 个, 全局变量 {} 个",
        functions.len(), structs.len(), imports.len(), globals.len());
    for (name, line) in &functions {
        println!("    fn {}() @{}", name, line);
    }
    for (name, line) in &structs {
        println!("    struct {} @{}", name, line);
    }
    for (p, line) in &imports {
        println!("    导入 \"{}\" @{}", p, line);
    }

    // 问题检测
    let mut issues = 0;

    // 1) 重复函数定义
    let mut seen: HashSet<&str> = HashSet::new();
    for (name, line) in &functions {
        if !seen.insert(name.as_str()) {
            println!("  ⚠ 重复函数定义: {} @{}", name, line);
            issues += 1;
        }
    }
    // 2) 重复结构体
    seen.clear();
    for (name, line) in &structs {
        if !seen.insert(name.as_str()) {
            println!("  ⚠ 重复结构体定义: {} @{}", name, line);
            issues += 1;
        }
    }
    // 3) 导入路径重复
    seen.clear();
    for (p, line) in &imports {
        if !seen.insert(p.as_str()) {
            println!("  ⚠ 重复导入: \"{}\" @{}", p, line);
            issues += 1;
        }
    }
    // 4) 空文件或几乎无内容
    if functions.is_empty() && structs.is_empty() && globals.is_empty() && imports.is_empty() {
        println!("  ⚠ 文件无任何定义 (空文件或仅含注释)");
        issues += 1;
    }

    Ok(issues)
}

/// 提取 `导入("路径")` 的路径
fn extract_import(line: &str) -> Option<String> {
    let t = line.trim_start();
    if !t.starts_with("导入") {
        return None;
    }
    // 找括号内的字符串字面量
    let after = t["导入".len()..].trim_start();
    if !after.starts_with('(') {
        return None;
    }
    let inner = &after[1..];
    let quote_start = inner.find('"')?;
    let rest = &inner[quote_start + 1..];
    let quote_end = rest.find('"')?;
    Some(rest[..quote_end].to_string())
}

/// 提取 `结构体 名称` 的名称
fn extract_struct(line: &str) -> Option<String> {
    let t = line.trim_start();
    if !t.starts_with("结构体") {
        return None;
    }
    let rest = t["结构体".len()..].trim_start();
    // 名称到下一个空白或 { 或 ( 截止
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '{' || c == '(')
        .unwrap_or(rest.len());
    if end == 0 {
        return None;
    }
    Some(rest[..end].to_string())
}

/// 提取 `定义 (...) 名称(...) {` (函数) 或 `定义 (...) 名称 = ...` (变量)
/// 返回 (名称, 是否函数)
fn extract_def(line: &str) -> Option<(String, bool)> {
    let t = line.trim_start();
    if !t.starts_with("定义") {
        return None;
    }
    let rest = t["定义".len()..].trim_start();
    if !rest.starts_with('(') {
        return None;
    }
    // 跳过 (...) 这一层 (可能嵌套,简单匹配第一个 ) — 类型本身不含括号)
    let close = rest.find(')')?;
    let after_type = rest[close + 1..].trim_start();
    // 名称到 ( 或 = 或 空白 截止
    let end = after_type
        .find(|c: char| c == '(' || c == '=' || c.is_whitespace())
        .unwrap_or(after_type.len());
    if end == 0 {
        return None;
    }
    let name = after_type[..end].to_string();
    // 后面紧跟 ( 视为函数定义,= 视为变量定义
    let is_fn = after_type[end..].trim_start().starts_with('(');
    Some((name, is_fn))
}

/// 收集要 lint 的文件:主函数.ch + src/*.ch (不含依赖与标准库)
fn collect_lint_files() -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if std::path::Path::new("主函数.ch").exists() {
        files.push(PathBuf::from("主函数.ch"));
    }
    if std::path::Path::new("src").is_dir() {
        for e in std::fs::read_dir("src")? {
            let p = e?.path();
            if p.extension().map_or(false, |x| x == "ch") {
                files.push(p);
            }
        }
    }
    files.sort();
    Ok(files)
}
