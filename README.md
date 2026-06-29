# CH+ 工具链 — chplus-toolchain

中文编程语言 **CH+** 的命令行工具链,cargo 风格。提供项目脚手架、运行、编译、格式化、测试命令,内置 C++ 解释器(fork 自官方 v3.0.0,AGPL-3.0)。

> 本仓库只做 shell 工具链,**不含图形界面**。IDE/编辑器是另一件事,后续以 VSCode 插件等形式独立解决。

## 安装

需要系统已安装 C++17 编译器(`g++` 或 `clang++`)。

```bash
git clone git@github.com:k4m7v2pz/chplus-toolchain.git
cd chplus-toolchain/cli
cargo install --path .
```

装好后 `chplus` 命令可用。

## 命令

```bash
chplus new <项目名>      # 创建新项目 (主函数.ch + chplus.toml + src/ + tests/)
chplus init              # 在当前目录初始化项目
chplus run [文件]        # 运行入口文件 (默认 chplus.toml 的 entry 或 主函数.ch)
chplus build [文件]      # 编译为 16进制 .chex 产物到 dist/
chplus fmt [--check]     # 格式化代码;--check 仅检查不改 (CI 用)
chplus test              # 跑 tests/*.ch,按 exit code 判通过/失败
chplus help              # 显示帮助
```

## 项目结构(chplus new 生成)

```
我的项目/
├── 主函数.ch           入口文件
├── chplus.toml         项目配置
├── src/                源码目录
├── tests/              测试目录
└── .gitignore
```

## 架构

```
chplus-toolchain/
├── chplus-lang/        C++ 解释器源码 (fork 自 abcdefgjh-li/chplus v3.0.0)
└── cli/                Rust wrapper (cargo 风格入口)
    └── src/commands/   new / run / build / fmt / test
```

CLI 是 wrapper,不修改 C++ 解释器源码,通过子进程调用。`cargo build` 时 `build.rs` 会自动编译底层 C++ 解释器,产物路径在编译期注入到 Rust 端。

## 许可证

AGPL-3.0 + Commons Clause 1.0,与 CH+ 官方一致。详见 `chplus-lang/LICENSE`。
