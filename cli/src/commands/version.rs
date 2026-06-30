// chplus version / -V / --version
// 输出工具链版本 (取自 Cargo.toml 的 package.version)
pub fn version(_args: &[String]) -> anyhow::Result<()> {
    println!("chplus {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
