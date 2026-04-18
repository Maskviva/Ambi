use crate::config::LoggerConfig;

use chrono::Local;
use std::fmt::Arguments;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::ops::Add;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    None,
    Debug,
    Info,
    Warn,
    Error,
    Unknown,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::None => "None",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Unknown => "UNKNOWN",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "None" => Some(LogLevel::None),
            "DEBUG" => Some(LogLevel::Debug),
            "INFO" => Some(LogLevel::Info),
            "WARN" => Some(LogLevel::Warn),
            "ERROR" => Some(LogLevel::Error),
            "UNKNOWN" => Some(LogLevel::Unknown),
            _ => None,
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            LogLevel::None => "\x1b[90m",    // 灰色
            LogLevel::Debug => "\x1b[32m",   // 绿色
            LogLevel::Info => "\x1b[34m",    // 蓝色
            LogLevel::Warn => "\x1b[33m",    // 黄色
            LogLevel::Error => "\x1b[31m",   // 红色
            LogLevel::Unknown => "\x1b[35m", // 品红色
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub struct Logger {
    file: Mutex<BufWriter<File>>,
    need_print: bool,
    prefix: String,
    min_level: LogLevel,
}

impl Logger {
    pub fn new(
        logger_cfg: LoggerConfig,
        log_file_name: &str,
        prefix: &str,
    ) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(logger_cfg.log_files_dir.add(&log_file_name))?;

        Ok(Logger {
            file: Mutex::new(BufWriter::new(file)),
            need_print: logger_cfg.need_print,
            prefix: prefix.to_string(),
            min_level: LogLevel::Info,
        })
    }

    pub fn with_min_level(
        logger_cfg: LoggerConfig,
        min_level: LogLevel,
        log_file_name: &str,
        prefix: &str,
    ) -> std::io::Result<Self> {
        let mut logger = Self::new(logger_cfg, log_file_name, prefix)?;
        logger.min_level = min_level;
        Ok(logger)
    }

    pub fn set_min_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }

    #[allow(dead_code)]
    fn should_log(&self, level: LogLevel) -> bool {
        level >= self.min_level
    }

    fn format_log(&self, level: LogLevel, message: &str) -> String {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        format!(
            "[{}] [{}] [{}] {}\n",
            timestamp,
            self.prefix,
            level.as_str(),
            message.trim_end()
        )
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        let formatted = self.format_log(level, message);

        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(formatted.as_bytes());
        }

        if self.need_print {
            let color = level.color_code();
            let reset = "\x1b[0m";
            let colored_log = format!("{}{}{}", color, formatted.trim_end(), reset);
            println!("{}", colored_log);
        }
    }

    pub fn write_raw(&self, message: &str) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(message.as_bytes());
        }

        if self.need_print {
            print!("{}", message);
            let _ = std::io::stdout().flush();
        }
    }

    #[allow(dead_code)]
    pub fn none(&self, message: &str) {
        self.log(LogLevel::None, message)
    }

    #[allow(dead_code)]
    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message)
    }

    #[allow(dead_code)]
    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message)
    }

    #[allow(dead_code)]
    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message)
    }

    #[allow(dead_code)]
    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message)
    }

    #[allow(dead_code)]
    pub fn unknown(&self, message: &str) {
        self.log(LogLevel::Unknown, message)
    }

    #[allow(dead_code)]
    pub fn nonef(&self, args: Arguments) {
        self.log(LogLevel::None, &args.to_string())
    }

    #[allow(dead_code)]
    pub fn debugf(&self, args: Arguments) {
        self.log(LogLevel::Debug, &args.to_string())
    }

    #[allow(dead_code)]
    pub fn infof(&self, args: Arguments) {
        self.log(LogLevel::Info, &args.to_string())
    }

    #[allow(dead_code)]
    pub fn warnf(&self, args: Arguments) {
        self.log(LogLevel::Warn, &args.to_string())
    }

    #[allow(dead_code)]
    pub fn errorf(&self, args: Arguments) {
        self.log(LogLevel::Error, &args.to_string())
    }

    #[allow(dead_code)]
    pub fn unknownf(&self, args: Arguments) {
        self.log(LogLevel::Unknown, &args.to_string())
    }
}
