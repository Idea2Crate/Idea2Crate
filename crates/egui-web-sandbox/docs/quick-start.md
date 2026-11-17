# Quick Start

目标：保留已有 egui 界面、布局和业务状态，在浏览器里让人或 Agent 操作同一套 Rust UI。
目前支持 egui 0.37，每个页面一个画布／桥接，每个并行测试使用独立 BrowserContext。

## 1. 先体验独立 Demo

在 Idea2Crate 仓库根目录，先安装固定 Rust 工具链和 WASM target：

```sh
rustup show
python3 -c 'import tomllib; print(next(p["version"] for p in tomllib.load(open("Cargo.lock", "rb"))["package"] if p["name"] == "wasm-bindgen"))'
```

用打印的版本安装 `wasm-bindgen-cli`：`cargo install wasm-bindgen-cli --version VERSION --locked`。
然后构建并服务独立工作台：

```sh
cargo build --locked -p egui-web-sandbox --example basic --features web --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/examples/basic.wasm --target web --no-typescript --out-dir .build/site/sandbox
cp crates/egui-web-sandbox/examples/index.html .build/site/sandbox/index.html
python3 -m http.server 8765 --bind 127.0.0.1 --directory .build/site
```

打开 <http://127.0.0.1:8765/sandbox/>。只构建通用工作台，不构建任何消费应用，
不需要系统字体或截屏权限。开发工具需要 Python 3.11+ 和 Node.js 24。

工作台有两个文档窗口、表单、文本编辑、节点画布和 Inspector 嵌入视口。
加 `?observe` 切换成只观察模式：界面仍可正常操作，但不提供场景重置。

浏览器回归命令见 [README](../README.md#independent-demo-and-acceptance)。

## 2. 给已有 Web 应用加观察

前提是应用已经能在 WASM 中运行。原生专属依赖需要应用自己的测试数据／适配器，
sandbox 不会自动把文件系统、网络任务或原生句柄替换成模拟实现。

尚未发布到 crates.io，使用 Idea2Crate 的固定提交 Git 依赖。将下面的占位值替换为
已审阅的完整提交 SHA：

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
egui-web-sandbox = { version = "0.1.0", git = "https://github.com/Idea2Crate/Idea2Crate", rev = "REVIEWED_COMMIT_SHA", features = ["web"] }
```

与应用统一 egui 的版本和来源；两个不同版本的 `egui::Context` 不能互传。
本库默认使用 crates.io 的 egui 0.37。消费项目如使用固定的上游补丁，在自己的
`[patch.crates-io]` 中统一来源，不修改本库来绑定某个应用的补丁。

只需要在现有 Web 创建入口安装一次，并由已有应用或 Web 生命周期对象保留桥接。
以下示例选择在应用中增加一个仅 WASM 使用的字段，原来的 `App::ui`、`logic`、
`save` 等实现都保持原样：

```rust,ignore
struct MyApp {
    // 原有业务字段……
    #[cfg(target_arch = "wasm32")]
    sandbox_bridge: Option<egui_web_sandbox::web::AgentBridge>,
}
```

原有构造函数把该字段初始化为 `None`。在 `WebRunner::start` 原有创建回调中：

```rust,ignore
Box::new(|cc| {
    let mut app = MyApp::new(cc); // 原有构造逻辑
    let sandbox = egui_web_sandbox::Sandbox::install(&cc.egui_ctx);
    app.sandbox_bridge = Some(
        egui_web_sandbox::web::AgentBridge::install(&cc.egui_ctx, &sandbox)
            .map_err(|error| format!("sandbox bridge: {error:?}"))?,
    );
    Ok(Box::new(app))
})
```

这就是不带重置的最小接入。重复 `Sandbox::install` 返回同一观察环境；无需逐帧调用、
登记窗口、改成 `Area`，或保存另一份业务状态。不要为了测试重写已有 `egui::Window`。
桥接按页面存活，释放它会删除 JS 入口和浏览器事件监听器。

## 3. 让 Agent 操作

使用浏览器自动化打开页面，导入 [browser/client.mjs](../browser/client.mjs)。
客户端不依赖特定应用，也不自行启动浏览器；传入已有 Playwright Page：

```js
const context = await browser.newContext();
const page = await context.newPage();
await page.goto(demoUrl);
const app = new SandboxPage(page);
await app.ready();
await app.control('TextInput', 'Title', {window: 'Project brief'}).fill('Next release');
await app.control('Button', 'Save', {window: 'Project brief'}).click();
const state = await app.control('CheckBox', 'Reviewed', {window: 'Project brief'}).read();
console.assert(state.checked === false);
```

重复名称通过所属窗口区分；客户端每次动作重新定位，歧义、禁用、视口外目标会明确报错。
操作发送正常浏览器鼠标／键盘输入，观察桥接没有点击、选择工具或修改业务数据的方法。
可用 `page.keyboard.press('ControlOrMeta+c')` 等标准快捷键测试文本复制／剪切／粘贴，
内容只保存在该实例内存中，不需要真实系统剪贴板或相关权限。

## 4. 按需启用重置

只有需要可重复场景的应用才做这一步。在桥接安装前调用 `sandbox.enable_reset()`，
并保留 `Sandbox` 句柄。在 Web 适配器每次绘制前消费请求：

```rust,ignore
if sandbox.take_reset_request() {
    // 应用负责取消旧任务、恢复测试数据和所需的 egui 编辑／布局状态。
    app.restore_fixture();
    sandbox.reset(); // 清空测试环境并确认新的 generation。
}
```

不要在普通业务控件中添加测试分支。工作台通过一个外层适配器替换为默认应用，
具体可看 [examples/basic.rs](../examples/basic.rs)。该示例应用只实现 `App::ui`；
已有项目若使用外层包装，需要转发其实际覆写的生命周期方法，或直接在已有入口接线。
没有启用处理器时，JS 不暴露 `reset()`，也不承诺能通用重置应用状态。

## 5. 自绘和多窗口

标准文本、按钮、表单和窗口自动观察。有名字的标准控件不需要重复标注。
图标按钮或自绘节点缺少名称时，优先使用 egui 原有语义：

```rust,ignore
let label = ui.label("Title");
ui.text_edit_singleline(&mut title).labelled_by(label.id);

let node = ui.interact(rect, id, egui::Sense::click_and_drag());
node.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "Input node"));
```

`widget_info` 回调中只构造信息，不重新读取 `Context`，避免锁重入。
自绘节点仍用正常 Response 处理拖动，sandbox 不接管它的业务行为。
普通多窗口直接沿用 egui 的嵌入视口回退；仅特殊原生窗口模拟才需要 `desktop::Desktop`。

## 已知范围

- 观察只描述实际提交的 egui UI；不保证系统窗口合成、遮挡及原生行为与浏览器一致。
- 画布默认无 CSS 变换、无边框；指针坐标是画布 CSS 原点加 egui 坐标乘 `zoom_factor`。
- 无名称的自绘像素不能自动变成可理解的业务控件。
- 本地存储、异步任务和外部服务的隔离由应用提供；每个 Agent 使用独立上下文。
- 当前验证了通用工作台和一个应用适配器，尚未声称兼容所有成熟 egui 项目。
