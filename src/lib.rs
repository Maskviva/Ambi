#[cfg(any(
    all(feature = "cuda", feature = "vulkan"),
    all(feature = "cuda", feature = "metal"),
    all(feature = "cuda", feature = "rocm"),
    all(feature = "vulkan", feature = "metal"),
    all(feature = "vulkan", feature = "rocm"),
    all(feature = "metal", feature = "rocm")
))]
compile_error!(
    "不能同时启用多个 LLM 硬件加速后端 请在 cuda, vulkan, metal, rocm 中只选择一个 feature。"
);

pub mod config;
pub mod core;
pub mod tools;
pub mod utils;
