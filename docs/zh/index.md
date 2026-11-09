---
layout: home

hero:
  name: Ambi
  text: 小而可组合的 Rust AI Agent Runtime
  tagline: 组合能力，而不是继承层级。Ambi 提供执行机制，智能体行为由你定义。
  image:
    src: /logo.svg
    alt: Ambi 标志
  actions:
    - theme: brand
      text: 快速开始
      link: /zh/guide/getting-started
    - theme: alt
      text: 设计理念
      link: /zh/advanced/design-philosophy
    - theme: alt
      text: GitHub
      link: https://github.com/Maskviva/Ambi

features:
  - icon: 🧩
    title: 组合优于继承
    details: 自由替换 Engine、Pipeline、Tool、Parser 和 Formatter，不强制多 Agent 层级或工作流图。
  - icon: 🌳
    title: Agent · Pipeline · Leaf
    details: Agent 是复用能力的树，Pipeline 是执行策略的枝，Leaf 是按任务生成、用完即丢的叶。
  - icon: 🔧
    title: 小型 Runtime，开放扩展点
    details: 内置 ReAct 与可选 Pipeline、Memory、Macro 扩展；其他开发者可以决定能力的上限。
  - icon: ⚙️
    title: 云端、本地与自定义模型
    details: OpenAI 兼容 API、本地 llama.cpp 和自定义 Engine 均可以接入运行时。
---

## 先理解设计，再使用 API

Ambi **不规定所有 Agent 必须如何思考或协作**。它提供支撑不同实现所需要的基础执行机制。建议从 [设计理念](/zh/advanced/design-philosophy) 和 [架构](/zh/advanced/architecture) 开始。

**0.4.1 新增：** 可选的 [Leaf 任务委派](/zh/guide/leaf) 让 `ChatRunner` 运行一次性、上下文隔离的子任务，不创建长期存在的第二个 Agent。Leaf 仍继承原始工具并共享 Engine，**不是安全沙箱**。

接着阅读 [快速开始](/zh/guide/getting-started) 或 [自定义 Pipeline](/zh/advanced/custom-pipeline)。
