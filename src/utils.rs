pub mod logger;

use crate::config::AppConfig;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static APP_CONFIG:  Lazy<AppConfig> =
    Lazy::new(|| AppConfig::load().expect("无法加载配置文件 config.toml"));

pub static AGENT_LOGGER: Lazy<Arc<logger::Logger>> = Lazy::new(|| {
    Arc::new(
        logger::Logger::new(APP_CONFIG.logger.clone(), "agent.txt", "Agent")
            .expect("Agent 日志管理器初始化失败"),
    )
});

pub static LLM_LOGGER: Lazy<Arc<logger::Logger>> = Lazy::new(|| {
    Arc::new(
        logger::Logger::new(APP_CONFIG.logger.clone(), "llm.txt", "LLM")
            .expect("LLM 日志管理器初始化失败"),
    )
});
