// chplus run [文件] [--watch]
// 默认跑 chplus.toml 里 entry 指定的入口,无 toml 时跑 主函数.ch
// --watch/-w:文件变动后自动重跑,直到 Ctrl+C
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

pub fn run(args: &[String]) -> Result<()> {
    let watch = args.iter().any(|a| a == "--watch" || a == "-w");
    let manifest = crate::manifest::load_current().ok();
    // 跑前校验依赖齐全 (无 manifest 时跳过,兼容裸 主函数.ch 项目)
    if let Some(m) = &manifest {
        let (ready, issues) = crate::commands::add::ensure_deps_ready(m);
        if !ready {
            let mut msg = String::from("依赖未就绪:\n");
            for i in &issues {
                msg.push_str(&format!("  - {}\n", i));
            }
            msg.push_str("修复后重试");
            return Err(anyhow!("{}", msg));
        }
    }
    let entry = crate::manifest::resolve_entry(manifest.as_ref(), args)?;
    let core = crate::core_path()?;

    if watch {
        run_watch(&core, &entry)
    } else {
        run_once(&core, &entry)
    }
}

/// 单次执行,转发输出,失败 exit 非 0
fn run_once(core: &PathBuf, entry: &PathBuf) -> Result<()> {
    // -n 禁止格式化覆盖源文件
    let out = Command::new(core)
        .arg("-n")
        .arg(entry)
        .output()
        .map_err(|e| anyhow!("启动解释器失败: {}", e))?;

    // 直接转发 stdout/stderr,保持原色
    use std::io::Write;
    std::io::stdout().write_all(&out.stdout).ok();
    std::io::stderr().write_all(&out.stderr).ok();
    let code = out.status.code().unwrap_or(-1);
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

/// watch 模式:监听项目内 .ch 文件变化,变动就重跑
/// 监听范围:当前目录 + src/ + chplus_modules/ (不含 dist/、target/、.git/)
/// 用 Debouncer 太重,直接轮询 poll:每秒扫一遍所有 .ch 的 mtime,变了就重跑
/// (notify crate 已加,但跨平台事件去抖麻烦,poll 对 CH+ 项目规模足够)
fn run_watch(core: &PathBuf, entry: &PathBuf) -> Result<()> {
    println!("👁 watch 模式已启动 (Ctrl+C 退出),监听 .ch 文件变动...");
    // 首次先跑一遍
    println!("\n{}", "─".repeat(40));
    run_once(core, entry)?;

    let mut last = snapshot_mtimes();
    loop {
        std::thread::sleep(Duration::from_secs(1));
        let now = snapshot_mtimes();
        if now != last {
            last = now;
            println!("\n{}\n⚡ 检测到文件变动,重跑...\n{}", "─".repeat(40), "─".repeat(40));
            // 重跑时把当前依赖状态再校验一次 (用户可能在 watch 期间 add 了依赖)
            if let Ok(m) = crate::manifest::load_current() {
                let (ready, issues) = crate::commands::add::ensure_deps_ready(&m);
                if !ready {
                    eprintln!("⚠ 依赖未就绪,跳过本次执行:");
                    for i in &issues {
                        eprintln!("  - {}", i);
                    }
                    continue;
                }
            }
            // 重跑错误不退出 watch,只报出来继续监听 (区别于单次模式的 exit)
            let out = match Command::new(core).arg("-n").arg(entry).output() {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("⚠ 启动解释器失败: {}", e);
                    continue;
                }
            };
            use std::io::Write;
            std::io::stdout().write_all(&out.stdout).ok();
            std::io::stderr().write_all(&out.stderr).ok();
            if !out.status.success() {
                eprintln!("⚠ 本次执行失败 (exit {}),继续监听...", out.status.code().unwrap_or(-1));
            }
        }
    }
}

/// 收集项目所有 .ch 文件的 (路径, mtime) 快照,用于 watch 比对
/// 扫 当前目录顶层 + src/ + chplus_modules/,跳过 dist/、target/、.git/
fn snapshot_mtimes() -> Vec<(PathBuf, std::time::SystemTime)> {
    let mut snap = Vec::new();
    let mut collect = |dir: &std::path::Path, recursive: bool| {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for e in entries.flatten() {
            let p = e.path();
            // 跳过 ignore 目录
            if p.is_dir() {
                if !recursive {
                    continue;
                }
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if matches!(name, "dist" | "target" | ".git" | "node_modules") {
                    continue;
                }
                // 递归子目录 — 用闭包不便,改成外层调用
                let sub = std::fs::read_dir(&p).ok();
                if let Some(sub) = sub {
                    for se in sub.flatten() {
                        let sp = se.path();
                        if sp.extension().map_or(false, |x| x == "ch") {
                            if let Ok(m) = std::fs::metadata(&sp).and_then(|m| m.modified()) {
                                snap.push((sp, m));
                            }
                        }
                    }
                }
            } else if p.extension().map_or(false, |x| x == "ch") {
                if let Ok(m) = std::fs::metadata(&p).and_then(|m| m.modified()) {
                    snap.push((p, m));
                }
            }
        }
    };

    collect(std::path::Path::new("."), false);
    collect(std::path::Path::new("src"), true);
    collect(std::path::Path::new("chplus_modules"), true);
    snap.sort_by(|a, b| a.0.cmp(&b.0));
    snap
}
