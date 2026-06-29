# AGENTS — 给 AI 助手的项目指引

## 项目定位

`chplus-toolchain` 是中文编程语言 CH+ 的**命令行工具链**。cargo 风格,内置 C++ 解释器。

**只做 shell 工具,不做图形界面。** IDE/编辑器是独立项目,不在本仓库范围内。

## 仓库结构

```
chplus-toolchain/
├── chplus-lang/        C++ 解释器 (fork 自 abcdefgjh-li/chplus v3.0.0)
│   ├── include/        头文件: lexer.h / parser.h / interpreter.h / asm.h
│   ├── src/            6 个主线源文件 (lexer/parser/interpreter/asm/CodeFormatter/CHFormatter)
│   ├── ch_Lib/         标准库 (.ch 文件)
│   ├── examples/       示例代码
│   ├── main.cpp        入口
│   └── CMakeLists.txt  已修正:指向外层 6 个 cpp,排除 src/ 下历史残留子目录
└── cli/                Rust 工具链 wrapper
    ├── build.rs        编译期调用 g++/clang++ 编译 chplus-lang,产物路径注入 env
    ├── Cargo.toml      crate 名 chplus
    └── src/
        ├── main.rs     命令分发
        └── commands/   new / run / build / fmt / test
```

## 关键设计

### Wrapper 模式
CLI 不碰 C++ 内部,通过子进程调用底层 `chplus_core` 二进制(由 build.rs 编译产物命名而来,避免与本 CLI 二进制 `chplus` 重名)。未来若有人用 Rust 重写解释器,wrapper 接口不变,只换底层。

### build.rs 自动编译
- `cli/build.rs` 在 `cargo build` 时调用 `g++`/`clang++` 编译 `../chplus-lang/` 下 6 个 cpp 文件
- 产物输出到 `OUT_DIR/bin/chplus_core`
- 路径通过 `cargo:rustc-env=CHPLUS_CORE_PATH=...` 注入到 Rust 编译期
- 运行时 `crate::core_path()` 读 `env!("CHPLUS_CORE_PATH")` 定位

### 已知底层坑(绕过而非修复)
1. **默认格式化覆盖源文件** → CLI 所有调用都加 `-n` 禁掉
2. **`-p` flag 已废弃** → 不用
3. **fmt 会在 `(` 前加空格、删尾换行** → `chplus new` 模板已预格式化迎合它
4. **fmt 时 stdout 有噪音** → CLI 调用时丢 stdout/stderr,只看文件差异

## 技术栈

| 层 | 技术 |
|---|---|
| 工具链入口 | Rust 2021 |
| 底层解释器 | C++17 (fork v3.0.0) |
| 配置格式 | TOML (chplus.toml) |
| 许可证 | AGPL-3.0 + Commons Clause 1.0 |

## 测试

```bash
cd cli && cargo build
./target/debug/chplus new /tmp/test-proj && cd /tmp/test-proj
chplus run           # 应输出 "你好,世界!"
chplus fmt --check   # 应通过
chplus build         # 应产出 dist/*.chex
chplus test          # 跑 tests/*.ch
```

## 命令规范

- 子命令英文 (`new/run/build/fmt/test/init`),保持与 cargo 一致的习惯
- 用户可见的提示信息用中文 (符合 CH+ 中文定位)
- 退出码:成功 0,失败非 0
