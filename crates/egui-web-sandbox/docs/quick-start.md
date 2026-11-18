# Quick Start

Sandbox 提供 `egui-web-sandbox` 命令，自动编译用户项目的 WASM 库并启动本地 Web 服务。项目侧只需声明 WASM 库，并提供调用 App 构造函数的 Web 入口。

当前支持 egui/eframe 0.37。项目负责让共享 UI 可编译到 WASM，并适配截图、文件系统、全局快捷键等原生能力。

## 1. 运行完整示例

在 Idea2Crate 仓库根目录：

```sh
cargo run --locked -p egui-web-sandbox-build --features cli --bin egui-web-sandbox -- examples/web-app/Cargo.toml
```

第一次构建需要工具链声明的 `wasm32-unknown-unknown` target。命令打印类似
`http://127.0.0.1:49152/` 的地址，打开即可操作 Notes 应用。每次运行使用空闲端口，
可同时启动多个进程；每个 BrowserContext／页面还有独立的 App 和内存剪贴板。

默认监听本机回环地址 `127.0.0.1`。Ctrl+C 结束服务。

示例只有 Cargo 配置、普通 egui 界面和 WASM 入口，没有 `main.rs` 或 `build.rs`。
见 [examples/web-app](../../../examples/web-app/)。

## 2. 在自己的项目接入

依赖通过同一个已审核提交固定；当前尚未发布到 crates.io。将下面的
`REVIEWED_COMMIT_SHA` 替换为实际完整提交。

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[target.'cfg(target_arch = "wasm32")'.dependencies]
egui-web-sandbox = { version = "0.1.0", git = "https://github.com/Idea2Crate/Idea2Crate", rev = "REVIEWED_COMMIT_SHA", features = ["launcher"] }
eframe = { version = "0.37", default-features = false, features = ["glow", "default_fonts"] }
wasm-bindgen = "=0.2.130"
wasm-bindgen-futures = "0.4"
```

在库的 WASM 入口提供 App 构造函数：

```rust,ignore
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn start() -> Result<egui_web_sandbox::web::WebApp, wasm_bindgen::JsValue> {
    egui_web_sandbox::web::launch(Box::new(|cc| {
        Ok(Box::new(MyApp::new(cc)))
    })).await
}
```

安装一次启动命令，再在应用 package 目录运行：

```sh
rustup target add wasm32-unknown-unknown
cargo install --locked --git https://github.com/Idea2Crate/Idea2Crate --rev REVIEWED_COMMIT_SHA --features cli egui-web-sandbox-build
egui-web-sandbox
```

从 workspace 根目录运行时，传入成员的 manifest，例如
`egui-web-sandbox preview/Cargo.toml`。命令按默认 features 编译 WASM 库，并沿用 Cargo 的
锁文件和构建缓存。生成绑定要求项目的 wasm-bindgen 版本与命令一致，本示例固定为 0.2.130。
原生服务器已经在安装命令时编译好，可以服务多个项目；
项目本身无需为浏览器模式再构建原生 binary。服务只提供生成的网页和绑定资源。

需要固定端口或指定监听地址时，可以加上位置端口或 `--port PORT`，
以及 `-b ADDRESS` 或 `--bind ADDRESS`。

```sh
egui-web-sandbox preview/Cargo.toml 8000 --bind ::1
```

这个命令打印 `http://[::1]:8000/`。端口 `0` 让系统分配空闲端口。
显式指定 `0.0.0.0` 或 `::` 会监听相应的所有网络接口。
用 `egui-web-sandbox --help` 查看参数。

## 3. 接入现有项目时的配置

- 普通 App 和业务逻辑放在可共享的模块或 library，入口只替换构造函数。
- egui/eframe 必须保持兼容版本和来源。本库默认使用 crates.io 0.37；
  项目使用 Git 补丁时，在自己的 workspace 中统一 `[patch.crates-io]`。
- 原生专属依赖按 target 限制，Web 入口提供浏览器实现或生成的测试数据。

如果要交付一个无需 Cargo 和源码、双击或运行就能自己提供网页的可执行文件，
使用[嵌入式示例](../../../examples/embedded-web-app/)的构建和服务器 API。

| 应用内容 | 项目负责 |
| --- | --- |
| 标准表单、窗口、文本编辑、自绘画布 | 复用 Rust UI，自绘控件补充可访问名称 |
| 文件系统、截图、系统剪贴板、全局快捷键 | 浏览器能力或内存测试替身 |
| 本地存储、网络请求、异步任务 | 数据隔离、任务取消和可选重置 |
| 自定义原生窗口命令 | 按需使用 `desktop::Desktop` 模拟 |

## 4. Agent 用正常浏览器输入操作

从服务取得同一版本的客户端，例如先将 `<服务地址>/client.mjs` 下载到测试工具目录，
或直接引用已安装的 sandbox 的 `browser/client.mjs`。客户端不要求消费项目有兄弟源码仓库。

```js
import {SandboxPage} from './client.mjs';

const context = await browser.newContext();
const page = await context.newPage();
await page.goto(demoUrl);
const app = new SandboxPage(page);
await app.ready();
await app.control('TextInput', 'Title').fill('Next idea');
await app.control('Button', 'Save').click();
```

每个 Agent 使用独立 BrowserContext 和页面。同名控件使用
`{window: 'Project brief'}` 限定范围。输入通过浏览器鼠标和键盘发送，
观察桥接没有业务数据修改方法；未启用的重置接口不会出现。

标准文本复制／剪切／粘贴使用实例内存，不读取系统剪贴板。协议 v1 使用 egui 点；
Canvas 不应有 CSS 变换或边框。`ready` 表示 egui 已产生输出，业务异步状态须另行等待。
浏览器测试不证明原生窗口管理器或系统权限行为。

## 5. 已有 Web 宿主与重置

已有页面可以继续使用 `web::launch(AppCreator)` 或自有 Canvas 的
`web::launch_on(canvas, options, AppCreator)`，无需启用原生 `server`。
已有 WebRunner 则只启用 `web`，自行安装并保留 `Sandbox` 与 `AgentBridge`。

启动器将 App 直接交给 eframe，保留 `logic`、输入处理、绘制和其他生命周期方法。
页面 shell 处理加载提示、启动错误和卸载；重复启动会报错并保留已有应用。
`WebApp` 由 `mount()` 保留，停止时释放渲染器和桥接。

可选重置由项目处理：在构造函数内取得 `Sandbox::install(&cc.egui_ctx)`，
先调用 `enable_reset()`，然后在外层适配器消费 `take_reset_request()`，
恢复应用和任务后调用 `sandbox.reset()`。
[工作台入口](../examples/basic.rs) 展示这一模式，业务界面不导入 sandbox。

图标或自绘控件使用 egui 的名称、`labelled_by` 和 `widget_info` 补充语义。
`widget_info` 回调只构造信息，避免重新读取 Context 导致锁重入。
