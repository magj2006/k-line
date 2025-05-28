use super::time_interval::TimeInterval;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// K-line (candlestick) data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KLine {
    /// Token symbol (e.g., "DOGE", "SHIB")
    pub token: String,
    /// Timestamp for the start of this interval
    pub timestamp: DateTime<Utc>,
    /// Time interval
    pub interval: TimeInterval,
    /// Opening price
    pub open: f64,
    /// Highest price in this interval
    pub high: f64,
    /// Lowest price in this interval
    pub low: f64,
    /// Closing price (current price for open intervals)
    pub close: f64,
    /// Trading volume
    pub volume: f64,
    /// Whether this K-line is closed (interval completed)
    pub is_closed: bool,
}

impl KLine {
    /// Create a new K-line from initial price
    pub fn new(
        token: String,
        timestamp: DateTime<Utc>,
        interval: TimeInterval,
        price: f64,
        volume: f64,
    ) -> Self {
        Self {
            token,
            timestamp,
            interval,
            open: price,
            high: price,
            low: price,
            close: price,
            volume,
            is_closed: false,
        }
    }

    /// Update K-line with new price and volume
    pub fn update(&mut self, price: f64, volume: f64) {
        if !self.is_closed {
            self.high = self.high.max(price);
            self.low = self.low.min(price);
            self.close = price;
            self.volume += volume;
        }
    }

    /// Close this K-line (mark as completed)
    pub fn close(&mut self) {
        self.is_closed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_interval_as_str() {
        assert_eq!(TimeInterval::Second1.as_str(), "1s");
        assert_eq!(TimeInterval::Minute1.as_str(), "1m");
        assert_eq!(TimeInterval::Minute5.as_str(), "5m");
        assert_eq!(TimeInterval::Minute15.as_str(), "15m");
        assert_eq!(TimeInterval::Hour1.as_str(), "1h");
    }

    #[test]
    fn test_time_interval_from_str() {
        assert_eq!("1s".parse::<TimeInterval>(), Ok(TimeInterval::Second1));
        assert_eq!("1m".parse::<TimeInterval>(), Ok(TimeInterval::Minute1));
        assert_eq!("5m".parse::<TimeInterval>(), Ok(TimeInterval::Minute5));
        assert_eq!("15m".parse::<TimeInterval>(), Ok(TimeInterval::Minute15));
        assert_eq!("1h".parse::<TimeInterval>(), Ok(TimeInterval::Hour1));
        assert_eq!(
            "invalid".parse::<TimeInterval>(),
            Err(String::from("Invalid time interval: invalid"))
        );
    }

    #[test]
    fn test_kline_new() {
        let now = Utc::now();
        let kline = KLine::new("DOGE".to_string(), now, TimeInterval::Minute1, 1.0, 100.0);

        assert_eq!(kline.token, "DOGE");
        assert_eq!(kline.timestamp, now);
        assert_eq!(kline.interval, TimeInterval::Minute1);
        assert_eq!(kline.open, 1.0);
        assert_eq!(kline.high, 1.0);
        assert_eq!(kline.low, 1.0);
        assert_eq!(kline.close, 1.0);
        assert_eq!(kline.volume, 100.0);
        assert!(!kline.is_closed);
    }
}
