// chplus check [文件]
// 语法/运行时检查:调底层 -n 跑一遍,丢 stdout,只在 stderr 非空或 exit 非 0 时报错
// 与 run 的区别:不转发 stdout (程序输出对 check 无意义),专注于是否报错
use anyhow::{anyhow, Result};
use std::process::Command;

pub fn check(args: &[String]) -> Result<()> {
    let manifest = crate::manifest::load_current().ok();
    // check 也校验依赖 (依赖缺失会导致语法错误,误报)
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

    let out = Command::new(&core)
        .arg("-n")
        .arg(&entry)
        .output()
        .map_err(|e| anyhow!("启动解释器失败: {}", e))?;

    let code = out.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&out.stderr);

    if code != 0 || !stderr.trim().is_empty() {
        // 报错时把 stderr 打出来,方便定位
        use std::io::Write;
        std::io::stderr().write_all(&out.stderr).ok();
        return Err(anyhow!("检查失败 (exit {}): {}", code, entry.display()));
    }

    println!("✓ {} 语法检查通过", entry.display());
    Ok(())
}
