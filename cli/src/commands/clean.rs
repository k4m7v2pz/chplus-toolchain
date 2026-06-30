// chplus clean
// 清理构建产物:dist/ 和所有 .chex 文件
// 不删 chplus_modules/ (那是依赖,删了得重新拉);要删依赖用 chplus clean --all
use anyhow::{anyhow, Result};
use std::path::Path;

pub fn clean(args: &[String]) -> Result<()> {
    let purge_all = args.iter().any(|a| a == "--all" || a == "-a");
    let mut removed: Vec<String> = Vec::new();

    // dist/ 目录整体删
    if Path::new("dist").is_dir() {
        std::fs::remove_dir_all("dist")
            .map_err(|e| anyhow!("删除 dist/ 失败: {}", e))?;
        removed.push("dist/".into());
    }

    // 项目根下的散落 .chex (不在 dist/ 里的)
    for entry in std::fs::read_dir(".").map_err(|e| anyhow!("读取当前目录失败: {}", e))? {
        let p = entry?.path();
        if p.extension().map_or(false, |e| e == "chex") {
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                std::fs::remove_file(&p).ok();
                removed.push(name.into());
            }
        }
    }

    // --all 时连依赖一起删
    if purge_all && Path::new("chplus_modules").is_dir() {
        std::fs::remove_dir_all("chplus_modules")
            .map_err(|e| anyhow!("删除 chplus_modules/ 失败: {}", e))?;
        removed.push("chplus_modules/".into());
    }

    if removed.is_empty() {
        println!("无需清理 (没有产物)");
    } else {
        println!("✓ 已清理:");
        for r in &removed {
            println!("  {}", r);
        }
    }
    Ok(())
}
