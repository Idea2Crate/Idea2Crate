# Examples

| 场景 | 入口／源码 | 用途 |
| --- | --- | --- |
| 用 sandbox 命令运行 | [Web 应用示例](../../../examples/web-app/) | 项目只提供 WASM 入口和 App 构造函数 |
| 交付独立可执行文件 | [嵌入式示例](../../../examples/embedded-web-app/) | 将网页嵌入项目自己的可执行文件 |
| 最小观察接入 | [Quick Start](../docs/quick-start.md#5-已有-web-宿主与重置) | 在现有 Web 创建回调安装并保留桥接，不改业务 UI |
| 可操作、无重置的应用 | `/sandbox/?observe` | 与工作台相同的真实界面，未注册重置处理器 |
| 可重复的完整工作台 | `/sandbox/`、[basic.rs](basic.rs) | Web 入口、桥接生命周期和外层重置适配 |
| 不依赖 sandbox 的业务界面 | [basic/workbench.rs](basic/workbench.rs) | 标准窗口、表单、同名操作、文本编辑和节点画布 |
| 多窗口展示 | 工作台的 Inspector 复选框 | 原有 `show_viewport_deferred` 自动使用 egui 的嵌入回退 |
| Agent 并行回归 | [parallel.mjs](../browser-tests/parallel.mjs) | 独立页面、语义查询、标准输入、重置和页面重载 |

按 [Web 应用示例](../../../examples/web-app/)执行启动命令，打开程序打印的地址。
示例不使用原生界面或系统剪贴板。

建议按顺序阅读：业务界面 → Web 入口 → 浏览器测试。注意业务界面没有调用
`Sandbox`、`Desktop`、窗口登记或观察接口。测试只通过公开观察协议读取状态，
然后发送正常浏览器输入。
