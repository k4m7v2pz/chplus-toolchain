// chplus.lock — 锁定依赖具体 commit hash,保证可复现构建
// 格式与 Cargo.lock 类似但更简单:TOML,逐条记录别名/url/ref/commit
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::manifest::{Dependency, Manifest};

pub const LOCK_FILE: &str = "chplus.lock";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Lock {
    /// lock 文件格式版本,以后改动 schema 用
    pub version: u32,
    /// alias → 锁定信息
    pub package: BTreeMap<String, LockedDep>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockedDep {
    /// 来源 url 或 path:xxx
    pub source: String,
    /// 锁定的具体 commit hash (path 依赖为空)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// 当初指定的 ref (tag/branch/rev),用于 add 后溯源
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
}

impl Lock {
    /// 从当前目录读 lock;不存在返回空 Lock (首次 add 前)
    pub fn load_current() -> Result<Lock> {
        Self::load(Path::new(LOCK_FILE))
    }

    pub fn load(p: &Path) -> Result<Lock> {
        if !p.exists() {
            return Ok(Lock {
                version: 1,
                package: BTreeMap::new(),
            });
        }
        let s = std::fs::read_to_string(p)
            .map_err(|e| anyhow!("读取 {} 失败: {}", p.display(), e))?;
        let mut lock: Lock = toml::from_str(&s)
            .map_err(|e| anyhow!("解析 {} 失败: {}", p.display(), e))?;
        if lock.version == 0 {
            lock.version = 1;
        }
        Ok(lock)
    }

    pub fn save_current(&self) -> Result<()> {
        self.save(Path::new(LOCK_FILE))
    }

    pub fn save(&self, p: &Path) -> Result<()> {
        let s = toml::to_string_pretty(self)
            .map_err(|e| anyhow!("序列化 lock 失败: {}", e))?;
        std::fs::write(p, s).map_err(|e| anyhow!("写入 {} 失败: {}", p.display(), e))?;
        Ok(())
    }

    /// lock 是否与 manifest 一致:每个 manifest 依赖都在 lock 里且 source 相同
    /// 返回 (是否一致, 缺失或对不上的依赖别名列表)
    pub fn consistency_with(&self, manifest: &Manifest) -> (bool, Vec<String>) {
        let mut drift = Vec::new();
        for (alias, dep) in &manifest.dependencies {
            let source = dep.source_url();
            match self.package.get(alias) {
                None => drift.push(alias.clone()),
                Some(locked) if locked.source != source => drift.push(alias.clone()),
                // git 依赖必须有 commit 才算锁定;path 依赖 commit 可空
                Some(locked) if !dep.is_path() && locked.commit.is_none() => {
                    drift.push(alias.clone())
                }
                _ => {}
            }
        }
        (drift.is_empty(), drift)
    }
}

/// 把单个依赖的锁定信息写回 lock (add / 更新时用)
pub fn record_dep(lock: &mut Lock, alias: &str, dep: &Dependency, commit: Option<String>) {
    let entry = LockedDep {
        source: dep.source_url(),
        commit,
        r#ref: dep.git_ref(),
    };
    lock.package.insert(alias.to_string(), entry);
}

/// 找一个依赖在 chplus_modules/ 下的本地路径
pub fn dep_local_path(alias: &str) -> PathBuf {
    PathBuf::from("chplus_modules").join(alias)
}
