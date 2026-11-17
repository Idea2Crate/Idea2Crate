# Quick Start

把已有的 `eframe::App` 构造函数交给启动器，即可使用同一套 Rust UI 运行浏览器 Demo
和 Agent 测试。当前支持 egui/eframe 0.37；应用的原生能力适配由项目负责。

## 1. 从完整接入模板开始

[Web App 模板](../../../templates/egui-web-app/) 包含可运行的应用、Web 入口、HTML、
固定工具链和构建脚本。先在 Idea2Crate 仓库中体验：

```sh
rustup show
python3 templates/egui-web-app/build.py --locked
python3 -m http.server 8765 --bind 127.0.0.1 --directory templates/egui-web-app/.build/site
```

打开 <http://127.0.0.1:8765/>。开发需要 Python 3.11+；构建器会提示与 Cargo.lock
匹配的 wasm-bindgen CLI 安装命令。浏览器自动化另需 Node.js 24 和 Playwright。

模板的 `src/app.rs` 是普通 Notes 应用，既使用 `App::ui`，也使用 `App::logic`。
[Web 入口](../../../templates/egui-web-app/src/lib.rs) 只提供构造函数：

```rust,ignore
use egui_web_sandbox::web::{WebApp, launch};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn start() -> Result<WebApp, JsValue> {
    launch(Box::new(|cc| Ok(Box::new(app::Notes::new(cc))))).await
}
```

启动器负责默认 `#canvas`、eframe WebRunner、观察桥接及其生命周期。
应用直接交给 eframe，`logic`、`raw_input_hook`、`save`、绘制等方法按框架原有流程执行。

## 2. 接入现有原生 egui 项目

复制模板目录作为项目的 Web 入口，然后做以下配置：

- 将普通 App 和可共享业务逻辑放在原生、Web 入口都能引用的模块或库中。
- Web 入口依赖应用库，将模板构造函数替换成自己的构造函数。
- 将模板中的本仓库 path 依赖改为固定提交的 Git 依赖；当前未发布到 crates.io。
- 把原生专属依赖限制在对应 target，在 Web 构造函数中提供浏览器实现或测试数据。

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
egui-web-sandbox = { version = "0.1.0", git = "https://github.com/Idea2Crate/Idea2Crate", rev = "REVIEWED_COMMIT_SHA", features = ["launcher"] }
```

egui/eframe 必须与应用保持兼容版本和来源。本库默认使用 crates.io 的 0.37；
使用上游 Git 补丁的项目在自己的 workspace 中用 `[patch.crates-io]` 统一来源。

| 应用内容 | 接入工作 |
| --- | --- |
| 标准表单、窗口、文本编辑、自绘画布 | 复用现有 Rust UI；自绘控件补充可访问名称 |
| 文件系统、截图、系统剪贴板、全局快捷键 | 项目提供浏览器能力或内存测试替身 |
| 本地存储、网络请求、异步任务 | 项目管理测试数据、隔离、取消和重置 |
| 自定义原生窗口命令 | 按需使用 `desktop::Desktop` 模拟 |

完整应用示例见 [工作台入口](../examples/basic.rs) 与
[不导入 sandbox 的业务界面](../examples/basic/workbench.rs)。

## 3. 构建与资源分发

在复制的模板目录运行：

```sh
python3 build.py
python3 -m http.server 8765 --bind 127.0.0.1 --directory .build/site
```

构建脚本从 Cargo metadata 获取目标、输出目录、sandbox 来源及 wasm-bindgen 版本，
生成 `app.js`/`app_bg.wasm`，并从同一 sandbox 版本取得 `launch.mjs` 和 `client.mjs`。
消费项目无需单独下载 JS 客户端，也不依赖 Idea2Crate 的兄弟目录。
提交 Cargo.lock 后，CI 使用 `python3 build.py --locked`；`--out-dir` 可指定生成目录。

HTML 模板用以下代码启动应用：

```js
import {mount} from './launch.mjs';
const stop = await mount(() => import('./app.js'));
// 在自己的页面路由卸载时调用 stop()。
```

`mount` 负责加载提示、初始化错误及页面最终离开时的释放；`stop()` 可重复调用。
返回的 Rust `WebApp` 由它保留。自行接入 JS 时须保留 `WebApp`，卸载时调用
`free()`；`destroy()` 可提前停止渲染和桥接。启动失败会释放本次创建的资源，
重复启动会报错并保留已有应用。

`mount` 完成代表启动注册成功；测试仍应等待观察就绪及实际业务条件。

## 4. Agent 通过正常浏览器输入操作

从生成目录导入客户端，传入已有的 Playwright Page：

```js
import {SandboxPage} from './.build/site/client.mjs';

const context = await browser.newContext();
const page = await context.newPage();
await page.goto(demoUrl);
const app = new SandboxPage(page);
await app.ready();
await app.control('TextInput', 'Title').fill('Next idea');
await app.control('Button', 'Save').click();
```

每个并行 Agent 使用独立 BrowserContext 和页面。同名控件用
`{window: 'Project brief'}` 指定窗口。客户端每次动作重新定位，禁用、歧义和视口外
目标会明确报错；操作使用浏览器鼠标／键盘，观察桥接没有业务数据修改方法。

标准文本复制／剪切／粘贴使用该实例的内存剪贴板，无需系统剪贴板权限。
协议 v1 的边界使用 egui 点；指针映射假定 Canvas 没有 CSS 变换或边框。
`ready` 表示 egui 已产生输出，业务异步任务和真实原生窗口行为需项目另行验证。

## 5. 可选配置和已有 Web 宿主

- 默认 `web::launch(AppCreator)` 使用模板的 `#canvas` 和默认 WebOptions。
- 自有 Canvas、渲染配置使用 `web::launch_on(canvas, options, AppCreator)`。
- 已有 WebRunner 的项目可继续只启用 `web` 特性，安装 `Sandbox` 和
  `AgentBridge`，不引入启动器。桥接由原有宿主保留并随其释放。

启动器在构造 App 前安装观察器，在构造完成后发布浏览器桥接。
需要重置时，可在构造函数内用 `Sandbox::install(&cc.egui_ctx)` 取得同一实例，
调用 `enable_reset()`，并在项目适配器中处理请求：

```rust,ignore
if sandbox.take_reset_request() {
    app.restore_fixture();
    sandbox.reset();
}
```

项目负责旧任务取消、业务状态及必要的 egui 布局／编辑状态恢复。
未启用时 JS 不暴露 `reset()`。[完整工作台](../examples/basic.rs) 展示可重置模式；
`?observe` 使用同一界面而不提供重置。

标准控件和 egui 窗口自动观察。图标按钮、自绘对象优先用 egui 语义补充名称：

```rust,ignore
let label = ui.label("Title");
ui.text_edit_singleline(&mut title).labelled_by(label.id);
let node = ui.interact(rect, id, egui::Sense::click_and_drag());
node.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "Input node"));
```

`widget_info` 回调只构造信息，避免重新读取 Context 导致锁重入。
普通多窗口沿用 egui 的嵌入视口；浏览器实例隔离不等同于操作系统安全沙箱。
