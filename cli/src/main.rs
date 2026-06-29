// chplus-cli:CH+ 中文编程语言工具链入口
// cargo 风格 subcommand,转发给底层 C++ 解释器
use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;

mod commands;

const HELP: &str = r#"CH+ 中文编程语言工具链

用法:
    chplus <命令> [选项] [参数]

命令:
    new <名称>       创建新 CH+ 项目
    init             在当前目录初始化 CH+ 项目
    run [文件]       编译并运行 .ch 文件 (默认入口:主函数.ch)
    build [文件]     编译为 16进制 .chex 产物
    fmt [文件]       格式化代码 (默认:整个项目)
    fmt --check      仅检查格式,不修改 (CI 用)
    test             运行 tests/ 下所有 .ch 测试
    help             显示本帮助

示例:
    chplus new 我的项目
    cd 我的项目
    chplus run
    chplus build
    chplus test

环境:
    底层解释器由 build.rs 编译,路径在编译期注入。
    若未编译,run/build 会给出友好错误。
"#;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print!("{}", HELP);
        return Ok(());
    }
    let cmd = args[1].as_str();
    let rest = &args[2..];

    match cmd {
        "-h" | "--help" | "help" => {
            print!("{}", HELP);
            Ok(())
        }
        "new" => commands::new::create(rest),
        "init" => commands::new::init(rest),
        "run" => commands::run::run(rest),
        "build" => commands::build::build(rest),
        "fmt" => commands::fmt::fmt(rest),
        "test" => commands::test::test(rest),
        other => Err(anyhow!(
            "未知命令: {}\n用 `chplus help` 查看可用命令",
            other
        )),
    }
}

/// 找到底层 C++ 解释器二进制路径
/// build.rs 通过 env! 注入;若为空说明编译失败
pub(crate) fn core_path() -> Result<PathBuf> {
    let p = env!("CHPLUS_CORE_PATH");
    if p.is_empty() {
        return Err(anyhow!(
            "底层 CH+ 解释器未编译 (build.rs 未找到 g++/clang++)\n\
             请安装 C++17 编译器后重新 cargo build"
        ));
    }
    Ok(PathBuf::from(p))
}
