use rand::Rng;
use std::time::Duration;
use tokio::time;
use crate::models::Transaction;
use crate::config::Config;

/// Mock data generator for meme tokens
#[derive(Debug)]
pub struct MockDataGenerator {
    /// Base prices for different tokens
    base_prices: Vec<(String, f64)>,
    /// Price volatility (percentage)
    volatility: f64,
    /// Volume range (min, max)
    volume_range: (f64, f64),
}

impl MockDataGenerator {
    /// Create a new mock data generator
    pub fn new() -> Self {
        Self {
            base_prices: vec![
                ("DOGE".to_string(), 0.15),
                ("SHIB".to_string(), 0.00001),
                ("PEPE".to_string(), 0.000001),
            ],
            volatility: 0.02, // 2% volatility
            volume_range: (100.0, 1000.0),
        }
    }

    /// Create a new mock data generator with configuration
    pub fn new_with_config(config: &Config) -> Self {
        let base_prices = if config.tokens.supported_tokens.is_empty() {
            // Use default tokens if none configured
            vec![
                ("DOGE".to_string(), 0.15),
                ("SHIB".to_string(), 0.00001),
                ("PEPE".to_string(), 0.000001),
            ]
        } else {
            // Use configured tokens
            config.tokens.supported_tokens
                .iter()
                .map(|token| (token.symbol.clone(), token.base_price))
                .collect()
        };

        Self {
            base_prices,
            volatility: config.data_generation.volatility,
            volume_range: config.data_generation.volume_range,
        }
    }

    /// Generate a random transaction for a specific token
    pub fn generate_transaction(&self, token: &str) -> Option<Transaction> {
        // Find base price for the token
        let base_price = self.base_prices
            .iter()
            .find(|(t, _)| t == token)
            .map(|(_, p)| *p)?;

        let mut rng = rand::thread_rng();

        // Generate random price change within volatility range
        let price_change = rng.gen_range(-self.volatility..self.volatility);
        let price = base_price * (1.0 + price_change);

        // Generate random volume
        let volume = rng.gen_range(self.volume_range.0..self.volume_range.1);

        // Randomly decide if it's a buy or sell
        let is_buy = rng.gen_bool(0.5);

        Some(Transaction::new(token.to_string(), price, volume, is_buy))
    }

    /// Generate a random transaction for any available token
    pub fn generate_random_transaction(&self) -> Transaction {
        let mut rng = rand::thread_rng();
        let token_index = rng.gen_range(0..self.base_prices.len());
        let (token, _) = &self.base_prices[token_index];
        
        self.generate_transaction(token).unwrap()
    }

    /// Get all available tokens
    pub fn get_available_tokens(&self) -> Vec<String> {
        self.base_prices.iter().map(|(token, _)| token.clone()).collect()
    }

    /// Start continuous data generation
    pub async fn start_continuous_generation<F>(&self, mut callback: F, interval_ms: u64)
    where
        F: FnMut(Transaction) + Send + 'static,
    {
        let mut interval = time::interval(Duration::from_millis(interval_ms));
        
        loop {
            interval.tick().await;
            
            // Generate transactions for all tokens
            for (token, _) in &self.base_prices {
                if let Some(transaction) = self.generate_transaction(token) {
                    callback(transaction);
                }
            }
        }
    }

    /// Generate historical data for testing
    pub fn generate_historical_data(&self, token: &str, count: usize) -> Vec<Transaction> {
        let mut transactions = Vec::new();
        
        for _ in 0..count {
            if let Some(transaction) = self.generate_transaction(token) {
                transactions.push(transaction);
            }
        }
        
        transactions
    }
}

impl Default for MockDataGenerator {
    fn default() -> Self {
        Self::new()
    }
} 