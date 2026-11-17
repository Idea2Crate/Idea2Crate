# Examples

| 场景 | 入口／源码 | 用途 |
| --- | --- | --- |
| 从原生 App 开始 | [完整 Web 模板](../../../templates/egui-web-app/) | 构造函数接入、HTML、资源构建、生命周期和 App logic |
| 最小观察接入 | [Quick Start](../docs/quick-start.md#2-给已有-web-应用加观察) | 在现有 Web 创建回调安装并保留桥接，不改业务 UI |
| 可操作、无重置的应用 | `/sandbox/?observe` | 与工作台相同的真实界面，未注册重置处理器 |
| 可重复的完整工作台 | `/sandbox/`、[basic.rs](basic.rs) | Web 入口、桥接生命周期和外层重置适配 |
| 不依赖 sandbox 的业务界面 | [basic/workbench.rs](basic/workbench.rs) | 标准窗口、表单、同名操作、文本编辑和节点画布 |
| 多窗口展示 | 工作台的 Inspector 复选框 | 原有 `show_viewport_deferred` 自动使用 egui 的嵌入回退 |
| Agent 并行回归 | [parallel.mjs](../browser-tests/parallel.mjs) | 独立页面、语义查询、标准输入、重置和页面重载 |

在 Idea2Crate 仓库内按 [Quick Start](../docs/quick-start.md#1-先体验独立-demo)
构建并服务独立示例。示例不使用原生服务或系统剪贴板。

建议按顺序阅读：业务界面 → Web 入口 → 浏览器测试。注意业务界面没有调用
`Sandbox`、`Desktop`、窗口登记或观察接口。测试只通过公开观察协议读取状态，
然后发送正常浏览器输入。
