// chplus new <名称> / chplus init
// 脚手架:创建项目结构 + 入口文件 + chplus.toml
use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

const CHPLUS_TOML: &str = r#"# CH+ 项目配置
[package]
name = "{name}"
version = "0.1.0"
entry = "主函数.ch"

[dependencies]
"#;

// 模板已预先格式化,符合底层 -a 的输出 (括号前空格,无尾换行),
// 这样 `chplus fmt --check` 在新项目上直接通过
const MAIN_CH: &str = "定义 (空类型) 主函数() {\n    控制台输出 (\"你好,世界!\");\n}";

const GITIGNORE: &str = r#"dist/
*.chex
"#;

/// chplus new <名称>  — 在当前目录下创建子目录
pub fn create(args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("用法: chplus new <项目名称>"));
    }
    let name = &args[0];
    let target = Path::new(name);
    if target.exists() {
        return Err(anyhow!("目录已存在: {}", name));
    }

    fs::create_dir_all(target.join("src"))?;
    fs::create_dir_all(target.join("tests"))?;
    fs::write(target.join("主函数.ch"), MAIN_CH)?;
    fs::write(
        target.join("chplus.toml"),
        CHPLUS_TOML.replace("{name}", name),
    )?;
    fs::write(target.join(".gitignore"), GITIGNORE)?;

    println!("✓ 已创建项目: {}", name);
    println!("  cd {} && chplus run", name);
    Ok(())
}

/// chplus init — 在当前空目录初始化项目
pub fn init(_args: &[String]) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let name = cwd
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("chplus-project")
        .to_string();

    if Path::new("chplus.toml").exists() {
        return Err(anyhow!("当前目录已是 CH+ 项目 (chplus.toml 已存在)"));
    }

    fs::create_dir_all("src")?;
    fs::create_dir_all("tests")?;
    if !Path::new("主函数.ch").exists() {
        fs::write("主函数.ch", MAIN_CH)?;
    }
    fs::write("chplus.toml", CHPLUS_TOML.replace("{name}", &name))?;
    fs::write(".gitignore", GITIGNORE)?;

    println!("✓ 已在当前目录初始化 CH+ 项目: {}", name);
    println!("  chplus run");
    Ok(())
}
