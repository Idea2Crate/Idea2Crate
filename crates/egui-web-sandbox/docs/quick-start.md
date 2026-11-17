# Quick Start

把 App 构造函数交给 Web 启动器，构建时自动编译并嵌入网页；原生可执行文件运行时直接提供本地 Web 服务。无需复制模板、编写 HTML、安装 wasm-bindgen CLI 或启动 Python 服务器。

当前支持 egui/eframe 0.37。项目负责让共享 UI 可编译到 WASM，并适配截图、文件系统、全局快捷键等原生能力。

## 1. 运行完整示例

在 Idea2Crate 仓库根目录：

```sh
cargo run --locked -p egui-web-app
```

第一次构建需要工具链声明的 `wasm32-unknown-unknown` target。命令打印类似
`http://127.0.0.1:49152/` 的地址，打开即可操作 Notes 应用。每次运行使用空闲端口，
可同时启动多个进程；每个 BrowserContext／页面还有独立的 App 和内存剪贴板。

也可以先构建，再单独运行产物：

```sh
cargo build --locked -p egui-web-app
./target/debug/egui-web-app
```

成品内嵌 WASM、JS、页面、客户端和许可声明。运行时不需要 Cargo、Python、源码目录
或外部资源文件。它只监听本机回环地址，不自动打开浏览器或访问系统剪贴板。
Ctrl+C 结束服务。

完整源码见 [examples/web-app](../../../examples/web-app/)。这是一份可运行示例，
接入项目不需要复制其目录。

## 2. 在自己的项目接入

依赖通过同一个已审核提交固定；当前尚未发布到 crates.io。将下面的
`REVIEWED_COMMIT_SHA` 替换为实际完整提交。

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[build-dependencies]
egui-web-sandbox-build = { version = "0.1.0", git = "https://github.com/Idea2Crate/Idea2Crate", rev = "REVIEWED_COMMIT_SHA" }

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
egui-web-sandbox = { version = "0.1.0", git = "https://github.com/Idea2Crate/Idea2Crate", rev = "REVIEWED_COMMIT_SHA", features = ["server"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
egui-web-sandbox = { version = "0.1.0", git = "https://github.com/Idea2Crate/Idea2Crate", rev = "REVIEWED_COMMIT_SHA", features = ["launcher"] }
eframe = { version = "0.37", default-features = false, features = ["glow", "default_fonts"] }
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
```

在已有 `build.rs` 中调用构建 API：

```rust,ignore
fn main() {
    egui_web_sandbox_build::build().expect("build embedded Web app");
}
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

原生入口调用服务器 API：

```rust,ignore
fn main() -> egui_web_sandbox::server::Result<()> {
    egui_web_sandbox::server::serve(egui_web_sandbox::include_app!())
}
```

然后直接 `cargo run`。项目原本已有桌面入口时，可以另加一个 Web 服务 binary，
原有桌面运行方式由项目保留。构建 API 会将本 package 的 library 编译到 WASM，
随后生成 JS 绑定并嵌入产物；WASM 子构建不会递归生成另一份网站。
原生服务 API 只提供这些内嵌资源，不暴露项目目录。

这不是运行时将原生二进制转换成 WASM：同一套 Rust UI 在构建时编译到两个目标。
首次构建会多进行一次 WASM 编译，后续复用 Cargo 增量缓存。UI 只存在于 WASM 的
示例中，原生部分只负责 Web 服务。

## 3. 接入现有项目时的配置

- 普通 App 和业务逻辑放在可共享的模块或 library，入口只替换构造函数。
- egui/eframe 必须保持兼容版本和来源。本库默认使用 crates.io 0.37；
  项目使用 Git 补丁时，在自己的 workspace 中统一 `[patch.crates-io]`。
- 原生专属依赖按 target 限制，Web 入口提供浏览器实现或生成的测试数据。
- 构建 API 默认使用 Web 目标的默认 Cargo features；需要不同组合时显式配置：

```rust,ignore
egui_web_sandbox_build::Builder::default()
    .no_default_features()
    .features(["web-demo"])
    .build()
    .expect("build embedded Web app");
```

构建器需要当前 package 有 cdylib library，沿用 Cargo.lock；生成的网页、嵌入代码及
嵌套构建产物均写入 Cargo OUT_DIR。
当前 package 源码变化使用 Cargo 默认的 package 跟踪；引用包外的共享源码或资源时，
应在 build.rs 声明对应的 `cargo:rerun-if-changed`，同时保留对本 package 输入的声明。
完整示例采用同一 package 内的源码，不需要额外声明。

服务器默认使用空闲端口。固定端口或集成到项目自身生命周期时，使用
`server::Server::bind(include_app!(), port)`，通过 `url()` 取得地址，
调用 `run()` 开始服务。服务阻塞当前线程，线程安排由调用方决定；
`stop()` 结束请求循环，等待服务线程结束后释放 Server 即关闭监听。

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
