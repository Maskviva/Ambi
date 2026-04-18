use crate::config::AppConfig;
use once_cell::sync::Lazy;
use simplelog::*;
use std::fs::OpenOptions;
use std::path::Path;

pub static APP_CONFIG: Lazy<AppConfig> =
    Lazy::new(|| AppConfig::load().expect("无法加载配置文件 config.toml"));

pub fn init_logger(config: &crate::config::LoggerConfig) {
    let dir = Path::new(&config.log_files_dir);

    let agent_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("agent.txt"))
        .unwrap();
    let llm_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("llm.txt"))
        .unwrap();

    let llm_config = ConfigBuilder::new()
        .add_filter_allow_str("Ambi::core::llm")
        .build();

    let agent_config = ConfigBuilder::new()
        .add_filter_allow_str("Ambi::core::agent")
        .add_filter_allow_str("Ambi::core::tool")
        .add_filter_allow_str("Ambi::tools")
        .build();

    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![
        WriteLogger::new(LevelFilter::Debug, agent_config, agent_file),
        WriteLogger::new(LevelFilter::Debug, llm_config, llm_file),
    ];

    if config.need_print {
        loggers.push(TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ));
    }

    CombinedLogger::init(loggers).expect("全局日志初始化失败");
}
