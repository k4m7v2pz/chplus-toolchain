// chplus run [文件]
// 默认跑 chplus.toml 里 entry 指定的入口,无 toml 时跑 主函数.ch
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;

pub fn run(args: &[String]) -> Result<()> {
    let entry = resolve_entry(args)?;
    let core = crate::core_path()?;

    // -n 禁止格式化覆盖源文件
    let out = Command::new(&core)
        .arg("-n")
        .arg(&entry)
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

/// 解析要跑的 .ch 文件:
///   1. 命令行显式指定 → 用之
///   2. 否则读 chplus.toml 的 entry 字段
///   3. 否则回退到 主函数.ch
fn resolve_entry(args: &[String]) -> Result<PathBuf> {
    if let Some(p) = args.first() {
        if !p.starts_with('-') {
            return Ok(PathBuf::from(p));
        }
    }
    if let Ok(toml_str) = std::fs::read_to_string("chplus.toml") {
        if let Ok(doc) = toml::from_str::<toml::Value>(&toml_str) {
            if let Some(entry) = doc
                .get("package")
                .and_then(|p| p.get("entry"))
                .and_then(|v| v.as_str())
            {
                return Ok(PathBuf::from(entry));
            }
        }
    }
    // 兜底:主函数.ch
    if PathBuf::from("主函数.ch").exists() {
        return Ok(PathBuf::from("主函数.ch"));
    }
    Err(anyhow!(
        "未找到入口文件\n\
         请指定: chplus run <文件.ch>\n\
         或创建 chplus.toml 配置 entry 字段\n\
         或新建 主函数.ch"
    ))
}
