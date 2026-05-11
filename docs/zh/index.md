---
layout: home

hero:
  name: "Ambi"
  text: "高自由 · 高可扩展 · 高性能 · 跨平台"
  tagline: 一个用 Rust 写的 AI Agent 框架，不挡你的路
  image:
    src: /logo.svg
    alt: Ambi
  actions:
    - theme: brand
      text: 快速开始
      link: /zh/guide/getting-started
    - theme: alt
      text: 查看 GitHub
      link: https://github.com/Maskviva/Ambi

features:
  - icon: 🎯
    title: 高自由
    details: 引擎、管道、解析器、格式化器——每一个环节都可以换。本地 llama.cpp、云 API、或者你自己写的后端，随意组合。
  - icon: 🧩
    title: 高可扩展
    details: 核心很薄，不挡路。所有能力都是 trait，实现你需要的，剩下的不管。未来的扩展库直接插上就行。
  - icon: ⚡
    title: 高性能
    details: Rust + Tokio 异步。Arc 共享蓝图，克隆零成本。工具并发执行。内存占用小。没有 GC。没有意外。
  - icon: 🌐
    title: 跨平台
    details: 一套代码。原生二进制跑在 Windows/Linux/macOS，或者编译成 WASM 跑在浏览器里。Ambi 在 Rust 能编译的地方都能跑。
---
