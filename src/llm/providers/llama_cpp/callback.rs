// src/llm/providers/llama_cpp/engine/callback.rs

use log::{debug, error, info, trace, warn};
use std::ffi::CStr;

/// Logging callback suitable for `llama_log_set`.
///
/// Translates llama.cpp's GGML log levels to Rust's `log` crate levels.
/// # Safety
///
/// This function is intended to be passed as a C callback.  It must only be
/// called from the llama.cpp library while the `CStr` pointer is valid.
pub(crate) unsafe extern "C" fn llama_log_callback(
    level: llama_cpp_sys_2::ggml_log_level,
    text: *const std::os::raw::c_char,
    _data: *mut std::os::raw::c_void,
) {
    // `text` is guaranteed to be a null-terminated C string by llama.cpp.
    let text = unsafe { CStr::from_ptr(text) };
    let log_str = text.to_string_lossy();
    // llama.cpp often appends a newline; trim it to avoid double-spacing.
    let clean = log_str.trim_end();

    match level {
        llama_cpp_sys_2::GGML_LOG_LEVEL_DEBUG => debug!("{}", clean),
        llama_cpp_sys_2::GGML_LOG_LEVEL_ERROR => error!("{}", clean),
        llama_cpp_sys_2::GGML_LOG_LEVEL_WARN => warn!("{}", clean),
        llama_cpp_sys_2::GGML_LOG_LEVEL_INFO => info!("{}", clean),
        llama_cpp_sys_2::GGML_LOG_LEVEL_CONT => trace!("{}", log_str), // keep original for continuation
        _ => {} // Silently ignore unknown log levels.
    }
}
