// chplus build [文件]
// 编译为 .chex 二进制产物,输出到 dist/
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn build(args: &[String]) -> Result<()> {
    let entry = resolve_entry(args)?;
    let core = crate::core_path()?;

    std::fs::create_dir_all("dist")?;
    let stem = entry
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let out_path = PathBuf::from("dist").join(format!("{}.chex", stem));

    // chplus -x <输出> <输入>
    let out = Command::new(&core)
        .arg("-x")
        .arg(&out_path)
        .arg(&entry)
        .output()
        .map_err(|e| anyhow!("启动解释器失败: {}", e))?;

    use std::io::Write;
    std::io::stderr().write_all(&out.stderr).ok();
    if !out.status.success() {
        return Err(anyhow!(
            "编译失败 (exit {})",
            out.status.code().unwrap_or(-1)
        ));
    }
    println!("✓ 已编译: {} → {}", entry.display(), out_path.display());
    println!("  运行: chplus run {}", out_path.display());
    Ok(())
}

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
    if Path::new("主函数.ch").exists() {
        return Ok(PathBuf::from("主函数.ch"));
    }
    Err(anyhow!("未找到要编译的 .ch 文件"))
}
