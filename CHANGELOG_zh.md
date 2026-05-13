# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.8] — 2026-05-13

### 新功能

- **引擎 Trait 重构** — 将通用的 `evaluate_sentence_entropy` 从 `LLMEngineTrait` 替换为类型化的
  `as_any()` 向下转型机制：
  - 新增 `impl_as_any!()` 宏，减少所有引擎后端的样板代码。
  - 新增 `LLMEngine::backend_downcast_ref::<T>()` 方法，安全直接地访问具体引擎类型
    （如 `LlamaEngine`、`OpenAIEngine`、自定义引擎）。
  - 为 `LLMEngineTrait` 添加 `'static` 约束，确保安全的向下转型。

- **Agent 公共 API 扩展** — Agent 新增以下访问器方法：
  - `get_config()` — 只读访问 `AgentConfig`。
  - `get_tool_map()` — 运行时检查所有已注册工具。
  - `get_cached_tool_prompt()` — 获取已格式化的工具描述字符串。
  - `get_llama_engine()` — 获取底层 `Arc<LLMEngine>` 的直接句柄。

### 改进

- **Python 绑定** — 为 `PyEngineBridge` 添加 `impl_as_any!()`，确保 `LLMEngineTrait` 的正确实现。
- **Node.js 绑定** — 新增 `tool.rs` 模块，完成 Rust 源文件拆分；对 `config.rs`、`engine.rs`、`pipeline.rs`
  进行格式化整理。
- **代码质量** — 对引擎、Agent 核心、管道、绑定源码和示例文件统一应用 rustfmt 格式；
  调整 ambi-pipelines 子模块的导入顺序。
- **示例** — 更新 `custom_model_back.rs` 示例和 `local_text.rs` 测试，使用新的 `backend_downcast_ref` API。
- **文档** — 更新中英文绑定文档（node.md、python.md）、高级主题（custom-engine.md、design-philosophy.md）、
  扩展模块（ambi-macros.md、ambi-memory.md、ambi-pipelines.md）和指南（configuration.md、tools.md）。

### 维护

- 版本升级：0.3.7 → 0.3.8。
- 从 `LLMEngineTrait` 和 `Agent` 中移除已废弃的 `evaluate_sentence_entropy`；
  熵评估现在仅可通过 `LlamaEngine::evaluate_sentence_entropy` 使用。

## [0.3.7] — 2026-05-11

### 新功能

- **Python 绑定** (`bindings/python/`) — 全新的原生 Python 扩展
  基于 maturin/PyO3 构建。公开完整的 Agent API：
    - `Agent.make()`、`AgentState`、`Pipeline`、`LLMEngineConfig`
    - `resolve_request()` 用于自定义引擎异步回调
    - `_tool_helpers.py` 中的 `build_tool()` 辅助函数，方便工具注册
    - 内置模板函数：`chatml_template()`、`deepseek_template()`、
      `llama3_template()`、`qwen_template()` 等
    - 6 个即用型示例，涵盖聊天、流媒体、自定义引擎、
      自定义模板、工具调用和内存清理。
- **Node.js 绑定重构** — 完全重构 Node 绑定：
    - 新增 `bindings/node/lib/` 层，提供完善的 JavaScript API
      （`Agent`、`LLMEngineConfig`、`Pipeline`、`ChatStream`），基于

NAPI-RS 生成的类型。

### 维护

- 版本升级：0.3.6 → 0.3.7。

## [0.3.6] — 2026-05-05

### 新功能

- **`AgentState::fork()` / `fork_shared()`** — 创建独立并行对话宇宙的状态分支原语
  `ChatHistory` 和 `ChatRunner` 现在继承自
  `Clone` 以支持此功能。专为需要并发、隔离推理分支的思维树 (BFS) 和自洽性 (CoT)
  管道而设计。

- **`ambi-memory` crate (v0.1.0)** — 一个可插拔的多维认知记忆
  系统。公开了 `AgentStateMemoryExt` 扩展特性，包含：

- **键值记忆**：存储/调用键值状态（反射设置）。

- **语义记忆**：归档和向量搜索长期交互。

- **摘要记忆**：通过 LLM 辅助的驱逐机制实现滚动式防遗忘摘要
  压缩（`summarize_evicted_messages` 自动摘要已驱逐的聊天记录）。

- **`ambi-pipelines` crate (v0.1.0)** — 高级认知执行流水线：
- **RAG**：文档检索、打包、语义检索和流水线编排。
- **CoT 自洽性**：采用多数投票聚合的并行推理分支。
- **思维树 (ToT)**：对多条思维路径进行 BFS 束搜索。
- **反思**：具有持久评论记忆的 Actor-评估者循环。
- `ChatRunner` 已重新导出为 `ReactPipeline`，以便统一访问。

### 文档

- **模块级文档注释** 已添加到：`ambi-macros`（lib、`#[tool]`、`#[agent]`），
  所有 `ambi-pipelines` 模块（RAG、CoT、ToT、Reflexion），所有 `ambi-memory` 模块，
  `llama_cpp`，`openai_api`（stream、sync、translator），以及 `src/config/agent.rs`。

- **新增扩展文档** — `docs/extensions/ambi-macros.md`（英文和中文）
  替换了已删除的 `ambi-macros/README_zh.md`，现已集成到 VitePress
  网站中，并带有完整的侧边栏导航。

- **VitePress 配置** — 添加了 i18n 语言环境配置、本地搜索、扩展侧边栏
  部分以及英文和中文的社交链接。

## [0.3.5] — 2026-05-05

> **注：** 0.3.4 因未本地化中文文本且未注释而被弃用
> “crate_agent_in_macrp”示例中的代码。

### 漏洞修复

- **本地化“crate_agent_in_macrp”示例** — 所有中文文档评论，内嵌
  评论和面向用户的提示字符串已被翻译成英文。
  中文调试提示词“你试试你的add工具能不能用，我在调试。“被替换
  并提出一个具体的英文问题：“什么是114514加8080？”

- **Cargo.toml 中的寄存器示例** — 新增缺失的 '[[example]]' 条目
  'crate_agent_in_macrp' 具有必要功能 '[“openai-api”， “macro”]'，
  使它能够通过“货运航线——示例crate_agent_in_macrp——具有双向/宏功能”来建造。

### 文档

- 新增了逐步的英文文档注释（步骤1–5），涵盖API密钥
  设置、引擎配置、构建器实例化、聊天执行和结果
  在“crate_agent_in_macrp”示例中输出。

### 维护

- 版本提升：0.3.4（弃用版）→0.3.5。

## [0.3.4] — 2026-05-05

### 新功能

- **'#[agent]' 派生宏** — 一种新的声明式属性宏，消除了
  通过生成完整的代理界面（“代理”、“代理状态”、“管道”）来实现样板模板。
  来自一个单位结构。支持工具绑定、自定义流水线注入、会话ID
  赋值，并暴露了“chat（）”、“chat_stream（）”、“execute（）”、“set_dynamic_context（）”，
  以及直接在生成的立面上进行“clear_history（）”方法。重新导出后通过
  “ambi：：宏：：agent”。

- **OpenAI 配置构建器** — 'OpenAIEngineConfig：：create（api_key， model_name）' 提供
  一个带有确定性默认值的简明入口点，辅以“.base_url（）”，
  '.temp（）'， '.top_p（）' 构建方法用于灵活覆盖。

- **'Agent：：with_dyn_tools（）'** — 新方法接受“Vec<Arc<T>>”注册
  预构、弧形包裹的动态工具，无需手动定义提取。

### ⚙改进

- **宏模块重新导出** — 'ambi_macros'现已重新导出为'ambi：：macros'，
  提供一个干净的“ambi：：macros：：{tool， agent}'导入路径。
- **“动态链接”功能** — 新货运功能，支持动态链接
  llama.cpp后端。
- **VitePress 文档配置** — 完整的文档网站配置，包含英文和
  中文本地搜索、侧边栏导航和GitHub社交链接。

### 示例

- **'crate_agent_in_macrp.rs'** — 演示“#[agent]'宏的端到端使用
  通过自定义的“AddTool”，将“builder（）”→“preamble（）”→“build（）”串联起来。
- 更新了'custom_tool_in_macro.rs'导入路径为'ambi：：macros：：tool'。

### 维护

- 版本提升至 0.3.4。
- 更新了“双宏/README”文档。

## [0.3.3] - 2026-05-06

### Changed

- **核心重构**：将 `Blueprint`（`Agent`）与 `State`（`AgentState`）彻底解耦。Agent 现在为只读蓝图，`AgentState` 持有可变运行时状态（
  `session_id`、`dynamic_context`）。
- `AgentState` 新增 `session_id` 字段，用于分布式追踪和 KV Cache 槽位分配。
- `AgentState` 新增 `dynamic_context` 字段，用于安全注入 RAG 结果、环境变量等易变数据，不受 token 驱逐影响。
- `ChatHistory` 将 `Message::System` 彻底移除，现为纯粹的 `User`、`Assistant`、`Tool` 事件 FIFO 队列，驱逐算法简化为 O(1)
  截断。
- `ChatHistory` 新增 `search_by_keyword`、`last_user_message`、`last_assistant_message` 便捷方法。
- `EvictionHandler` 回调签名修改为接收 `&AgentState`，允许在归档回调中安全提取状态扩展字段（如数据库连接池）。
- `Agent` 内部字段（`config`、`cached_tool_prompt`）改用 `Arc` 包装，实现 Tokio 任务间零成本克隆。
- `ChatRunner` 从单元结构体改为持有 `maximum_concurrency` 字段（默认 5），允许灵活配置并行工具执行的速率限制。
- `AgentState::new` 和 `new_shared` 现需传入 `session_id` 参数。
- `LLMEngineConfig` 新增 `Custom(Box<dyn LLMEngineTrait>)` 变体，`Agent::with_custom_engine` 标记为废弃，推荐使用
  `Agent::make(LLMEngineConfig::Custom(...))`。
- 更新所有示例和测试，以使用新的 API 模式。

### Added

- **WASM 流式支持**：在 WASM 目标上，利用浏览器原生 `fetch` 和 `ReadableStream` API 实现了完整的流式响应处理 (
  `openai_api/stream.rs`)。
- **WASM 示例增强**：`examples/webAssembly` 新增流式/普通模式切换开关，支持实时流式文本生成演示。
- **`#[tool]` 宏增强**：新增 `params(...)` 属性，允许为函数参数注入面向 LLM 的描述，提升工具路由准确性。
- `AgentState` 新增 `set_dynamic_context`、`append_dynamic_context`、`clear_dynamic_context` 方法，方便操作动态上下文。
- 新增 `anymap2` 依赖，为 `AgentState` 提供类型安全的扩展存储 (`extensions`)。

### Fixed

- 修复 `ChatHistory` 驱逐算法中的潜在溢出问题。
- 修复 OpenAI 提供者流式实现中的工具调用增量收集逻辑。
- 修复 WASM 示例中的 API 基础 URL 和模型配置。

## [0.3.2] - 2026-05-05

### Added

- `AgentState::new_shared()` 便捷构造函数，返回 `Arc<RwLock<AgentState>>`，减少样板代码。

### Changed

- 所有示例、测试、文档（中英文）均更新为使用 `new_shared()`。
- 版本号更新至 0.3.2。
- Node.js 绑定文档和可选依赖中，平台范围缩限并添加 `@maskviva/` 命名空间前缀。

## [0.3.1] - 2026-05-01

### Changed

- 更新 README 中的版本号至 0.3。
- 更新 `.gitignore` 以忽略 Node 绑定的构建产物。
- 修复 VitePress 配置中的格式问题。
- 重构 Agent 构建器初始化逻辑。

## [0.3.0] - 2026-04-30

### Added

- **零成本跨平台运行时**：新增 `src/runtime.rs`，作为平台无关的异步运行时抽象层。
- **WebAssembly 支持**：框架现可编译到 `wasm32-unknown-unknown` 目标，在浏览器中运行。
- 新增 `examples/webAssembly` 完整项目，展示如何导出有状态的 `AmbiSession` 类到 JavaScript。
- 新增高级定制指南文档页面（架构、自定义引擎、自定义管道、流式格式化器、工具解析器、上下文驱逐）。
- 英/中双语 VitePress 文档站点初始化。
- 新增 `#[tool]` 宏的文档和使用说明。

### Changed

- 版本号更新至 0.3.0。
- `ChatRunner` 的方法调用从静态风格（`ChatRunner::chat(&runner, ...)`）改为实例风格（`runner.chat(...)`）。
- 所有示例和测试均更新为新的调用方式。
- 清理和重构了多个内部模块结构。
- `docs.rs` 特性限制为不依赖硬件 SDK 的组合，避免文档构建失败。

### Fixed

- 修复 WASM 目标下的编译错误（`Send`/`Sync` trait bound 调整）。
- 限制 `docs.rs` 特性以避免硬件 SDK 编译失败。

## [0.2.8] - 2026-04-24

### Added

- **`#[ambi::tool]` 过程宏**：引入 `ambi-macros` 子包，开发者可注解异步函数，自动生成 `Tool` trait 实现。
- 新增 `custom_tool_in_macro.rs` 示例，演示宏驱动的工具创建流程。
- `#[tool]` 宏支持行内配置：`name`， `description`， `timeout_secs`， `max_retries`， `is_idempotent`。
- 新增 `chat_stream.rs`、`custom_chat_template.rs`、`custom_tool_parser.rs`、`memory_eviction_handler.rs` 示例。

### Changed

- 版本号更新至 0.2.8。
- 更新 `.gitignore` 忽略 `ambi-macros` 的锁文件。

## [0.2.7] - 2026-04-22

### Changed

- **DDD 架构重构**：代码库严格遵循领域驱动设计原则重组，消除反向依赖和抽象泄漏。
- **类型模块纯化**：`types` 模块现仅包含纯数据传输对象（DTO）和契约，业务逻辑移至 `agent` 层。
- **配置提取**：`AgentConfig`、`LlamaEngineConfig`、`OpenAIEngineConfig` 统一到顶级 `config` 模块。
- **格式化器解耦**：移除 `ToolCallParser` 与 `StreamFormatter` 间的硬依赖，通过 `FormatterFactory` 实现依赖注入。
- **全面文档化**：所有公开模块、结构体、trait 和方法均添加专业英文 Rustdoc 注释。

### Fixed

- **关键线程挂起漏洞**：修复 `ToolManager::run_tool` 中非幂等工具绕过超时控制的严重 bug。
- **驱逐算法性能陷阱**：消除 `evict_old_messages` 中的 O(N^2) 复杂度和潜在的静默失败/无限循环，替换为高效 O(N) FIFO 算法。
- **快速失败验证**：Agent 构建器中，重复的工具注册现在立即触发 `AmbiError`，而非静默覆盖。

## [0.2.6] - 2026-04-19

### Changed

- **错误传播全面改造**：`LLMEngine::count_tokens` 和 `LLMEngine::from_custom` 现返回 `Result`，防止分词器初始化失败时的静默回退。
- **非幂等工具处理**：声明 `is_idempotent: false` 的工具现直接调用，绕过框架级超时/重试循环，防止支付等关键操作因超时而中断。
- **统一工具错误报告**：`handle_tool_calls` 现返回 `Result<Vec<_>>`，在首个失败时即停止流，并短接客户端断开后的幽灵工具执行。
- **安全熵评估**：`evaluate_sentence_entropy` 现快照并恢复推理会话状态，仅清除消耗的 token 的 KV Cache，防止上下文损坏。
- **Llama.cpp 优化**：移除可能导致关闭时停滞的不必要 `join()`；添加上下文批量大小配置；移除模块级 `#![cfg(...)]` 守卫，改用
  crate 级特性标记。
- **清理**：移除 `VisionContext` 上不安全的 `Send`/`Sync` 实现。
- **文档**：为 `DynTool` 实现者添加取消安全性文档。

### Fixed

- 修复非幂等工具在错误场景下的语义行为，提升管道可靠性。

## [0.2.5] - 2026-04-16

### Added

- **原生多模态推理**：启用基于 MTMD API 的完整多模态推理支持。
- 新增 `multimodal_inference` 专用推理路径，包含干净的 KV Cache 处理。
- 通过 `ChatTemplate` 传播 `<__media__>` 标记。

### Changed

- **工具调用解析鲁棒性增强**：现在可优雅恢复截断的 JSON（尝试解析到最后一个 `}` 的回退机制）。
- **流式格式化器优化**：避免不必要的前缀，改进对 think/tool 标签的检测。
- **引擎修复**：修正 `sampler.sample()` 以在生成循环中使用正确的 `logits_idx`。
- **多模态支持暴露**：通过 `LLMEngineTrait::supports_multimodal` 暴露多模态能力。
- 版本号更新至 0.2.5。

## [0.2.4] - 2026-04-13

### Changed

- **Llama.cpp 引擎架构完全重构**：将单体引擎分解为单职责模块（`command`， `dispatch`， `inference`， `session`， `vision`）。
- **快照/回滚推理完整性**：每个失败路径都遵循一致的快照/回滚模式，防止状态损坏。
- **引擎存活检测**：`LlamaEngine` 现通过 `Arc<AtomicBool>` 暴露 `is_alive()` 方法。
- **API 改进**：`with_eviction_strategy` 现接受语义化的 `EvictionStrategy` 结构体。
- **会话模板扩展**：`ChatTemplate` 新增 `tool_id_prefix`/`tool_id_suffix` 字段。
- **依赖更新**：`async-openai` 更新至 0.36.0，`llama-cpp-2` 更新至 0.1.145。
- 版本号更新至 0.2.4。

### Security

- **流式处理异常安全**：新增看门狗任务，捕获内部异常并转发错误，防止静默丢弃连接。
- **客户端断连感知工具执行**：`handle_tool_calls` 现可检测客户端断连并立即中止幽灵工具执行。
- **缓冲区溢出保护**：`TagStreamFormatter` 在超限时清空自身并记录错误，而非返回可能误导用户的错误信息。

## [0.2.3] - 2026-04-10

### Changed

- **精确上下文驱逐**：以精确的 `total_tokens` 累加器取代基于长度的 token 估算，消除所有 `/4` 启发式算法。
- **推理状态完整性**：引入 `InferenceSession::snapshot`/`restore` 机制，每个失败路径均恢复状态并清除 KV Cache。
- **增强流式鲁棒性**：重构 `process_llm_stream` 以正确传播流失败；OpenAI 流式处理器在原生 `tool_calls` 开始后抑制文本内容。

### Added

- **清晰配置 API**：用自文档化的 `EvictionStrategy` 结构体取代原有的不透明元组。
- **扩展提示词创作**：`ChatTemplate` 新增 `tool_id_prefix`/`tool_id_suffix`，让自定义模板完全控制 tool-call 标识符的渲染。
- **引擎存活状态**：`LlamaEngine::is_alive()` 通过 `Arc<AtomicBool>` 实现，线程崩溃或退出时所有后续派发错误会明确指示用户重建
  `Agent`。
- **运行时要求**：README 中明确标注 Tokio `rt-multi-thread` 为必须项。
- **多模态安全**：`process_images` 现返回明确 `EngineError` 且包含前瞻性消息。
- 版本号更新至 0.2.3。

### Fixed

- 修复高并发下状态同步的潜在数据竞争问题。
- 修复流式响应中断后上下文未能正确清理的问题。
- **方法重命名**：`enable_formatting` 改为 `with_formatting`，与其他流式 API 保持一致。

## [0.2.2] - 2026-04-07

### Changed

- 重构 Agent 状态管理和并发模型：`AgentState` 从 `Agent` 中分离，封装对话历史和缓存 token 计数。
- **并发改进**：将 `tokio::sync::Mutex` 替换为 `std::sync::Mutex` 管理 `AgentState`，消除异步锁竞争。
- **引擎不可变性**：`LLMEngineTrait` 所有方法从 `&mut self` 改为 `&self`，允许跨线程自由共享。
- 统一同步与流式聊天循环的实现代码。
- 版本号更新至 0.2.2。

### Fixed

- 移除 `AgentBusy` 错误变体，新架构通过状态克隆和显式管道隔离支持并发访问。
- 修复并发工具调用中的状态错误。

## [0.2.1] - 2026-04-04

### Changed

- 采用提前返回（early return）模式重构错误处理逻辑，减少代码嵌套。
- 版本号提升至 0.2.1。

## [0.2.0] - 2026-04-01

### Added

- **原生多模态和视觉支持**：Agent 管道现可处理图像，`ContentPart` 扩展支持 `Image`，新增 `Message::user_multimodal` 助手方法。
- **RAG 长期记忆**：引入 `MemoryManager`、`EmbeddingEngineTrait`、`MemoryStoreTrait` 及 `VectorMemoryStore`，支持自动归档和检索。
- **基础 RAG 记忆能力**：`Agent::with_memory()` 挂载记忆管理器，自动注入相关历史上下文。
- 引入 `#[tool]` 宏（v0.2.8 中回归），为工具创建实现零样板代码。
- 新增工具解析器、格式化器和模板的自定义能力。

### Changed

- **引擎接口抽象化**：引入 `LLMEngineTrait`，支持自定义引擎通过 `Agent::with_custom_engine` 接入。
- **Llama.cpp 异步重构**：将同步推理隔离到独立后台线程，通过 `mpsc`/`oneshot` 通道通信，解决 Tokio 运行时阻塞问题。
- **强化工具调用**：添加超时机制（默认15秒）和自动重试（最多3次，含500ms延迟）。
- 内存系统解耦为可通过 `on_evict` 钩子加载的外部插件。
- 实现 KV 缓存移位、动态 `n_ctx` 保护及幽灵工具中止等性能优化。
- `EngineConfig` 从 `Option` 结构体重构为类型安全的 `Enum`。
- 全面重构错误处理、Llama 引擎模块化和管道设计。
- 对模块架构进行大规模解耦和重新设计。
- 版本号更新至 0.2.0。

### Fixed

- 为 Agent 增强了 OOM 保护、工具幂等性及基于 token 数量的智能驱逐算法。
- 修复示例代码中的导入路径。
- 修复并发工具调用中的状态错误。

## [0.1.6] - 2026-03-25

### Added

- 引入 token 级别的熵评估功能，可用于 Llama 引擎生成质量监测。

### Changed

- 更新 README 以反映 v0.1.6 功能。

## [0.1.5] - 2026-03-22

### Added

- 核心 Agent 的消息、历史记录和格式化模块基本成形。

### Changed

- 重构代码风格和逻辑流程，增强可读性。
- 移除自定义日志工具，改用标准 `log` 库。
- 深度重构 LLM 引擎架构并增强 Agent 核心能力。
- 重组模块结构，移除 `core` 中间层。
- 重构消息和历史管理结构，优化数据流。

### Fixed

- 修复配置示例代码的 `required-features` 设置并清理编译警告。
- 修复 CI 中的代码格式错误（`cargo fmt`）。

### Security

- 通过强化工具执行的幂等性和安全校验，降低意外状态修改风险。

## [0.1.4] - 2026-03-15

### 新增

- 增强了工具定义，增加了 `timeout_secs`、`max_retries` 和 `is_idempotent` 标志。
- 添加了全面的日志记录和错误传播改进。

### 更改

- 版本提升至 0.1.4。

## [0.1.3] - 2026-03-14

### 更改

- 更新了 README 和文档以包含新功能。
- 改进了工具管理和上下文驱逐逻辑。

## [0.1.2] - 2026-03-13

### 更改

- 版本提升至 0.1.2。

## [0.1.1] - 2026-03-12

### 更改

- 版本提升至 0.1.1。

## [0.1.0] - 2026-03-15

### Added

- 项目初始提交，包含 Apache-2.0 许可证。
- 首个可用原型：含基本的 Agent 循环、LLM 引擎抽象（OpenAI API 和 Llama.cpp）和示例代码（`chat_cloud`、`chat_local`、
  `custom_tool`）。
- 配置驱动的系统提示词、对话模板和工具挂载。
- 添加 GitHub Actions CI 流程，进行自动化测试、格式检查和代码风格检查（clippy）。
- 加入初步的 README 文档和快速入门指南。