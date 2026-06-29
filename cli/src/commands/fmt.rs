// chplus fmt [文件] [--check]
// 格式化 .ch 代码,支持 --check 仅检查不改
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn fmt(args: &[String]) -> Result<()> {
    let check_only = args.iter().any(|a| a == "--check" || a == "-c");
    let target: Option<PathBuf> = args.iter().find(|a| !a.starts_with('-')).map(PathBuf::from);
    let core = crate::core_path()?;

    let files: Vec<PathBuf> = match target {
        Some(f) => vec![f],
        None => collect_project_ch_files()?,
    };

    if files.is_empty() {
        return Err(anyhow!("未找到 .ch 文件 (默认查找 当前目录/主函数.ch)"));
    }

    let mut changed = 0;
    for f in &files {
        let original = std::fs::read_to_string(f)?;
        if check_only {
            // 检查模式:格式化到临时文件,对比是否变化
            // stdout/stderr 全丢,只看文件内容差异 (底层会打印 "文件已自动格式化" 噪音)
            let tmp = std::env::temp_dir().join(format!(
                "chplus_fmt_{}.ch",
                f.file_name().and_then(|s| s.to_str()).unwrap_or("tmp")
            ));
            std::fs::write(&tmp, &original)?;
            let status = Command::new(&core)
                .arg("-a")
                .arg(&tmp)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()?;
            let formatted = std::fs::read_to_string(&tmp)?;
            std::fs::remove_file(&tmp)?;
            if !status.success() {
                return Err(anyhow!("格式化失败: {}", f.display()));
            }
            if formatted != original {
                println!("✗ 格式不一致: {}", f.display());
                changed += 1;
            } else {
                println!("✓ {}", f.display());
            }
        } else {
            // 实际格式化:直接调 -a 覆盖原文件,丢噪音
            let status = Command::new(&core)
                .arg("-a")
                .arg(f)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()?;
            if !status.success() {
                return Err(anyhow!("格式化失败: {}", f.display()));
            }
            println!("✓ 已格式化: {}", f.display());
        }
    }

    if check_only && changed > 0 {
        eprintln!("\n{} 个文件需要格式化,运行 `chplus fmt` 修复", changed);
        std::process::exit(1);
    }
    Ok(())
}

/// 默认收集项目内所有 .ch 文件 (主函数.ch + src/*.ch + ch_Lib/*.ch)
fn collect_project_ch_files() -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if Path::new("主函数.ch").exists() {
        files.push(PathBuf::from("主函数.ch"));
    }
    for dir in ["src", "ch_Lib"] {
        if Path::new(dir).exists() {
            for entry in std::fs::read_dir(dir)? {
                let p = entry?.path();
                if p.extension().map_or(false, |e| e == "ch") {
                    files.push(p);
                }
            }
        }
    }
    Ok(files)
}
