// chplus.toml 配置 schema + 解析
// 统一给 run/build/add/check/lint 等命令复用,避免每个命令各读一遍 toml
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// 整个 chplus.toml 的根结构
#[derive(Debug, Deserialize, Default)]
pub struct Manifest {
    pub package: Package,
    #[serde(default)]
    pub profile: BTreeMap<String, Profile>,
    /// 用 BTreeMap 保证依赖顺序稳定 (写回 lock 时输出顺序可预测)
    #[serde(default)]
    pub dependencies: BTreeMap<String, Dependency>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Package {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_entry")]
    pub entry: String,
}

fn default_version() -> String {
    "0.1.0".into()
}
fn default_entry() -> String {
    "主函数.ch".into()
}

/// profile 段:目前 dev/release 行为相同,框架先立起来
/// 后续可挂接不同的底层 flag (如 -O 级别)
#[derive(Debug, Deserialize, Default, Clone)]
pub struct Profile {
    #[serde(default)]
    pub opt_level: u8,
    #[serde(default)]
    pub debug: bool,
}

/// 依赖三种形式:
///   库名 = "1.2.3"                      (暂不支持 registry,先占位)
///   库名 = { git = "url", tag = "v1" }   (git + tag)
///   库名 = { git = "url", branch = "x" } (git + branch)
///   库名 = { git = "url", rev = "hash" } (git + 具体 commit)
///   库名 = { path = "../本地库" }        (本地路径)
/// 不写 tag/branch/rev 默认拉 default branch HEAD
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Git {
        git: String,
        #[serde(default)]
        tag: Option<String>,
        #[serde(default)]
        branch: Option<String>,
        #[serde(default)]
        rev: Option<String>,
    },
    Path {
        path: String,
    },
    /// 纯字符串形式:保留作占位,目前只在 add 时被替换为 git/path,
    /// 直接读到会报错让用户改成结构形式
    Version(String),
}

impl Dependency {
    /// 依赖落地的本地目录名 (chplus_modules/<此值>)
    pub fn local_name(alias: &str) -> String {
        alias.to_string()
    }
    pub fn source_url(&self) -> String {
        match self {
            Dependency::Git { git, .. } => git.clone(),
            Dependency::Path { path } => format!("path:{}", path),
            Dependency::Version(v) => format!("version:{}", v),
        }
    }
    /// ref 字符串,用于 git checkout / lock 记录
    pub fn git_ref(&self) -> Option<String> {
        match self {
            Dependency::Git {
                tag,
                branch,
                rev,
                ..
            } => {
                if let Some(t) = tag {
                    Some(format!("tags/{}", t))
                } else if let Some(b) = branch {
                    Some(format!("refs/heads/{}", b))
                } else if let Some(r) = rev {
                    Some(r.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn is_path(&self) -> bool {
        matches!(self, Dependency::Path { .. })
    }
}

/// 在指定目录加载并解析 chplus.toml
pub fn load(root: &Path) -> Result<Manifest> {
    let path = root.join("chplus.toml");
    if !path.exists() {
        return Err(anyhow!(
            "未找到 chplus.toml\n\
             用 `chplus new <名称>` 创建项目,或 `chplus init` 在当前目录初始化"
        ));
    }
    let s = std::fs::read_to_string(&path)
        .map_err(|e| anyhow!("读取 {} 失败: {}", path.display(), e))?;
    toml::from_str(&s).map_err(|e| anyhow!("解析 chplus.toml 失败: {}", e))
}

/// 当前目录加载 (常用入口)
pub fn load_current() -> Result<Manifest> {
    load(&PathBuf::from("."))
}

/// 解析入口文件路径:命令行显式 > toml 的 entry > 主函数.ch
pub fn resolve_entry(manifest: Option<&Manifest>, args: &[String]) -> Result<PathBuf> {
    if let Some(p) = args.first() {
        if !p.starts_with('-') {
            return Ok(PathBuf::from(p));
        }
    }
    if let Some(m) = manifest {
        return Ok(PathBuf::from(&m.package.entry));
    }
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
