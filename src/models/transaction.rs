use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Transaction data structure for generating K-lines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Token symbol
    pub token: String,
    /// Transaction price
    pub price: f64,
    /// Transaction volume
    pub volume: f64,
    /// Transaction timestamp
    pub timestamp: DateTime<Utc>,
    /// Whether this is a buy (true) or sell (false)
    pub is_buy: bool,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(token: String, price: f64, volume: f64, is_buy: bool) -> Self {
        Self {
            token,
            price,
            volume,
            timestamp: Utc::now(),
            is_buy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_new() {
        let transaction = Transaction::new("DOGE".to_string(), 1.0, 100.0, true);

        assert_eq!(transaction.token, "DOGE");
        assert_eq!(transaction.price, 1.0);
        assert_eq!(transaction.volume, 100.0);
        assert!(transaction.is_buy);
        assert!(transaction.timestamp <= Utc::now());
        assert!(transaction.timestamp >= Utc::now() - chrono::Duration::seconds(1));
    }
}
