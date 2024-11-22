use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub buffer_size: usize,
    pub timeouts: TimeoutConfig,
}

#[derive(Debug, Deserialize)]
pub struct TimeoutConfig {
    pub connection: u64,    // 秒
    pub retry_delay: u64,   // 毫秒
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: String,
    pub rotation: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());
        
        // 获取配置文件的目录路径
        let config_dir = std::env::current_dir()
            .map(|d| d.join("config"))
            .unwrap_or_else(|_| PathBuf::from("config"));

        let s = Config::builder()
            // 首先加载默认配置
            .add_source(File::from(config_dir.join("default.yaml")))
            // 然后加载环境特定的配置
            .add_source(File::from(config_dir.join(format!("{}.yaml", run_mode))).required(false))
            // 最后加载环境变量（可选，格式为 APP_SERVER_HOST 等）
            .add_source(Environment::with_prefix("app").separator("_"))
            .build()?;

        println!("Loading configuration from: {:?}", config_dir);
        s.try_deserialize()
    }

    // 辅助方法，获取连接超时的 Duration
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.server.timeouts.connection)
    }

    // 辅助方法，获取重试延迟的 Duration
    pub fn retry_delay(&self) -> Duration {
        Duration::from_millis(self.server.timeouts.retry_delay)
    }

    // 获取服务器地址字符串
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}
