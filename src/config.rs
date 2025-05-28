use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Token configuration
    pub tokens: TokensConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Data generation configuration
    pub data_generation: DataGenerationConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Listen address
    pub host: String,
    /// Listen port
    pub port: u16,
    /// Number of worker threads
    pub workers: Option<usize>,
}

/// Token configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    /// Token symbol
    pub symbol: String,
    /// Base price for mock data generation
    pub base_price: f64,
    /// Volatility percentage for mock data generation
    pub volatility: f64,
}

/// Tokens configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokensConfig {
    /// Supported tokens
    pub supported_tokens: Vec<TokenConfig>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Whether to output to file
    pub file_output: bool,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Number of worker threads
    pub worker_threads: usize,
    /// WebSocket heartbeat interval (seconds)
    pub websocket_heartbeat_interval: u64,
    /// Client timeout (seconds)
    pub client_timeout: u64,
    /// K-line data retention time (hours)
    pub kline_retention_hours: u64,
    /// Maximum WebSocket connections
    pub max_websocket_connections: usize,
}

/// Data generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataGenerationConfig {
    /// Whether to enable data generation
    pub enabled: bool,
    /// Generation interval (milliseconds)
    pub interval_ms: u64,
    /// Price volatility (percentage)
    pub volatility: f64,
    /// Volume range
    pub volume_range: (f64, f64),
}

impl Config {
    /// Load configuration from TOML files
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Start with default configuration
        let mut config = Self::load_from_file("config/default.toml")?;

        // Get environment (default to development)
        let env = env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());

        // Try to load environment-specific configuration
        let env_config_path = format!("config/{}.toml", env);
        if Path::new(&env_config_path).exists() {
            let env_config = Self::load_from_file(&env_config_path)?;
            config = config.merge_with(env_config);
        }

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from a specific TOML file
    fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Merge this configuration with another (other takes precedence)
    fn merge_with(mut self, other: Config) -> Self {
        // Simple field-by-field merge
        if other.server.host != self.server.host {
            self.server.host = other.server.host;
        }
        if other.server.port != self.server.port {
            self.server.port = other.server.port;
        }
        if other.server.workers.is_some() {
            self.server.workers = other.server.workers;
        }

        // Merge other sections as needed
        if !other.tokens.supported_tokens.is_empty() {
            self.tokens = other.tokens;
        }

        self.logging = other.logging;
        self.performance = other.performance;
        self.data_generation = other.data_generation;

        self
    }

    /// Validate configuration values
    fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port must be greater than 0".to_string());
        }

        if self.data_generation.volatility < 0.0 || self.data_generation.volatility > 1.0 {
            return Err("Volatility must be between 0.0 and 1.0".to_string());
        }

        if self.data_generation.volume_range.0 >= self.data_generation.volume_range.1 {
            return Err("Volume range minimum must be less than maximum".to_string());
        }

        Ok(())
    }

    /// Get list of supported token symbols
    pub fn get_supported_tokens(&self) -> Vec<String> {
        self.tokens
            .supported_tokens
            .iter()
            .map(|token| token.symbol.clone())
            .collect()
    }

    /// Get token information by symbol
    pub fn get_token_info(&self, symbol: &str) -> Option<&TokenConfig> {
        self.tokens
            .supported_tokens
            .iter()
            .find(|token| token.symbol == symbol)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: None,
            },
            tokens: TokensConfig {
                supported_tokens: vec![
                    TokenConfig {
                        symbol: "DOGE".to_string(),
                        base_price: 0.15,
                        volatility: 5.0,
                    },
                    TokenConfig {
                        symbol: "SHIB".to_string(),
                        base_price: 0.00005,
                        volatility: 8.0,
                    },
                    TokenConfig {
                        symbol: "PEPE".to_string(),
                        base_price: 0.000008,
                        volatility: 10.0,
                    },
                ],
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_output: false,
            },
            performance: PerformanceConfig {
                worker_threads: 4,
                websocket_heartbeat_interval: 5,
                client_timeout: 10,
                kline_retention_hours: 24,
                max_websocket_connections: 1000,
            },
            data_generation: DataGenerationConfig {
                enabled: true,
                interval_ms: 100,
                volatility: 0.02,
                volume_range: (100.0, 1000.0),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.data_generation.interval_ms, 100);
        assert_eq!(config.data_generation.volatility, 0.02);
        assert!(config.data_generation.enabled);
        assert_eq!(config.tokens.supported_tokens.len(), 3);
    }

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = Config::default();
        invalid_config.server.port = 0;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_token_methods() {
        let config = Config::default();
        let tokens = config.get_supported_tokens();
        assert!(tokens.contains(&"DOGE".to_string()));
        assert!(tokens.contains(&"SHIB".to_string()));
        assert!(tokens.contains(&"PEPE".to_string()));

        let doge_info = config.get_token_info("DOGE");
        assert!(doge_info.is_some());
        assert_eq!(doge_info.unwrap().base_price, 0.15);
    }
}
