use std::time::Duration;
use tokio::time;
use rand::Rng;
use chrono::Utc;

use crate::models::transaction::Transaction;

/// Service for generating mock transaction data
pub struct MockService {
    /// Base price for each token
    base_prices: Vec<(String, f64)>,
    /// Price volatility (percentage)
    volatility: f64,
    /// Volume range (min, max)
    volume_range: (f64, f64),
}

impl MockService {
    /// Create a new MockService
    pub fn new() -> Self {
        Self {
            base_prices: vec![
                ("DOGE".to_string(), 0.15),
                ("SHIB".to_string(), 0.00001),
            ],
            volatility: 0.02, // 2% volatility
            volume_range: (100.0, 1000.0),
        }
    }

    /// Generate a random transaction for a token
    pub fn generate_transaction(&self, token: &str) -> Option<Transaction> {
        // Find base price for token
        let base_price = self.base_prices
            .iter()
            .find(|(t, _)| t == token)
            .map(|(_, p)| *p)?;

        // Generate random price change
        let mut rng = rand::thread_rng();
        let price_change = rng.gen_range(-self.volatility..self.volatility);
        let price = base_price * (1.0 + price_change);

        // Generate random volume
        let volume = rng.gen_range(self.volume_range.0..self.volume_range.1);

        // Randomly decide if it's a buy or sell
        let is_buy = rng.gen_bool(0.5);

        Some(Transaction::new(token.to_string(), price, volume, is_buy))
    }

    /// Start generating mock transactions
    pub async fn start_generation(&self, kline_service: &crate::services::kline_service::KLineService) {
        let mut interval = time::interval(Duration::from_millis(100)); // 10 transactions per second

        loop {
            interval.tick().await;
            
            // Generate transactions for each token
            for (token, _) in &self.base_prices {
                if let Some(transaction) = self.generate_transaction(token) {
                    kline_service.process_transaction(&transaction);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_transaction() {
        let service = MockService::new();
        
        // Test DOGE token
        let transaction = service.generate_transaction("DOGE").unwrap();
        assert_eq!(transaction.token, "DOGE");
        assert!(transaction.price > 0.0);
        assert!(transaction.volume >= 100.0 && transaction.volume <= 1000.0);
        
        // Test SHIB token
        let transaction = service.generate_transaction("SHIB").unwrap();
        assert_eq!(transaction.token, "SHIB");
        assert!(transaction.price > 0.0);
        assert!(transaction.volume >= 100.0 && transaction.volume <= 1000.0);
        
        // Test invalid token
        assert!(service.generate_transaction("INVALID").is_none());
    }
} 