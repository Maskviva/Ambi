#![cfg(feature = "llama-cpp")]

use crate::llm::{LLMEngineTrait, LLMRequest};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
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
use serde::Deserialize;
use std::ffi::CStr;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

/// Configuration settings for the local `llama.cpp` inference engine.
///
/// This struct defines the hardware acceleration, context windows, sampling penalties,
/// and memory allocation parameters required to initialize a local GGUF model via
/// the `llama-cpp-2` bindings.
///
/// # Examples
///
/// ```rust
/// use ambi::llm::providers::llama_cpp::LlamaEngineConfig;
///
/// let config = LlamaEngineConfig {
///     model_path: "./models/llama-3-8b.gguf".to_string(),
///     max_tokens: 4096,
///     buffer_size: 32,
///     use_gpu: true,
///     n_gpu_layers: 100, // Offload all layers to GPU
///     n_ctx: 8192,
///     n_tokens: 512,
///     n_seq_max: 1,
///     penalty_last_n: 64,
///     penalty_repeat: 1.1,
///     penalty_freq: 0.0,
///     penalty_present: 0.0,
///     temp: 0.7,
///     top_p: 0.9,
///     seed: 42,
///     min_keep: 1,
/// };
/// ```
#[derive(Debug, Deserialize, Clone)]
pub struct LlamaEngineConfig {
    pub model_path: String,
    pub max_tokens: i32,
    pub buffer_size: usize,
    pub use_gpu: bool,
    pub n_gpu_layers: u32,
    pub n_ctx: u32,
    pub n_tokens: usize,
    pub n_seq_max: i32,
    pub penalty_last_n: i32,
    pub penalty_repeat: f32,
    pub penalty_freq: f32,
    pub penalty_present: f32,
    pub temp: f32,
    pub top_p: f32,
    pub seed: u32,
    pub min_keep: usize,
}

impl LlamaEngineConfig {
    pub fn validate(&self) -> Result<()> {
        if !Path::new(&self.model_path).exists() {
            return Err(anyhow!(
                "Local model file does not exist: {}",
                self.model_path
            ));
        }
        if self.n_ctx == 0 {
            return Err(anyhow!("Context n_ctx cannot be 0."));
        }
        if self.temp < 0.0 || self.temp > 2.0 {
            return Err(anyhow!("Temperature must be between 0.0 and 2.0"));
        }
        Ok(())
    }
}

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
        chunk_tx: Sender<Result<String, anyhow::Error>>,
        done_tx: oneshot::Sender<()>,
    },
    Reset,
    EvaluateEntropy {
        sentence: String,
        reply_tx: oneshot::Sender<Result<f32>>,
    },
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
                                true
                            },
                        );
                        let _ = reply_tx.send(res.map(|_| full_response));
                    }
                    LlamaCommand::ChatStream {
                        prompt,
                        chunk_tx,
                        done_tx,
                    } => {
                        let res = Self::run_inference(
                            &prompt,
                            &model,
                            &mut context,
                            &mut batch,
                            &llama_cfg,
                            &mut pos,
                            &mut history_tokens,
                            &mut utf8_buffer,
                            |piece| chunk_tx.blocking_send(Ok(piece)).is_ok(),
                        );
                        if let Err(e) = res {
                            let _ = chunk_tx.blocking_send(Err(e));
                        }
                        let _ = done_tx.send(());
                    }
                    LlamaCommand::Reset => {
                        context.clear_kv_cache();
                        history_tokens.clear();
                        utf8_buffer.clear();
                        batch.clear();
                        pos = 0;
                    }
                    LlamaCommand::EvaluateEntropy { sentence, reply_tx } => {
                        let res = Self::run_evaluate_entropy(
                            &sentence,
                            &model,
                            &mut context,
                            &mut batch,
                            &mut pos,
                        );
                        let _ = reply_tx.send(res);
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

    pub async fn stream_internal(&self, prompt: &str, tx: Sender<Result<String, anyhow::Error>>) {
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
        F: FnMut(String) -> bool,
    {
        debug!("\n{}\n========================================", prompt);
        let tokens_list = model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| anyhow!("Tokenize failed: {}", e))?;
        let current_tokens = tokens_list.to_vec();

        if current_tokens.len() >= cfg.n_ctx as usize {
            return Err(anyhow!(
                "Prompt size ({}) strictly exceeds configured n_ctx limit ({}).",
                current_tokens.len(),
                cfg.n_ctx
            ));
        }

        let dynamic_max_tokens = std::cmp::min(
            cfg.max_tokens as usize,
            (cfg.n_ctx as usize).saturating_sub(current_tokens.len()),
        );

        if dynamic_max_tokens < 32 {
            return Err(anyhow!(
                "Insufficient token space left for generation (only {} tokens). Force context eviction required.",
                dynamic_max_tokens
            ));
        }

        let mut match_len = 0;
        for (t1, t2) in history_tokens.iter().zip(current_tokens.iter()) {
            if t1 == t2 {
                match_len += 1;
            } else {
                break;
            }
        }

        if match_len < history_tokens.len() {
            let evicted_len = history_tokens.len() - match_len;
            info!(
                "Evicting {} tokens, applying KV Cache Shift to save Eval cost.",
                evicted_len
            );

            let p0 = match_len as u32;
            let p1 = (match_len + evicted_len) as u32;

            if let Err(e) = context.clear_kv_cache_seq(Some(0), Some(p0), Some(p1)) {
                warn!("Failed to cleanly remove KV cache sequence: {}", e);
                context.clear_kv_cache();
                match_len = 0;
            } else {
                if let Err(e) = context.kv_cache_seq_add(0, Some(p1), None, -(evicted_len as i32)) {
                    warn!(
                        "Failed to shift KV cache sequence: {}. Falling back to full reset.",
                        e
                    );
                    context.clear_kv_cache();
                    match_len = 0;
                } else {
                    match_len = history_tokens.len() - evicted_len;
                }
            }
        }

        *pos = match_len as i32;
        let new_tokens = &current_tokens[match_len..];

        let chunk_size = cfg.n_tokens;
        let total_new_tokens = new_tokens.len();
        let mut processed = 0;

        for chunk in new_tokens.chunks(chunk_size) {
            batch.clear();

            for &t in chunk.iter() {
                processed += 1;
                let is_absolute_last = processed == total_new_tokens;
                batch.add(t, *pos, &[0], is_absolute_last)?;
                *pos += 1;
            }

            if !chunk.is_empty() {
                context
                    .decode(batch)
                    .map_err(|e| anyhow::anyhow!("Decoding failed: {}", e))?;
            }
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

            if model.is_eog_token(next_token) || decoded_count >= dynamic_max_tokens {
                break;
            }

            history_tokens.push(next_token);

            if let Ok(bytes) = model.token_to_piece_bytes(next_token, cfg.buffer_size, true, None) {
                utf8_buffer.extend_from_slice(&bytes);
                match std::str::from_utf8(utf8_buffer) {
                    Ok(valid_str) => {
                        if !callback(valid_str.to_string()) {
                            break;
                        }
                        utf8_buffer.clear();
                    }
                    Err(e) => {
                        let valid_len = e.valid_up_to();
                        if valid_len > 0 {
                            let valid_str =
                                unsafe { std::str::from_utf8_unchecked(&utf8_buffer[..valid_len]) };
                            if !callback(valid_str.to_string()) {
                                break;
                            }
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

    pub async fn evaluate_sentence_entropy_internal(&self, sentence: &str) -> Result<f32> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.cmd_tx
            .send(LlamaCommand::EvaluateEntropy {
                sentence: sentence.to_string(),
                reply_tx,
            })
            .map_err(|_| anyhow!("Llama Engine thread died unexpectedly"))?;
        reply_rx
            .await
            .map_err(|_| anyhow!("Reply channel closed prematurely"))?
    }

    fn run_evaluate_entropy(
        sentence: &str,
        model: &LlamaModel,
        context: &mut LlamaContext,
        batch: &mut LlamaBatch,
        pos: &mut i32,
    ) -> Result<f32> {
        let tokens_list = model
            .str_to_token(sentence, AddBos::Always)
            .map_err(|e| anyhow!("Tokenize failed: {}", e))?;
        let tokens = tokens_list.to_vec();

        if tokens.is_empty() {
            return Ok(0.0);
        }

        batch.clear();
        for (i, &t) in tokens.iter().enumerate() {
            batch.add(t, *pos + i as i32, &[0], true)?;
        }

        context
            .decode(batch)
            .map_err(|e| anyhow::anyhow!("Decoding failed: {}", e))?;

        let mut total_entropy = 0.0;

        for i in 0..tokens.len() {
            let logits_slice = context.get_logits_ith(i as i32);

            total_entropy += calculate_token_entropy(logits_slice);
        }

        *pos += tokens.len() as i32;

        Ok(total_entropy / tokens.len() as f32)
    }
}

#[async_trait]
impl LLMEngineTrait for LlamaEngine {
    async fn chat(&mut self, request: LLMRequest) -> Result<String> {
        self.chat_internal(&request.formatted_prompt).await
    }

    async fn chat_stream(
        &mut self,
        request: LLMRequest,
        tx: Sender<Result<String, anyhow::Error>>,
    ) {
        self.stream_internal(&request.formatted_prompt, tx).await;
    }

    fn reset_context(&mut self) {
        self.reset_internal();
    }

    async fn evaluate_sentence_entropy(&mut self, sentence: &str) -> Result<f32> {
        self.evaluate_sentence_entropy_internal(sentence).await
    }
}

fn calculate_token_entropy(logits: &[f32]) -> f32 {
    let max_logit = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut sum_exp = 0.0;
    let mut exps = Vec::with_capacity(logits.len());
    for &l in logits {
        let exp = (l - max_logit).exp();
        exps.push(exp);
        sum_exp += exp;
    }
    let mut entropy = 0.0;
    for exp in exps {
        let p = exp / sum_exp;
        if p > 1e-7 {
            entropy -= p * p.ln();
        }
    }
    entropy
}
