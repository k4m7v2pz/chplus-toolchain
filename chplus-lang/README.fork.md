# chplus-lang — CH+ 解释器源码 (fork 说明)

本目录是 CH+ 中文编程语言解释器的 C++ 源码,**从官方仓库 `abcdefgjh-li/chplus` 的 `v3.0.0` tag fork 而来**(2026-02-10,commit `ce03907`)。

## 为什么放在这里

官方仓库的 `main` 分支已不再包含解释器源码(只剩 `chplus_ide.py` 一个壳),源码仅保留在历史 tag 中。为保障 `chplus-toolchain` 的可编译性、可审计性与长期可控,将 v3.0.0 的源码直接纳入本仓库作为一等公民,由 `cli/build.rs` 在编译期调用 `g++`/`clang++` 自动构建,产物作为底层 `chplus_core` 二进制被 CLI wrapper 通过子进程调用。

> 原 `README.md` 是官方 v3.0.0 的文档,保留未动;本文件仅说明 fork 状态。

## 目录结构

```
chplus-lang/
├── main.cpp              主程序入口
├── CMakeLists.txt        CMake 构建(已修正:指向外层 6 个 cpp,不再用 src/ 子目录)
├── include/             头文件
│   ├── lexer.h          词法分析
│   ├── parser.h         语法分析 + AST
│   ├── interpreter.h    解释器
│   ├── asm.h            16进制汇编器/VM
│   ├── CodeFormatter.h
│   └── CHFormatter.h
├── src/                 主线源码
│   ├── lexer.cpp
│   ├── parser.cpp
│   ├── interpreter.cpp
│   ├── asm.cpp
│   ├── CodeFormatter.cpp
│   └── CHFormatter.cpp
├── ch_Lib/              标准库 (.ch 文件)
├── examples/            示例 .ch 代码
├── README.md            官方 v3.0.0 README(保留)
├── README.mk.md         官方 v3.0.0 README(旧版,保留)
└── LICENSE             AGPL v3.0 + Commons Clause 1.0
```

## 编译

本项目已通过 `cli/build.rs` 自动编译此 C++ 源码,无需手动操作。`cargo build` 会自动调用系统 `g++`/`clang++` 编译,产物输出到 `OUT_DIR/bin/chplus_core`,运行时由 CLI 的 `crate::core_path()` 读取注入的 `CHPLUS_CORE_PATH` 环境变量定位调用。

如需独立编译(调试语言本身时):

```bash
cd chplus-lang
g++ -std=c++17 -O2 -I include main.cpp src/lexer.cpp src/parser.cpp src/interpreter.cpp src/asm.cpp src/CodeFormatter.cpp src/CHFormatter.cpp -o chplus_core
./chplus_core examples/hello.ch
```

## 与上游的差异

仅以下文件相对 v3.0.0 有改动:

- `CMakeLists.txt` — 修正源文件路径(指向外层 6 个 cpp,排除 `src/` 下重复子目录 `lexer/`、`parser/`、`executor/`、`utils/`,这些是官方历史残留副本)
- 新增本说明文件

源码本身未改一字。

## 已知清理项(后续)

- 解释器默认会格式化覆盖源文件,IDE 端通过 `-n` 参数绕过;长期应在语言层移除该默认行为
- `-p` flag 在 v3.0.0 已移除但 `--help` 文档未更新
- `src/` 下的子目录副本未来可彻底删除
