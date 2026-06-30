// chplus build [文件]
// 编译为 .chex 二进制产物,输出到 dist/
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;

pub fn build(args: &[String]) -> Result<()> {
    let manifest = crate::manifest::load_current().ok();
    // 编译前同样校验依赖 (保证可复现构建)
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
