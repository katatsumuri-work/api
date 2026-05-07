use std::env;
use std::error::Error;
use std::fmt;
use std::time::Duration;

const ENV_BIND_ADDR: &str = "API_BIND_ADDR";
const ENV_CORS_ALLOWED_ORIGINS: &str = "API_CORS_ALLOWED_ORIGINS";
const ENV_REQUEST_BODY_LIMIT_BYTES: &str = "API_REQUEST_BODY_LIMIT_BYTES";
const ENV_REQUEST_TIMEOUT_SECS: &str = "API_REQUEST_TIMEOUT_SECS";

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3000";
const DEFAULT_REQUEST_BODY_LIMIT_BYTES: usize = 1024 * 1024;
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr: String,
    pub cors_allowed_origins: Vec<String>,
    pub request_body_limit_bytes: usize,
    pub request_timeout: Duration,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            bind_addr: env::var(ENV_BIND_ADDR).unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string()),
            cors_allowed_origins: parse_csv_env(ENV_CORS_ALLOWED_ORIGINS),
            request_body_limit_bytes: parse_usize_env(
                ENV_REQUEST_BODY_LIMIT_BYTES,
                DEFAULT_REQUEST_BODY_LIMIT_BYTES,
            )?,
            request_timeout: Duration::from_secs(parse_u64_env(
                ENV_REQUEST_TIMEOUT_SECS,
                DEFAULT_REQUEST_TIMEOUT_SECS,
            )?),
        })
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            bind_addr: DEFAULT_BIND_ADDR.to_string(),
            cors_allowed_origins: Vec::new(),
            request_body_limit_bytes: DEFAULT_REQUEST_BODY_LIMIT_BYTES,
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidNumber { var: &'static str, value: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidNumber { var, value } => {
                write!(
                    f,
                    "環境変数 {var} の値 '{value}' を数値としてパースできませんでした"
                )
            }
        }
    }
}

impl Error for ConfigError {}

fn parse_csv_env(name: &str) -> Vec<String> {
    let Ok(raw) = env::var(name) else {
        return Vec::new();
    };
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn parse_usize_env(name: &'static str, default: usize) -> Result<usize, ConfigError> {
    match env::var(name) {
        Ok(value) => value
            .parse::<usize>()
            .map_err(|_| ConfigError::InvalidNumber { var: name, value }),
        Err(_) => Ok(default),
    }
}

fn parse_u64_env(name: &'static str, default: u64) -> Result<u64, ConfigError> {
    match env::var(name) {
        Ok(value) => value
            .parse::<u64>()
            .map_err(|_| ConfigError::InvalidNumber { var: name, value }),
        Err(_) => Ok(default),
    }
}
