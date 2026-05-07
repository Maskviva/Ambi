# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.5] — 2026-05-05

> **Note:** 0.3.4 is deprecated due to unlocalized Chinese text and uncommented
> code in the `crate_agent_in_macrp` example.

### Bug Fixes

- **Localize `crate_agent_in_macrp` example** — All Chinese doc comments, inline
  comments, and user-facing prompt strings have been translated to English.
  The Chinese debug prompt "你试试你的add工具能不能用，我在调试。" is replaced
  with a concrete English query "What is 114514 plus 8080?".

- **Register example in Cargo.toml** — Added missing `[[example]]` entry for
  `crate_agent_in_macrp` with required features `["openai-api", "macro"]`,
  enabling it to be built via `cargo run --example crate_agent_in_macrp --features ambi/macro`.

### Documentation

- Added step-by-step English documentation comments (Steps 1–5) covering API key
  setup, engine configuration, builder instantiation, chat execution, and result
  output in the `crate_agent_in_macrp` example.

### Maintenance

- Version bump: 0.3.4 (deprecated) → 0.3.5.

## [0.3.4] — 2026-05-05

### New Features

- **`#[agent]` Derive Macro** — A new declarative attribute macro that eliminates
  boilerplate by generating a complete agent facade (`Agent` + `AgentState` + `Pipeline`)
  from a unit struct. Supports tool binding, custom pipeline injection, session ID
  assignment, and exposes `chat()`, `chat_stream()`, `execute()`, `set_dynamic_context()`,
  and `clear_history()` methods directly on the generated facade. Re-exported via
  `ambi::macros::agent`.

- **OpenAI Config Builder** — `OpenAIEngineConfig::create(api_key, model_name)` provides
  a concise entry point with deterministic defaults, complemented by `.base_url()`,
  `.temp()`, `.top_p()` builder methods for flexible overrides.

- **`Agent::with_dyn_tools()`** — New method accepting `Vec<Arc<T>>` for registering
  pre-constructed, Arc-wrapped dynamic tools without manual definition extraction.

### ⚙Improvements

- **Macro Module Re-export** — `ambi_macros` is now re-exported as `ambi::macros`,
  providing a clean `ambi::macros::{tool, agent}` import path.
- **`dynamic-link` Feature** — New Cargo feature enabling dynamic linking for
  llama.cpp backends.
- **VitePress Docs Config** — Full documentation site configuration with English and
  Chinese locales, local search, sidebar navigation, and GitHub social link.

### Examples

- **`crate_agent_in_macrp.rs`** — Demonstrates end-to-end usage of the `#[agent]` macro
  with a custom `AddTool`, chaining `builder()` → `preamble()` → `build()`.
- Updated `custom_tool_in_macro.rs` import path to `ambi::macros::tool`.

### Maintenance

- Bumped version to 0.3.4.
- Updated `ambi-macros/README` documentation.

## [0.3.3] - 2026-05-06

### Changed

- **Core refactor**: completely decoupled `Agent` (read‑only blueprint) from `AgentState` (mutable runtime state holding
  `session_id` and `dynamic_context`).
- `AgentState` now carries a `session_id` field for distributed tracing and KV‑cache slotting.
- `AgentState` gains a `dynamic_context` field for safely injecting volatile data (e.g., RAG results, environment
  variables) without interfering with token eviction.
- `ChatHistory` purged `Message::System`; it is now a pure FIFO queue of `User`, `Assistant`, and `Tool` events, making
  the eviction algorithm O(1).
- `ChatHistory` gained helper methods: `search_by_keyword`, `last_user_message`, `last_assistant_message`.
- `EvictionHandler` callback signature changed to receive `&AgentState`, allowing safe extraction of extension fields (
  e.g., DB connection pools) during archiving.
- `Agent`’s internal fields (`config`, `cached_tool_prompt`) wrapped in `Arc` for zero‑cost cloning across Tokio tasks.
- `ChatRunner` converted from a unit struct to a struct holding `maximum_concurrency` (default 5), enabling rate‑limited
  parallel tool execution.
- `AgentState::new` and `new_shared` now require a `session_id` argument.
- `LLMEngineConfig` now includes a `Custom(Box<dyn LLMEngineTrait>)` variant; `Agent::with_custom_engine` is deprecated
  in favour of `Agent::make(LLMEngineConfig::Custom(...))`.
- All examples and tests updated to the new API patterns.

### Added

- **WASM streaming**: on WASM targets, browser‑native `fetch` and `ReadableStream` APIs are used to implement full
  streaming response handling (`openai_api/stream.rs`).
- **WASM example enhancement**: `examples/webAssembly` UI now includes a streaming/normal toggle, demonstrating
  real‑time text generation.
- **`#[tool]` macro enhancement**: a new `params(...)` attribute allows LLM‑facing descriptions to be injected into
  function arguments for better tool routing.
- `AgentState` gained `set_dynamic_context`, `append_dynamic_context`, and `clear_dynamic_context` methods.
- Added `anymap2` dependency for type‑safe extension storage (`extensions`) in `AgentState`.

### Fixed

- Potential overflow in `ChatHistory` eviction algorithm.
- Tool‑call delta collection logic in the OpenAI provider’s streaming implementation.
- API base URL and model configuration in the WASM example.

## [0.3.2] - 2026-05-05

### Added

- `AgentState::new_shared()` convenience constructor returning `Arc<RwLock<AgentState>>`, reducing boilerplate.

### Changed

- All examples, tests, and documentation (English & Chinese) updated to use `new_shared()`.
- Version bumped to 0.3.2.
- Node.js binding docs and optional dependencies narrowed platform scope and added `@maskviva/` namespace prefix.

## [0.3.1] - 2026-05-01

### Changed

- Updated version in README to 0.3.
- Updated `.gitignore` for Node binding build artifacts.
- Fixed formatting in VitePress config.
- Refactored Agent builder initialization logic.

## [0.3.0] - 2026-04-30

### Added

- **Zero‑cost cross‑platform runtime**: new `src/runtime.rs` serving as a platform‑agnostic async runtime abstraction.
- **WebAssembly support**: the framework now compiles to `wasm32-unknown-unknown` and runs in the browser.
- Complete `examples/webAssembly` project demonstrating how to export a stateful `AmbiSession` class to JavaScript.
- Advanced customization guide documentation pages (architecture, custom engines, custom pipelines, stream formatters,
  tool parsers, context eviction).
- Bilingual (EN/ZH) VitePress documentation site initialized.
- Documentation and usage instructions for the `#[tool]` macro.

### Changed

- Version bumped to 0.3.0.
- `ChatRunner` method calls changed from static style (`ChatRunner::chat(&runner, ...)`) to instance style (
  `runner.chat(...)`).
- All examples and tests updated to the new invocation style.
- Multiple internal module structures cleaned up and refactored.
- `docs.rs` feature set restricted to avoid hardware SDK compilation failures.

### Fixed

- Compilation errors on WASM targets (`Send`/`Sync` trait bound adjustments).
- Restricted `docs.rs` features to avoid hardware SDK compilation failures.

## [0.2.8] - 2026-04-24

### Added

- **`#[ambi::tool]` procedural macro**: introduced the `ambi‑macros` sub‑crate; developers can annotate async functions
  to auto‑generate `Tool` trait implementations.
- New `custom_tool_in_macro.rs` example demonstrating macro‑driven tool creation.
- The `#[tool]` macro supports inline configuration: `name`, `description`, `timeout_secs`, `max_retries`,
  `is_idempotent`.
- New examples: `chat_stream.rs`, `custom_chat_template.rs`, `custom_tool_parser.rs`, `memory_eviction_handler.rs`.

### Changed

- Version bumped to 0.2.8.
- Updated `.gitignore` to ignore `ambi‑macros` lock file.

## [0.2.7] - 2026-04-22

### Changed

- **DDD architecture refactor**: the codebase was reorganized following Domain‑Driven Design principles, eliminating
  reverse dependencies and abstraction leaks.
- **Type module purification**: the `types` module now strictly contains pure data transfer objects (DTOs) and
  contracts; business logic moved back to the `agent` layer.
- **Configuration extraction**: `AgentConfig`, `LlamaEngineConfig`, and `OpenAIEngineConfig` unified into a top‑level
  `config` module.
- **Formatter decoupling**: removed the hard dependency between `ToolCallParser` and `StreamFormatter`, introducing
  dependency injection via `FormatterFactory`.
- **Comprehensive documentation**: all public modules, structs, traits, and methods now have professional English
  Rustdoc comments.

### Fixed

- **Critical thread‑hanging vulnerability**: fixed a bug in `ToolManager::run_tool` where non‑idempotent tools could
  bypass timeout controls.
- **Eviction algorithm performance trap**: eliminated the O(N²) complexity and potential silent‑failure/infinite‑loop in
  `evict_old_messages`, replacing it with an efficient O(N) FIFO algorithm.
- **Fail‑fast validation**: duplicate tool registrations in the Agent builder now immediately trigger an `AmbiError`
  instead of silently overwriting.

## [0.2.6] - 2026-04-19

### Changed

- **Error propagation overhaul**: `LLMEngine::count_tokens` and `LLMEngine::from_custom` now return `Result`, preventing
  silent fallbacks when tokenizer initialization fails.
- **Non‑idempotent tool handling**: tools declaring `is_idempotent: false` are now invoked directly, bypassing the
  framework‑level timeout/retry loop to avoid interrupting critical operations (e.g., payments) on timeout.
- **Unified tool‑error reporting**: `handle_tool_calls` now returns `Result<Vec<_>>`, stopping the stream on the first
  failure and short‑circuiting ghost tool execution after client disconnect.
- **Safe entropy evaluation**: `evaluate_sentence_entropy` now snapshots and restores inference session state, clearing
  only the consumed tokens’ KV cache to prevent context corruption.
- **Llama.cpp optimizations**: removed unnecessary `join()` that could stall shutdown; added batch‑size configuration;
  removed module‑level `#![cfg(...)]` gate in favour of crate‑level feature flags.
- **Cleanup**: removed unsafe `Send`/`Sync` implementations on `VisionContext`.
- **Documentation**: added cancel‑safety documentation for `DynTool` implementors.

### Fixed

- Fixed non‑idempotent tool semantics in error scenarios, improving pipeline reliability.

## [0.2.5] - 2026-04-16

### Added

- **Native multimodal inference**: full multimodal inference support enabled via the MTMD API.
- New dedicated `multimodal_inference` path with clean KV‑cache handling.
- `<__media__>` markers propagated through `ChatTemplate`.

### Changed

- **Tool‑call parsing robustness**: now gracefully recovers from truncated JSON (fallback parse up to the last `}`).
- **Stream formatter optimizations**: avoided unnecessary prefixes, improved detection of think/tool tags.
- **Engine fixes**: corrected `sampler.sample()` to use proper `logits_idx` throughout the generation loop.
- **Multimodal support exposed**: `LLMEngineTrait::supports_multimodal` now correctly reflects engine capability.
- Version bumped to 0.2.5.

## [0.2.4] - 2026-04-13

### Changed

- **Llama.cpp engine architecture fully restructured**: decomposed the monolithic engine into single‑responsibility
  modules (`command`, `dispatch`, `inference`, `session`, `vision`).
- **Snapshot/rollback inference integrity**: every failure path consistently restores state and clears the KV cache to
  prevent corruption.
- **Engine liveness detection**: `LlamaEngine` now exposes `is_alive()` backed by an `Arc<AtomicBool>`.
- **API improvement**: `with_eviction_strategy` now accepts a self‑documenting `EvictionStrategy` struct.
- **Chat template extension**: `ChatTemplate` gained `tool_id_prefix` and `tool_id_suffix` fields.
- **Dependency updates**: `async-openai` upgraded to 0.36.0, `llama‑cpp‑2` to 0.1.145.
- Version bumped to 0.2.4.

### Security

- **Streaming panic safety**: a watchdog task now catches internal panics and forwards errors, preventing silent
  connection drops.
- **Client‑disconnect‑aware tool execution**: `handle_tool_calls` detects client disconnection and immediately aborts
  ghost tool executions.
- **Buffer overflow protection**: `TagStreamFormatter` now clears itself and logs an error on overflow instead of
  returning potentially misleading error messages.

## [0.2.3] - 2026-04-10

### Changed

- **Precise context eviction**: replaced the old length‑based token estimation with the exact `total_tokens`
  accumulator, eliminating all `/4` heuristics.
- **Inference state integrity**: introduced `InferenceSession::snapshot`/`restore`; every failure path restores state
  and clears the KV cache.
- **Enhanced streaming robustness**: refactored `process_llm_stream` to properly propagate stream failures; the OpenAI
  streaming handler now suppresses text content once native `tool_calls` begin.

### Added

- **Clear configuration API**: replaced the opaque legacy tuple with a self‑documenting `EvictionStrategy` struct.
- **Extended prompt authoring**: `ChatTemplate` gained `tool_id_prefix` and `tool_id_suffix` fields, giving custom
  templates full control over tool‑call identifier rendering.
- **Engine liveness**: `LlamaEngine::is_alive()` backed by `Arc<AtomicBool>`; after a thread crash or exit, all
  subsequent dispatch errors explicitly instruct users to recreate the `Agent`.
- **Runtime requirement**: README now clearly states that Tokio `rt‑multi‑thread` is mandatory.
- **Multimodal safety**: `process_images` now returns a clear `EngineError` with a forward‑looking message.
- Version bumped to 0.2.3.

### Fixed

- Fixed potential data races in state synchronization under high concurrency.
- Fixed context not being properly cleaned up after stream interruption.
- **Method rename**: `enable_formatting` renamed to `with_formatting` for consistency with the fluent API.

## [0.2.2] - 2026-04-07

### Changed

- Refactored Agent state management and concurrency model: `AgentState` separated from `Agent`, encapsulating
  conversation history and cached token counts.
- **Concurrency improvement**: replaced `tokio::sync::Mutex` with `std::sync::Mutex` for `AgentState` management,
  eliminating async lock contention.
- **Engine immutability**: all `LLMEngineTrait` methods changed from `&mut self` to `&self`, allowing engines to be
  freely shared across threads.
- Unified the sync and streaming chat loop implementations.
- Version bumped to 0.2.2.

### Fixed

- Removed the `AgentBusy` error variant; the new architecture inherently supports concurrent access through state
  cloning and explicit pipeline isolation.
- Fixed state errors during concurrent tool calls.

## [0.2.1] - 2026-04-04

### Changed

- Refactored error handling logic using early‑return patterns to reduce nesting.
- Version bumped to 0.2.1.

## [0.2.0] - 2026-04-01

### Added

- **Native multimodal and vision support**: the Agent pipeline can now process images; `ContentPart` extended with
  `Image` variant; new `Message::user_multimodal` helper.
- **RAG‑based long‑term memory**: introduced `MemoryManager`, `EmbeddingEngineTrait`, `MemoryStoreTrait`, and
  `VectorMemoryStore` for automatic archiving and retrieval.
- **Basic RAG memory capabilities**: `Agent::with_memory()` mounts a memory manager and automatically injects relevant
  historical context.
- Introduced the `#[tool]` macro (re‑introduced in v0.2.8) for zero‑boilerplate tool creation.
- Customization capabilities for tool parsers, formatters, and templates.

### Changed

- **Engine interface abstraction**: introduced `LLMEngineTrait`; custom engines can be integrated via
  `Agent::with_custom_engine`.
- **Llama.cpp async refactor**: synchronous inference isolated into dedicated background threads, communicating via
  `mpsc`/`oneshot` channels, solving Tokio runtime blocking issues.
- **Robust tool calling**: added timeout mechanism (default 15 s) and automatic retry (up to 3 times with 500 ms delay).
- Memory system decoupled into external plugins via the `on_evict` hook.
- Implemented KV‑cache shifting, dynamic `n_ctx` protection, and ghost‑tool abortion.
- `EngineConfig` refactored from `Option` structs to a type‑safe `Enum`.
- Comprehensive overhaul of error handling, Llama engine modularization, and pipeline design.
- Major architectural decoupling and reorganization of modules.
- Version bumped to 0.2.0.

### Fixed

- Added OOM protection, tool idempotency, and token‑aware intelligent eviction for the Agent.
- Fixed import paths in example code.
- Fixed state errors during concurrent tool calls.

## [0.1.7] - 2026-03-25

### Changed

- Refined error handling and module organization: introduced `AmbiError` enum with `thiserror`, extracted config types
  into `src/types/config`, and refactored pipeline into `ChatRunner`.

## [0.1.6] - 2026-03-25

### Added

- Token‑level entropy evaluation capability for the Llama engine (generation quality monitoring).
- Updated README to reflect v0.1.6 functionality.

## [0.1.5] - 2026-03-22

### Added

- Core Agent message, history, and formatter modules took shape.

### Changed

- Refactored code style and logic flows for readability.
- Removed custom logger in favour of the standard `log` crate.
- Deeply refactored the LLM engine architecture and enhanced core Agent capabilities.
- Reorganized module structure, removing the `core` middle layer.
- Refactored message and history management, optimizing data flow.

### Fixed

- Fixed `required‑features` in example configs and cleaned up compilation warnings.
- Fixed code formatting errors in CI (`cargo fmt`).

### Security

- Hardened tool execution through idempotency checks and safety validations, reducing the risk of accidental state
  modifications.

## [0.1.4] - 2026-03-15

### Added

- Enhanced tool definitions with `timeout_secs`, `max_retries`, and `is_idempotent` flags.
- Added comprehensive logging and error propagation improvements.

### Changed

- Version bumped to 0.1.4.

## [0.1.3] - 2026-03-14

### Changed

- Updated README and documentation with new features.
- Improved tool management and context eviction logic.

## [0.1.2] - 2026-03-13

### Changed

- Version bumped to 0.1.2.

## [0.1.1] - 2026-03-12

### Changed

- Version bumped to 0.1.1.

## [0.1.0] - 2026-03-12

### Added

- Initial project commit with Apache‑2.0 license.
- First functional prototype: basic Agent loop, LLM engine abstractions (OpenAI API and Llama.cpp), and example code (
  `chat_cloud`, `chat_local`, `custom_tool`).
- Configuration‑driven system prompts, chat templates, and tool mounting.
- GitHub Actions CI workflow for automated testing, formatting, and clippy checks.
- Initial README documentation and quick‑start guide.