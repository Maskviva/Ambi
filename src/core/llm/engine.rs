use anyhow::{anyhow, Result};
use std::ffi::CStr;
use std::num::NonZeroU32;
use std::path::Path;

use crate::core::llm::LlamaEngineConfig;
use crate::utils::LLM_LOGGER;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_sys_2;

extern "C" fn llm_engine_log_callback(
    level: llama_cpp_sys_2::ggml_log_level,
    text: *const std::os::raw::c_char,
    _data: *mut std::os::raw::c_void,
) {
    let text = unsafe { CStr::from_ptr(text) };
    let log_str = &text.to_string_lossy();

    // 根据日志级别处理
    match level {
        llama_cpp_sys_2::GGML_OP_NONE => {
            LLM_LOGGER.none(log_str);
        }
        llama_cpp_sys_2::GGML_LOG_LEVEL_DEBUG => {
            LLM_LOGGER.debug(log_str);
        }
        llama_cpp_sys_2::GGML_LOG_LEVEL_ERROR => {
            LLM_LOGGER.error(log_str);
        }
        llama_cpp_sys_2::GGML_LOG_LEVEL_WARN => {
            LLM_LOGGER.warn(log_str);
        }
        llama_cpp_sys_2::GGML_LOG_LEVEL_INFO => {
            LLM_LOGGER.info(log_str);
        }
        llama_cpp_sys_2::GGML_LOG_LEVEL_CONT => {
            LLM_LOGGER.write_raw(log_str);
        }
        _ => {
            LLM_LOGGER.unknown(log_str);
        }
    }
}

pub struct LlamaEngine {
    backend: &'static LlamaBackend,
    model: &'static LlamaModel,
    context: &'static mut LlamaContext<'static>,
    batch: LlamaBatch<'static>,
    cfg: LlamaEngineConfig,
    pos: llama_cpp_sys_2::llama_pos,
}

unsafe impl Send for LlamaEngine {}
unsafe impl Sync for LlamaEngine {}

impl LlamaEngine {
    pub fn load(llama_cfg: LlamaEngineConfig) -> Result<Self> {
        unsafe {
            llama_cpp_sys_2::llama_log_set(Some(llm_engine_log_callback), std::ptr::null_mut());
        }

        if llama_cfg.use_gpu == 0 {
            std::env::set_var("GGML_VULKAN_DISABLE", "1");
            std::env::set_var("CUDA_VISIBLE_DEVICES", "-1");
            std::env::set_var("HIP_VISIBLE_DEVICES", "-1");
            std::env::set_var("ONEAPI_DEVICE_SELECTOR", "opencl:cpu");
        }

        let backend = Box::new(LlamaBackend::init()?);
        let backend_ref: &'static mut LlamaBackend = Box::leak(backend);

        let mut model_params = LlamaModelParams::default();

        if llama_cfg.use_gpu == 1 {
            model_params = model_params.with_n_gpu_layers(llama_cfg.n_gpu_layers);
        } else {
            model_params = model_params.with_n_gpu_layers(0);
        }

        let model = Box::new(
            LlamaModel::load_from_file(
                backend_ref,
                Path::new(&llama_cfg.model_path.clone()),
                &model_params,
            )
            .map_err(|e| anyhow!("模型加载失败: {}", e))?,
        );
        let model_ref: &'static mut LlamaModel = Box::leak(model);

        let mut ctx_params = LlamaContextParams::default();
        ctx_params = ctx_params.with_n_ctx(NonZeroU32::new(llama_cfg.n_ctx));
        ctx_params = ctx_params.with_n_threads(num_cpus::get() as i32);

        let context = Box::new(
            model_ref
                .new_context(backend_ref, ctx_params)
                .map_err(|e| anyhow!("上下文创建失败: {}", e))?,
        );
        let context_ref: &'static mut LlamaContext<'static> = Box::leak(context);

        let batch = LlamaBatch::new(llama_cfg.n_tokens, llama_cfg.n_seq_max);

        Ok(Self {
            backend: backend_ref,
            model: model_ref,
            context: context_ref,
            batch,
            cfg: llama_cfg,
            pos: 0,
        })
    }

    pub fn generate_response<F>(&mut self, new_prompt: &str, callback: F) -> Result<()>
    where
        F: Fn(String),
    {
        LLM_LOGGER.debugf(format_args!(
            "\n {} \n ========================================",
            new_prompt
        ));
        let tokens_list = self
            .model
            .str_to_token(new_prompt, AddBos::Always)
            .map_err(|e| anyhow!("Tokenize 错误: {}", e))?;
        let tokens = tokens_list.to_vec();

        self.batch.clear();
        let last_idx = (tokens.len() as i32) - 1;

        for (i, &t) in tokens.iter().enumerate() {
            self.batch.add(t, self.pos, &[0], i as i32 == last_idx)?;
            self.pos += 1;
        }

        self.context
            .decode(&mut self.batch)
            .map_err(|e| anyhow!("解码失败: {}", e))?;

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(
                self.cfg.penalty_last_n,
                self.cfg.penalty_repeat,
                self.cfg.penalty_freq,
                self.cfg.penalty_present,
            ),
            LlamaSampler::top_p(self.cfg.top_p, self.cfg.min_keep),
            LlamaSampler::temp(self.cfg.temp),
            LlamaSampler::dist(self.cfg.seed),
        ]);

        let mut decoded_count = 0;

        loop {
            let next_token = sampler.sample(&self.context, self.batch.n_tokens() - 1);
            sampler.accept(next_token);

            if self.model.is_eog_token(next_token) || decoded_count >= self.cfg.max_tokens {
                break;
            }

            if let Ok(bytes) =
                self.model
                    .token_to_piece_bytes(next_token, self.cfg.buffer_size, true, None)
            {
                let piece = String::from_utf8_lossy(&bytes).into_owned();
                callback(piece);
            }

            self.batch.clear();
            self.batch.add(next_token, self.pos, &[0], true)?;
            self.context.decode(&mut self.batch)?;

            self.pos += 1;
            decoded_count += 1;
        }

        Ok(())
    }

    pub fn reset_context(&mut self) {
        self.context.clear_kv_cache();
        self.batch.clear();
        self.pos = 0;
    }
}

impl Drop for LlamaEngine {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.context as *mut LlamaContext<'static>);
            let _ = Box::from_raw(self.model as *const LlamaModel as *mut LlamaModel);
            let _ = Box::from_raw(self.backend as *const LlamaBackend as *mut LlamaBackend);
        }
    }
}
