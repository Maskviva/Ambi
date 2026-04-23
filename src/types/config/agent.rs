use crate::llm::ChatTemplate;

#[derive(Clone, Debug)]
pub struct AgentConfig {
    pub system_prompt: String,
    pub template: ChatTemplate,
    pub max_iterations: usize,
    pub enable_formatting: bool,

    /// (initial_keep, recent_keep, max_safe_tokens)
    pub eviction_strategy: (usize, usize, usize),
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            template: ChatTemplate::chatml(),
            max_iterations: 10,
            enable_formatting: false,
            eviction_strategy: (2, 6, 3000),
        }
    }
}
