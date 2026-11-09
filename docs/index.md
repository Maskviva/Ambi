---
layout: home

hero:
  name: Ambi
  text: A small, composable AI Agent Runtime for Rust
  tagline: Compose capabilities, not Agent hierarchies. Ambi provides execution mechanisms; you define the behavior.
  image:
    src: /logo.svg
    alt: Ambi logo
  actions:
    - theme: brand
      text: Get started
      link: /guide/getting-started
    - theme: alt
      text: Design philosophy
      link: /advanced/design-philosophy
    - theme: alt
      text: GitHub
      link: https://github.com/Maskviva/Ambi

features:
  - icon: 🧩
    title: Composition over inheritance
    details: Replace engines, pipelines, tools, parsers and formatters. No mandatory Agent class hierarchy or graph architecture.
  - icon: 🌳
    title: Agent · Pipeline · Leaf
    details: Reusable capabilities form the tree, execution strategies are branches, and optional disposable task executions are leaves.
  - icon: 🔧
    title: Small runtime, open extension points
    details: Built-in ReAct and optional pipeline/memory/macro crates. Other developers define what grows around the core.
  - icon: ⚙️
    title: Cloud, local and custom backends
    details: OpenAI-compatible APIs, native llama.cpp inference and user-defined engines behind a common runtime contract.
---

## Begin with the design, then the API

Ambi **does not prescribe how all agents should think or cooperate**. It supplies the small mechanisms for running them. See [Design philosophy](/advanced/design-philosophy) and [Architecture](/advanced/architecture) before choosing a workflow.

**New in 0.4.1:** opt-in [Leaf task delegation](/guide/leaf) gives `ChatRunner` a temporary, isolated subtask execution without creating a persistent second Agent. A Leaf inherits the parent's original tools and shares its engine, so it is **not a security sandbox**.

Continue with [Getting started](/guide/getting-started) or [Custom Pipeline](/advanced/custom-pipeline).
