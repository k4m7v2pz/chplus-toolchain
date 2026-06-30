// chplus add <库名> <git url> [--tag/--branch/--rev/--path]
// 把依赖写进 chplus.toml 并拉到 chplus_modules/,同时更新 chplus.lock
//
// 形式:
//   chplus add 字符串工具 https://github.com/x/str.git --tag v0.1.0
//   chplus add 工具库 https://github.com/x/lib.git --branch main
//   chplus add 工具库 https://github.com/x/lib.git --rev abc123
//   chplus add 本地库 --path ../my-lib
use anyhow::{anyhow, Result};
use std::process::Command;

use crate::lock::{record_dep, dep_local_path, Lock};
use crate::manifest::{Dependency, Manifest};

pub fn add(args: &[String]) -> Result<()> {
    // 解析参数:至少要一个别名;第二个非 flag 参数是 git url 或(配合 --path)是路径
    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if positional.is_empty() {
        return Err(anyhow!(
            "用法:\n\
             \x20 chplus add <库名> <git url> [--tag <t> | --branch <b> | --rev <r>]\n\
             \x20 chplus add <库名> --path <本地路径>"
        ));
    }
    let alias = positional[0];

    let path_val = take_value(args, &["--path"]);
    let tag_val = take_value(args, &["--tag"]);
    let branch_val = take_value(args, &["--branch"]);
    let rev_val = take_value(args, &["--rev"]);

    // 构造依赖定义
    // --path 与 git url 互斥
    // 注意:--path 的值会被 positional 收集(不以 - 开头),需排除
    if path_val.is_some() {
        let has_git_url = positional.len() > 1
            && positional.get(1).map(|s| s.as_str()) != path_val.as_deref();
        if has_git_url {
            return Err(anyhow!("--path 与 git url 不能同时指定"));
        }
    }
    let dep = if let Some(p) = path_val {
        Dependency::Path { path: p }
    } else {
        // git 形式:url 优先取第二个位置参数
        let git = positional
            .get(1)
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("缺少 git url (用法: chplus add <库名> <git url> [--tag/--branch/--rev] 或 --path <本地路径>)"))?;
        // ref 三选一,都为空则拉默认分支 HEAD
        let ref_count = [tag_val.is_some(), branch_val.is_some(), rev_val.is_some()]
            .iter()
            .filter(|x| **x)
            .count();
        if ref_count > 1 {
            return Err(anyhow!("--tag/--branch/--rev 只能选一个"));
        }
        Dependency::Git {
            git,
            tag: tag_val,
            branch: branch_val,
            rev: rev_val,
        }
    };

    // 1) 落地依赖内容到 chplus_modules/<别名>
    let commit = materialize(alias, &dep)?;

    // 2) 把依赖写进 chplus.toml (若已存在则覆盖该条)
    write_dep_to_manifest(alias, &dep)?;

    // 3) 更新 chplus.lock
    let mut lock = Lock::load_current()?;
    record_dep(&mut lock, alias, &dep, commit.clone());
    lock.save_current()?;

    println!("✓ 已添加依赖: {}", alias);
    println!("  来源: {}", dep.source_url());
    if let Some(c) = commit {
        println!("  锁定 commit: {}", c);
    }
    match &dep {
        Dependency::Path { path } => println!("  本地路径: {}", path),
        _ => println!("  本地路径: {}", dep_local_path(alias).display()),
    }
    Ok(())
}

/// 把依赖拉到 chplus_modules/<别名>/
/// git 依赖:clone 到临时目录,checkout 到指定 ref,再移到目标位置(避免已存在时 git 报错)
/// path 依赖:不复制,直接记录路径(运行时按路径找)
/// 返回 git commit hash(path 依赖返回 None)
fn materialize(alias: &str, dep: &Dependency) -> Result<Option<String>> {
    let local = dep_local_path(alias);

    match dep {
        Dependency::Path { .. } => {
            // path 依赖不复制,只校验路径存在
            if let Dependency::Path { path } = dep {
                if !std::path::Path::new(path).exists() {
                    return Err(anyhow!("本地依赖路径不存在: {}", path));
                }
            }
            Ok(None)
        }
        Dependency::Git { git, tag, branch, rev } => {
            // 已存在则先删,保证幂等(再次 add 同一库可更新)
            if local.exists() {
                std::fs::remove_dir_all(&local)
                    .map_err(|e| anyhow!("清理旧依赖目录失败 {}: {}", local.display(), e))?;
            }
            std::fs::create_dir_all("chplus_modules")
                .map_err(|e| anyhow!("创建 chplus_modules/ 失败: {}", e))?;

            // clone 到目标目录
            let mut cmd = Command::new("git");
            cmd.arg("clone").arg("--depth").arg("50").arg(git).arg(&local);
            // 浅克隆 + 指定 ref:tag/branch 用 --branch;rev 必须先 clone 再 checkout
            if let Some(t) = tag {
                cmd.arg("--branch").arg(t);
            } else if let Some(b) = branch {
                cmd.arg("--branch").arg(b);
            }
            run_git(&mut cmd)?;

            // rev 形式:克隆后 checkout 到具体 commit
            if let Some(r) = rev {
                run_git(Command::new("git").arg("fetch").arg("--depth").arg("1").arg(git).arg(r).current_dir(&local))?;
                run_git(Command::new("git").arg("checkout").arg(r).current_dir(&local))?;
            }

            // 取当前 HEAD commit hash
            let out = Command::new("git")
                .arg("rev-parse").arg("HEAD")
                .current_dir(&local)
                .output()
                .map_err(|e| anyhow!("git rev-parse 失败: {}", e))?;
            if !out.status.success() {
                return Err(anyhow!("获取 commit hash 失败"));
            }
            let hash = String::from_utf8_lossy(&out.stdout).trim().to_string();
            // 删 .git 省空间,依赖不需要 git 历史
            let _ = std::fs::remove_dir_all(local.join(".git"));
            Ok(Some(hash))
        }
        Dependency::Version(v) => Err(anyhow!(
            "纯版本号依赖 '{}' 暂不支持 (无包注册表);请用 git 或 path 形式",
            v
        )),
    }
}

fn run_git(cmd: &mut Command) -> Result<()> {
    let out = cmd.output().map_err(|e| anyhow!("启动 git 失败: {}", e))?;
    if !out.status.success() {
        return Err(anyhow!(
            "git 操作失败 (exit {}): {}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(())
}

/// 从 args 里取 `--flag value` 的 value
fn take_value(args: &[String], flags: &[&str]) -> Option<String> {
    for (i, a) in args.iter().enumerate() {
        for f in flags {
            if a == *f {
                return args.get(i + 1).cloned();
            }
        }
    }
    None
}

/// 把依赖追加到 chplus.toml 的 [dependencies] 段 (已存在则覆盖该条)
/// 策略:整文件重读 → 改 deps 段 → 整文件写回,保留其他段注释
/// 用 toml 编辑会丢注释,这里手写追加最简单可靠
fn write_dep_to_manifest(alias: &str, dep: &Dependency) -> Result<()> {
    let path = std::path::Path::new("chplus.toml");
    if !path.exists() {
        return Err(anyhow!("未找到 chplus.toml,请先在项目目录运行"));
    }
    let original = std::fs::read_to_string(path)?;
    let dep_line = format_dep_line(alias, dep);

    // 简单策略:删掉已有 alias 行,在 [dependencies] 段末尾追加新行
    // 中文别名 TOML 必须加引号,所以匹配两种形态:别名 = 和 "别名" =
    let bare = format!("{} =", alias);
    let quoted = format!("\"{}\" =", alias);
    let bare2 = format!("{}=", alias);
    let quoted2 = format!("\"{}\"=", alias);
    let cleaned: Vec<&str> = original
        .lines()
        .filter(|line| {
            let t = line.trim_start();
            !t.starts_with(&bare) && !t.starts_with(&bare2)
                && !t.starts_with(&quoted) && !t.starts_with(&quoted2)
        })
        .collect();

    // 找 [dependencies] 段位置;没有就在文件末尾补一个
    let dep_section_idx = cleaned
        .iter()
        .position(|l| l.trim() == "[dependencies]");

    let mut new_lines: Vec<String> = cleaned.iter().map(|s| s.to_string()).collect();
    match dep_section_idx {
        Some(idx) => {
            // 在 [dependencies] 之后、下一个 [section] 之前插入
            let mut insert_at = idx + 1;
            while insert_at < new_lines.len() && !new_lines[insert_at].trim_start().starts_with('[') {
                insert_at += 1;
            }
            new_lines.insert(insert_at, dep_line);
        }
        None => {
            if !new_lines.is_empty() && !new_lines.last().map_or(false, |s| s.is_empty()) {
                new_lines.push(String::new());
            }
            new_lines.push("[dependencies]".into());
            new_lines.push(dep_line);
        }
    }

    // 保留原文件是否以换行结尾
    let trailing_nl = original.ends_with('\n');
    let mut out = new_lines.join("\n");
    if trailing_nl {
        out.push('\n');
    }
    std::fs::write(path, out)?;
    Ok(())
}

/// 生成 toml 里一行依赖定义
/// 中文别名 TOML 不允许裸 key,统一加引号包裹
fn format_dep_line(alias: &str, dep: &Dependency) -> String {
    // 含非 ASCII 或特殊字符的别名必须加引号;简单判断:含非 ASCII 就加
    let needs_quote = alias.chars().any(|c| !c.is_ascii());
    let key = if needs_quote {
        format!("\"{}\"", alias)
    } else {
        alias.to_string()
    };
    match dep {
        Dependency::Git { git, tag, branch, rev } => {
            let mut parts = vec![format!("git = \"{}\"", git)];
            if let Some(t) = tag {
                parts.push(format!("tag = \"{}\"", t));
            }
            if let Some(b) = branch {
                parts.push(format!("branch = \"{}\"", b));
            }
            if let Some(r) = rev {
                parts.push(format!("rev = \"{}\"", r));
            }
            format!("{} = {{ {} }}", key, parts.join(", "))
        }
        Dependency::Path { path } => format!("{} = {{ path = \"{}\" }}", key, path),
        Dependency::Version(_) => unreachable!("materialize 已拦掉 Version"),
    }
}

/// 供 run/build 复用:确保所有 manifest 依赖都已拉到本地且与 lock 一致
/// 返回 (是否就绪, 缺失或漂移的依赖描述列表)
pub fn ensure_deps_ready(manifest: &Manifest) -> (bool, Vec<String>) {
    let lock = Lock::load_current().unwrap_or_default();
    let (consistent, drift) = lock.consistency_with(manifest);
    if !consistent {
        let msgs = drift
            .iter()
            .map(|a| format!("{} (未拉取或与 lock 不一致,请跑 `chplus add {} <url>` 或重新 add)", a, a))
            .collect();
        return (false, msgs);
    }
    // lock 一致,再确认本地目录都在(path 依赖跳过)
    let mut missing = Vec::new();
    for (alias, dep) in &manifest.dependencies {
        if dep.is_path() {
            continue;
        }
        if !dep_local_path(alias).exists() {
            missing.push(format!("{} (chplus_modules/{} 缺失,请跑 `chplus add {} <url>`)", alias, alias, alias));
        }
    }
    (missing.is_empty(), missing)
}
