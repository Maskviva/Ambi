use anyhow::{anyhow, Result};
use std::ffi::CStr;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use tokio::sync::oneshot;

use crate::llm::engine::llama_cpp_2::llama_cpp_2_config::LlamaEngineConfig;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::token::LlamaToken;
use llama_cpp_sys_2;
use log::{debug, error, info, trace, warn};

extern "C" fn llm_engine_log_callback(
    level: llama_cpp_sys_2::ggml_log_level,
    text: *const std::os::raw::c_char,
    _data: *mut std::os::raw::c_void,
) {
    let text = unsafe { CStr::from_ptr(text) };
    let log_str = text.to_string_lossy();

    let clean_str = log_str.trim_end();

    match level {
        llama_cpp_sys_2::GGML_LOG_LEVEL_DEBUG => debug!("{}", clean_str),
        llama_cpp_sys_2::GGML_LOG_LEVEL_ERROR => error!("{}", clean_str),
        llama_cpp_sys_2::GGML_LOG_LEVEL_WARN => warn!("{}", clean_str),
        llama_cpp_sys_2::GGML_LOG_LEVEL_INFO => info!("{}", clean_str),
        llama_cpp_sys_2::GGML_LOG_LEVEL_CONT => trace!("{}", log_str),
        _ => {}
    }
}

enum LlamaCommand {
    Chat {
        prompt: String,
        reply_tx: oneshot::Sender<Result<String>>,
    },
    ChatStream {
        prompt: String,
        chunk_tx: tokio::sync::mpsc::Sender<String>,
        done_tx: oneshot::Sender<()>,
    },
    Reset,
}

pub struct LlamaEngine {
    cmd_tx: mpsc::Sender<LlamaCommand>,
}
impl LlamaEngine {
    pub fn load(llama_cfg: LlamaEngineConfig) -> Result<Self> {
        unsafe {
            llama_cpp_sys_2::llama_log_set(Some(llm_engine_log_callback), std::ptr::null_mut());
        }

        let (init_tx, init_rx) = mpsc::channel();
        let (cmd_tx, cmd_rx) = mpsc::channel::<LlamaCommand>();

        thread::spawn(move || {
            let backend = match LlamaBackend::init() {
                Ok(b) => b,
                Err(e) => {
                    let _ = init_tx.send(Err(anyhow!("Backend init failed: {}", e)));
                    return;
                }
            };

            let mut model_params = LlamaModelParams::default();
            if llama_cfg.use_gpu {
                model_params = model_params.with_n_gpu_layers(llama_cfg.n_gpu_layers);
            } else {
                model_params = model_params.with_n_gpu_layers(0);
            }

            let model = match LlamaModel::load_from_file(
                &backend,
                Path::new(&llama_cfg.model_path),
                &model_params,
            ) {
                Ok(m) => m,
                Err(e) => {
                    let _ = init_tx.send(Err(anyhow!("Model loading failed: {}", e)));
                    return;
                }
            };

            let mut ctx_params = LlamaContextParams::default();
            ctx_params = ctx_params.with_n_ctx(NonZeroU32::new(llama_cfg.n_ctx));
            ctx_params = ctx_params.with_n_threads(num_cpus::get() as i32);

            let mut context = match model.new_context(&backend, ctx_params) {
                Ok(c) => c,
                Err(e) => {
                    let _ = init_tx.send(Err(anyhow!("Failed to create context: {}", e)));
                    return;
                }
            };

            let mut batch = LlamaBatch::new(llama_cfg.n_tokens, llama_cfg.n_seq_max);
            let mut history_tokens: Vec<LlamaToken> = Vec::new();
            let mut utf8_buffer: Vec<u8> = Vec::with_capacity(32);
            let mut pos: i32 = 0;

            if init_tx.send(Ok(())).is_err() {
                return;
            }

            while let Ok(cmd) = cmd_rx.recv() {
                match cmd {
                    LlamaCommand::Chat { prompt, reply_tx } => {
                        let mut full_response = String::new();
                        let res = Self::run_inference(
                            &prompt,
                            &model,
                            &mut context,
                            &mut batch,
                            &llama_cfg,
                            &mut pos,
                            &mut history_tokens,
                            &mut utf8_buffer,
                            |piece| {
                                full_response.push_str(&piece);
                            },
                        );
                        let _ = reply_tx.send(res.map(|_| full_response));
                    }
                    LlamaCommand::ChatStream {
                        prompt,
                        chunk_tx,
                        done_tx,
                    } => {
                        let _res = Self::run_inference(
                            &prompt,
                            &model,
                            &mut context,
                            &mut batch,
                            &llama_cfg,
                            &mut pos,
                            &mut history_tokens,
                            &mut utf8_buffer,
                            |piece| {
                                let _ = chunk_tx.blocking_send(piece);
                            },
                        );
                        let _ = done_tx.send(());
                    }
                    LlamaCommand::Reset => {
                        context.clear_kv_cache();
                        history_tokens.clear();
                        utf8_buffer.clear();
                        batch.clear();
                        pos = 0;
                    }
                }
            }
        });

        match init_rx.recv() {
            Ok(Ok(_)) => Ok(Self { cmd_tx }),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow!("Engine initialization thread panicked")),
        }
    }

    pub async fn chat_internal(&self, prompt: &str) -> Result<String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.cmd_tx
            .send(LlamaCommand::Chat {
                prompt: prompt.to_string(),
                reply_tx,
            })
            .map_err(|_| anyhow!("Llama Engine thread died unexpectedly"))?;
        reply_rx
            .await
            .map_err(|_| anyhow!("Reply channel closed prematurely"))?
    }

    pub async fn stream_internal(&self, prompt: &str, tx: tokio::sync::mpsc::Sender<String>) {
        let (done_tx, done_rx) = oneshot::channel();
        if self
            .cmd_tx
            .send(LlamaCommand::ChatStream {
                prompt: prompt.to_string(),
                chunk_tx: tx,
                done_tx,
            })
            .is_err()
        {
            error!("Llama Engine thread died unexpectedly");
            return;
        }
        let _ = done_rx.await;
    }

    pub fn reset_internal(&self) {
        let _ = self.cmd_tx.send(LlamaCommand::Reset);
    }

    fn run_inference<F>(
        prompt: &str,
        model: &LlamaModel,
        context: &mut LlamaContext,
        batch: &mut LlamaBatch,
        cfg: &LlamaEngineConfig,
        pos: &mut i32,
        history_tokens: &mut Vec<LlamaToken>,
        utf8_buffer: &mut Vec<u8>,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(String),
    {
        debug!("\n {} \n ========================================", prompt);
        let tokens_list = model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| anyhow!("Tokenize failed: {}", e))?;
        let current_tokens = tokens_list.to_vec();

        let mut match_len = 0;
        for (t1, t2) in history_tokens.iter().zip(current_tokens.iter()) {
            if t1 == t2 {
                match_len += 1;
            } else {
                break;
            }
        }

        if current_tokens.len() >= cfg.n_ctx as usize || match_len < history_tokens.len() {
            context.clear_kv_cache();
            history_tokens.clear();
            *pos = 0;
            match_len = 0;
        }

        *pos = match_len as i32;
        let new_tokens = &current_tokens[match_len..];

        batch.clear();
        let last_idx = (new_tokens.len() as i32) - 1;

        for (i, &t) in new_tokens.iter().enumerate() {
            batch.add(t, *pos, &[0], i as i32 == last_idx)?;
            *pos += 1;
        }

        if !new_tokens.is_empty() {
            context
                .decode(batch)
                .map_err(|e| anyhow!("Decoding failed: {}", e))?;
        }

        *history_tokens = current_tokens.clone();

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(
                cfg.penalty_last_n,
                cfg.penalty_repeat,
                cfg.penalty_freq,
                cfg.penalty_present,
            ),
            LlamaSampler::top_p(cfg.top_p, cfg.min_keep),
            LlamaSampler::temp(cfg.temp),
            LlamaSampler::dist(cfg.seed),
        ]);

        let mut decoded_count = 0;

        loop {
            let next_token = sampler.sample(context, batch.n_tokens() - 1);
            sampler.accept(next_token);

            if model.is_eog_token(next_token) || decoded_count >= cfg.max_tokens {
                break;
            }

            history_tokens.push(next_token);

            if let Ok(bytes) = model.token_to_piece_bytes(next_token, cfg.buffer_size, true, None) {
                utf8_buffer.extend_from_slice(&bytes);
                match std::str::from_utf8(utf8_buffer) {
                    Ok(valid_str) => {
                        callback(valid_str.to_string());
                        utf8_buffer.clear();
                    }
                    Err(e) => {
                        let valid_len = e.valid_up_to();
                        if valid_len > 0 {
                            let valid_str =
                                unsafe { std::str::from_utf8_unchecked(&utf8_buffer[..valid_len]) };
                            callback(valid_str.to_string());
                            utf8_buffer.drain(..valid_len);
                        }
                    }
                }
            }

            batch.clear();
            batch.add(next_token, *pos, &[0], true)?;
            context.decode(batch)?;

            *pos += 1;
            decoded_count += 1;
        }
        Ok(())
    }
}
