use crate::core::llm::{LlamaEngineConfig, OpenAIEngineConfig};

use config::{Config, ConfigError, File};
use serde::Deserialize;
use crate::core::llm::chat_template::ChatTemplateType;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub agent: AgentConfig,
    pub logger: LoggerConfig,
    pub llm_engine_config: LlamaEngineConfig,
    pub open_ai_engine_config: OpenAIEngineConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggerConfig {
    pub log_files_dir: String,
    pub need_print: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AgentConfig {
    #[serde(default)]
    pub template: ChatTemplateType,
    pub system_prompt: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config"))
            .add_source(config::Environment::with_prefix("APP"))
            .build()?;

        s.try_deserialize()
    }
}
