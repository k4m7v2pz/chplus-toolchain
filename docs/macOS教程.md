# macOS 上玩 CH+ 中文编程 — 从零到跑通

面向没装过 Rust、没用过命令行编译的新手。按顺序走完,你就能在 Mac 上写中文代码跑游戏。

---

## 一、准备工作(一次性,装好就不用再装)

打开「终端」应用(Spotlight 搜 `终端` 或 `Terminal`),逐条复制粘贴执行。

### 1.1 装 Xcode 命令行工具(提供 g++ 编译器)

CH+ 的底层解释器是 C++ 写的,需要 `g++` 或 `clang++` 来编译。

```bash
xcode-select --install
```

弹窗点「安装」,等几分钟。装完验证:

```bash
g++ --version
```

能看到版本号(类似 `Apple clang version 15.x`)就 OK。

### 1.2 装 Rust(提供 cargo,用来装 CH+ 工具链)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

中途问选项,直接按回车选默认(`1`)。装完让它生效:

```bash
source "$HOME/.cargo/env"
```

验证:

```bash
cargo --version
```

能看到 `cargo 1.x.x` 就 OK。

### 1.3 装好 CH+ 工具链

```bash
git clone git@github.com:k4m7v2pz/chplus-toolchain.git
cd chplus-toolchain/cli
cargo install --path .
```

这一步会编译 Rust CLI + 编译底层 C++ 解释器,大约 1-3 分钟。

验证:

```bash
chplus version
```

输出 `chplus 0.1.0` 就装好了。现在 `chplus` 命令全局可用。

> 如果提示 `command not found`,执行 `source "$HOME/.cargo/env"` 再试。

---

## 二、玩现成游戏(最快上手)

挑一个游戏仓库 clone 下来直接跑。比如猜数字:

```bash
git clone git@github.com:k4m7v2pz/chplus-game-guess.git
cd chplus-game-guess
chplus run
```

程序会提示你猜 1-100 之间的数字,输入数字回车,根据"大了/小了"提示继续猜,猜中显示用了几次。

可选仓库(任选其一 clone 来玩):

| 仓库 | 游戏 | 难度 |
|---|---|---|
| chplus-game-guess | 猜数字 | 入门 |
| chplus-game-24point | 算 24 点 | 入门 |
| chplus-game-idiom | 成语接龙 | 入门 |
| chplus-game-8puzzle | 8 拼(数字推盘) | 入门 |
| chplus-game-sudoku | 数独 | 中级 |
| chplus-game-minesweeper | 扫雷 | 中级 |
| chplus-game-2048 | 2048 | 中级 |
| chplus-game-life | 生命游戏 | 中级 |
| chplus-game-gomoku | 五子棋 | 中级 |

clone 地址统一格式:`git@github.com:k4m7v2pz/<上面的名字>.git`

---

## 三、自己写一个 CH+ 项目(动手开始)

### 3.1 新建项目

```bash
cd ~                       # 回到主目录,你也可以 cd 到任何想放代码的地方
chplus new 我的第一个项目
cd 我的第一个项目
```

CH+ 会自动建好项目结构:

```
我的第一个项目/
├── 主函数.ch       ← 你写代码的地方
├── chplus.toml     ← 项目配置
├── src/            ← 源码目录(可放多文件)
├── tests/          ← 测试目录
└── .gitignore
```

### 3.2 打开源文件看一眼

```bash
cat 主函数.ch
```

输出:

```
定义 (空类型) 主函数() {
    控制台输出 ("你好,世界!");
}
```

这就是 CH+ 的中文代码。关键字全是中文:`定义`、`空类型`、`主函数`、`控制台输出`。

### 3.3 跑起来

```bash
chplus run
```

输出:

```
你好,世界!
```

恭喜,你已经跑通了第一个 CH+ 程序。

### 3.4 改改试试

用任何编辑器打开 `主函数.ch`(VS Code、Sublime、TextEdit 都行,UTF-8 编码)。改成:

```
定义 (空类型) 主函数() {
    控制台输出 ("你好,我是 张三!");
    控制台输出 ("今天学 CH+ 第一天");
}
```

再跑 `chplus run`,看输出变了。

---

## 四、CH+ 基础语法速查

### 变量

```
定义(整型) x = 10;
定义(小数) y = 3.14;
定义(字符串) s = "中文";
定义(布尔) ok = 真;
```

### 输入输出

```
控制台输出("提示文字");
定义(整型) n;
控制台输入(n);           // 阻塞等用户输入
```

### 条件

```
如果 (x > 0) {
    控制台输出("正数");
}否则如果 (x == 0) {
    控制台输出("零");
}否则{
    控制台输出("负数");
}
```

> ⚠ 必须用中文 `否则如果`,不能用 `else if`。

### 循环

```
// C 风格 for
对于 (i = 0; i < 5; i = i + 1) {
    控制台输出(i);
}

// while
定义(整型) n = 0;
当 (n < 10) {
    控制台输出(n);
    n = n + 1;
}
```

### 函数

```
定义(整型) 加(定义(整型) a, 定义(整型) b) {
    返回 a + b;
}

定义(空类型) 主函数() {
    定义(整型) r = 加(3, 4);
    控制台输出(r);        // 输出 7
}
```

> ⚠ 参数必须写 `定义(类型) 名字`,不能简写成 `类型 名字`。

### 数组

```
定义(整型) arr[5];
arr[0] = 10;
arr[1] = 20;
控制台输出(arr[0] + arr[1]);   // 输出 30
```

### 结构体

```
定义(结构体) 点 {
    整型 x;
    整型 y;
};

定义(空类型) 主函数() {
    定义(点) p;
    p.x = 100;
    p.y = 200;
    控制台输出("x=" + p.x + " y=" + p.y);
}
```

### 用标准库

```
导入("ch_Lib/math.ch");      // 数学函数:PI、sin、cos、log、pow

定义(空类型) 主函数() {
    定义(小数) r = 5.0;
    定义(小数) 面积 = PI * r * r;
    控制台输出("圆面积 = " + 面积);
}
```

---

## 五、CH+ 命令速查

在项目目录下:

| 命令 | 作用 |
|---|---|
| `chplus run [文件]` | 运行入口文件(默认 `主函数.ch`) |
| `chplus check [文件]` | 只检查语法不运行,CI 用 |
| `chplus build [文件]` | 编译为 `.chex` 产物到 `dist/` |
| `chplus fmt` | 格式化代码 |
| `chplus fmt --check` | 仅检查格式不改,CI 用 |
| `chplus lint` | 静态分析,列符号 + 检查重复定义 |
| `chplus test` | 跑 `tests/*.ch` 所有测试 |
| `chplus run --watch` | 改文件自动重跑,开发爽 |
| `chplus clean` | 清理 `dist/` 和 `.chex` |
| `chplus clean --all` | 连依赖一起删 |
| `chplus add <库名> <git url> --tag <v1.0>` | 加 git 依赖 |
| `chplus add <库名> --path <本地路径>` | 加本地依赖 |
| `chplus version` | 显示版本 |

---

## 六、一个完整小例子 — 简易计算器

新建项目后把 `主函数.ch` 改成:

```
定义(空类型) 主函数() {
    控制台输出("═══ 简易计算器 ═══");
    控制台输出("输入两个数,我帮你加起来");
    
    控制台输出("第一个数:");
    定义(整型) a;
    控制台输入(a);
    
    控制台输出("第二个数:");
    定义(整型) b;
    控制台输入(b);
    
    定义(整型) sum = a + b;
    控制台输出("和是: " + sum);
}
```

跑:

```bash
chplus run
```

输入 3 和 5,输出 `和是: 8`。

试着自己加:减法、乘法、除法、判断奇偶、循环输入多次。

---

## 七、常见问题

### Q1: `chplus run` 报"底层 CH+ 解释器未编译"

说明 `cargo install` 时没找到 `g++`/`clang++`。回到第一步装 Xcode 命令行工具,然后重新 `cargo install --path .`。

### Q2: `command not found: chplus`

Rust 的 PATH 没生效。执行 `source "$HOME/.cargo/env"`,或重开终端。如果想永久生效,把这行加到你的 `~/.zshrc`:

```bash
echo 'source "$HOME/.cargo/env"' >> ~/.zshrc
```

### Q3: 输出中文乱码

macOS 终端默认 UTF-8,通常不会有问题。如果真乱码,检查终端编码设置:终端 → 设置 → 描述文件 → 文本 → 字符编码设为 UTF-8。

### Q4: 我改了代码但 `chplus run` 还是输出旧结果

确认保存了文件。还在跑旧的就 `chplus clean` 清一下再 `chplus run`。

### Q5: 想清掉所有产物重新来

```bash
chplus clean --all      # 删 dist/、.chex、chplus_modules/
```

### Q6: 报错"语句必须以分号结束"但我看不出来哪缺分号

90% 是你用了英文关键字(如 `else if` 而非 `否则如果`)。CH+ 关键字必须中文。剩下 10% 是真漏分号 — 检查每条语句末尾。

### Q7: 报错"函数参数必须使用 '定义(类型) 名字' 语法"

参数写法不对。正确:`定义(整型) 加(定义(整型) a, 定义(整型) b)`,错误:`定义(整型) 加(整型 a, 整型 b)`。

### Q8: 我想看更多坑

读 [踩坑.md](./踩坑.md),把所有已知坑都列了。

---

## 八、下一步

1. **改改示例游戏** — clone 一个游戏仓库,改难度、改提示语、加新规则
2. **写自己的小程序** — 待办、计算器、猜单词
3. **读源码学** — 看 `主函数.ch` 怎么写的,中文关键字应该能看懂
4. **关注工具链** — [chplus-toolchain](https://github.com/k4m7v2pz/chplus-toolchain) 会持续更新

---

## 九、关于 CH+ 的立场

CH+ 反对收费中文编程(易语言、火山等),坚持开源、免费、跨平台、git 友好。
详见 [立场声明](https://github.com/k4m7v2pz/chplus-toolchain/blob/dev/STATEMENT.md)。

中文编程不该是门生意。

---

记录人:本轮 agent(2026-06-30)
