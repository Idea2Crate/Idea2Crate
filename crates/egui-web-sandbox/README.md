# egui-web-sandbox

Observe and test an existing egui application using normal browser mouse and
keyboard input. People and agents use the same Rust UI in isolated browser
contexts, without Computer Use or native services.

Maintained in [Idea2Crate/Idea2Crate](https://github.com/Idea2Crate/Idea2Crate).
The library has its own demo, browser client and regression tests. It has no
application or editor dependency and is not published to crates.io.

Start with the [Quick Start / 快速接入](docs/quick-start.md) and [Examples](examples/README.md).

## Who uses it, and why?

| Use case | What the sandbox adds | What the application keeps |
| --- | --- | --- |
| Share a public interactive demo | Inspectable UI, instance-local clipboard, optional reset | Existing Rust UI and generated/sample data |
| Parallel agent regression tests | Semantic observations and a browser client; a separate BrowserContext per worker | Business logic and ordinary UI event handling |
| Test forms, editors and custom canvases | Names, roles, values, checkbox/radio/slider/focus states and window scope | Standard widgets; accessible names for custom drawing |
| Demonstrate a multi-window tool | Automatic observation of egui windows, including embedded viewports | egui's own layout, dragging, resizing and viewport callbacks |
| Simulate special native windows | Optional `desktop::Desktop` for explicit custom drag/resize requests | Platform-specific command adapter and lock/modal policy |

The design has three responsibilities, within one crate:

1. **Observation:** discover the actual egui accessibility output automatically.
   No per-frame registration or business-state mirror is needed for standard UI.
2. **Test environment:** keep clipboard data in memory and coordinate an optional
   application reset. Fixtures, asynchronous services and native I/O remain app-owned.
3. **Optional virtual desktop:** translate custom native window commands only when
   egui's built-in windows/embedded viewports cannot represent the application's needs.

## Attach to an existing web app

Use the same egui **0.37** source/version as the application. In the existing web
creation callback, install an observer and retain its bridge alongside the app:

```rust,ignore
let sandbox = egui_web_sandbox::Sandbox::install(&cc.egui_ctx);
let bridge = egui_web_sandbox::web::AgentBridge::install(&cc.egui_ctx, &sandbox)?;
// Retain `bridge` in the web entry/outer adapter for the lifetime of the page.
```

`install` is idempotent per Context. It enables accessibility, derives viewport
bounds/window hierarchy automatically and does not replace the app's UI or memory.
No `begin_frame`, window ID allocation or manual rectangle storage is required.
Use existing egui `labelled_by`/`accessible_name` for unnamed widgets; custom drawing
should expose a normal egui Response with widget information. Actions still go
through the application's real pointer/keyboard handling.

The bridge redirects browser copy/cut/paste to the instance's in-memory clipboard,
including ordinary TextEdit shortcuts and clipboard events. It never reads or
writes the OS clipboard. Apps with custom image-paste commands read `sandbox.image()`
at their existing command adapter. Browser listeners are removed when the bridge drops.

For reset, call `enable_reset()` **before** installing the bridge. In an outer
adapter, consume `take_reset_request()`, restore the application's fixture/state,
then call `sandbox.reset()` to acknowledge and clear the environment. Reset does
not generically reconstruct app state, cancel app-owned jobs or clear egui memory.
Without an explicit handler, the bridge does not expose `reset()`.

## Agent observations and actions

Retain one bridge per page. `window.eguiSandbox` is frozen and exposes `version`,
`snapshot()` and, when supported, `reset()`. There are no business-mutation or
synthetic click methods. `snapshot()` returns a fresh observation; `ready` means
egui has produced output, not that a compositor has acknowledged a visible frame.

Use the supplied Playwright-compatible client with an already-open page:

```js
import {SandboxPage} from './browser/client.mjs';
const app = new SandboxPage(page);
await app.ready();
await app.control('TextInput', 'Title', {window: 'Project brief'}).fill('New title');
await app.control('Button', 'Save', {window: 'Project brief'}).click();
const reviewed = await app.control('CheckBox', 'Reviewed', {window: 'Project brief'}).read();
console.assert(reviewed.checked === false);
await app.reset(); // Only if this app registered a reset handler.
```

The client resolves each target from a fresh snapshot and sends real browser
input. Ambiguous names require window scope; disabled/outside-viewport targets fail
explicitly. It does not force clicks through overlays or infer virtualized/offscreen
content. Reveal or scroll content using the app's existing UI first.

Protocol v1 uses `[left, top, right, bottom]` rectangles in **egui points**.
For eframe web input, CSS coordinates are the canvas content origin plus the point
multiplied by `zoom_factor`; never multiply by device pixel ratio or infer scale
from the backing texture. The client assumes an untransformed, borderless canvas.
Window/widget IDs are opaque strings, not persistent selectors. Prefer role/name
and window scope. Roles are our own named protocol enum; unknown roles map to `Other`.
Controls include parent/window IDs, focused/disabled, checked/mixed, selected/expanded,
text/numeric values and numeric ranges when egui supplies them. TextRun glyph nodes
are omitted. Child relationships can include structural nodes without a control entry.

Additive fields are allowed; clients ignore unknown fields/roles. Breaking changes
require a protocol version change; Rust compatibility follows crate versions separately.
Observation structs are non-exhaustive and implementation state is private.

## Independent demo and acceptance

**Workspace Lab** is an ordinary document workbench: two windows with repeated
control names, standard form/text editing, a draggable custom canvas and an optional
embedded viewport inspector. `examples/basic/workbench.rs` has **no sandbox imports**.
Only `examples/basic.rs`, the browser entry/outer reset adapter, knows about the host.
`?observe` runs the same app without application reset support.
From the Idea2Crate repository root, with the wasm-bindgen CLI version recorded
in `Cargo.lock`:

```sh
rustup target add wasm32-unknown-unknown
cargo build --locked -p egui-web-sandbox --example basic --features web --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/examples/basic.wasm --target web --no-typescript --out-dir .build/site/sandbox
cp crates/egui-web-sandbox/examples/index.html .build/site/sandbox/index.html
python3 -m http.server 8765 --bind 127.0.0.1 --directory .build/site
```

Open `/sandbox/`. In `crates/egui-web-sandbox/browser-tests/`, run `npm ci --ignore-scripts`, `npx playwright install chromium --only-shell`,
then `npm test`. `SANDBOX_URL` changes the origin, `SANDBOX_DEMO_PATH` the demo path,
and `CHROMIUM_EXECUTABLE` optionally selects an existing headless Chromium binary.
The tests exercise real form/clipboard/canvas/window input, optional reset, reload
and independent concurrent contexts, including DPR 2. No application-specific tests
or dependencies are needed. A consuming application owns its integration tests.

The host currently targets egui 0.37 and one canvas/bridge per page. Each agent uses
a fresh BrowserContext and page; application persistence/network services must be
configured as fixtures by the consuming project. Native window-manager behavior
and OS I/O still require their own adapters and native acceptance. A small example
does not establish compatibility with every existing egui application.

## 中文

源码、独立 Demo 和浏览器回归由 Idea2Crate 仓库维护，通过固定提交的 Git 依赖接入，
暂未发布到 crates.io。浏览器上下文隔离浏览器状态，不代表构建命令、生成代码或
外部服务处于操作系统安全沙箱中。

面向应用交互 Demo 和 Agent 并行 UI 回归。通用接入只需在已有 Web 入口安装并保留
观察桥接，业务界面继续使用标准 egui，不登记每个窗口，也不镜像业务状态。
观察负责控件名称、状态与窗口归属；测试环境负责内存剪贴板及可选重置；虚拟桌面
仅供特殊原生窗口模拟使用。独立工作台业务代码不导入 sandbox，表单、文本、画布
和多窗口都走正常 egui 输入。需要先让原生专属服务可在 WASM 中使用测试数据替代。

MIT licensed. Original extraction copyright belongs to Snip2Pin contributors.
