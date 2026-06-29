// build.rs for chplus-cli
// 编译期把底层 C++ 解释器路径注入到 env,运行时直接读取,免去查找开销
use std::env;
use std::path::PathBuf;

fn main() {
    // cli/ 在 src-tauri/cli/,chplus-lang/ 在 src-tauri/chplus-lang/
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let lang_dir = manifest_dir.join("..").join("chplus-lang");
    let include_dir = lang_dir.join("include");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bin_dir = out_dir.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("创建 bin 目录失败");

    let exe_name = if cfg!(target_os = "windows") {
        "chplus_core.exe"
    } else {
        "chplus_core"
    };
    let exe_path = bin_dir.join(exe_name);

    // 选编译器
    let compiler = which("g++").or_else(|| which("clang++"));
    match compiler {
        None => {
            println!("cargo:warning=未找到 g++/clang++,底层解释器未编译");
            println!("cargo:warning=CLI 能运行但无法执行 .ch 文件");
            println!("cargo:rustc-env=CHPLUS_CORE_PATH=");
        }
        Some(cc) => {
            let sources = [
                "main.cpp",
                "src/lexer.cpp",
                "src/parser.cpp",
                "src/interpreter.cpp",
                "src/asm.cpp",
                "src/CodeFormatter.cpp",
                "src/CHFormatter.cpp",
            ];
            let mut cmd = std::process::Command::new(&cc);
            cmd.current_dir(&lang_dir)
                .arg("-std=c++17").arg("-O2")
                .arg("-I").arg(&include_dir)
                .args(&sources)
                .arg("-o").arg(&exe_path);
            if cfg!(target_os = "windows") {
                cmd.args(&["-lws2_32", "-lshlwapi"]);
            }
            let status = cmd.status().unwrap_or_else(|e| {
                panic!("启动 C++ 编译器失败 {}: {}", cc.display(), e);
            });
            if !status.success() {
                panic!("底层 CH+ 解释器 C++ 编译失败");
            }
            // 把产物路径重命名以避开与本 CLI 二进制同名冲突
            // (本 CLI 叫 chplus,底层叫 chplus_core,Runner 转发时用 chplus_core)
            println!("cargo:rustc-env=CHPLUS_CORE_PATH={}", exe_path.display());
            for src in &sources {
                println!("cargo:rerun-if-changed={}", lang_dir.join(src).display());
            }
            println!("cargo:rerun-if-changed={}", include_dir.display());
        }
    }
}

fn which(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
        if cfg!(target_os = "windows") {
            let candidate = dir.join(format!("{}.exe", name));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}
