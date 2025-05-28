use crate::models::{KLine, TimeInterval, Transaction};
use chrono::{DateTime, Duration, Timelike, Utc};
use dashmap::DashMap;

/// K-line data service using DashMap for high-performance concurrent access
#[derive(Debug)]
pub struct KLineService {
    /// Storage for K-lines: token -> interval -> timestamp -> KLine
    /// Using DashMap for lock-free concurrent access
    klines: DashMap<String, DashMap<TimeInterval, DashMap<DateTime<Utc>, KLine>>>,
}

impl KLineService {
    /// Create a new K-line service
    pub fn new() -> Self {
        Self {
            klines: DashMap::new(),
        }
    }

    /// Process a transaction and update K-lines
    pub fn process_transaction(&self, transaction: &Transaction) {
        // Update K-lines for all supported intervals
        for interval in [
            TimeInterval::Second1,
            TimeInterval::Minute1,
            TimeInterval::Minute5,
            TimeInterval::Minute15,
            TimeInterval::Hour1,
        ] {
            self.update_kline_for_interval(transaction, interval);
        }
    }

    /// Update K-line for a specific interval
    fn update_kline_for_interval(&self, transaction: &Transaction, interval: TimeInterval) {
        let interval_start = self.get_interval_start(transaction.timestamp, interval);

        // Get or create token-level map
        let token_klines = self.klines.entry(transaction.token.clone()).or_default();

        // Get or create interval-level map
        let interval_klines = token_klines.entry(interval).or_default();

        // Close expired K-lines before updating
        self.close_expired_klines(&interval_klines, interval_start, interval);

        // Update or create K-line for this interval
        interval_klines
            .entry(interval_start)
            .and_modify(|kline| {
                kline.update(transaction.price, transaction.volume);
            })
            .or_insert_with(|| {
                KLine::new(
                    transaction.token.clone(),
                    interval_start,
                    interval,
                    transaction.price,
                    transaction.volume,
                )
            });
    }

    /// Close K-lines that have expired (interval has passed)
    fn close_expired_klines(
        &self,
        interval_klines: &DashMap<DateTime<Utc>, KLine>,
        current_interval_start: DateTime<Utc>,
        interval: TimeInterval,
    ) {
        let interval_duration = Duration::seconds(interval.duration_seconds() as i64);

        // Iterate through all K-lines and close expired ones
        for mut kline_ref in interval_klines.iter_mut() {
            let kline = kline_ref.value_mut();
            if kline.timestamp + interval_duration <= current_interval_start && !kline.is_closed {
                kline.close();
            }
        }
    }

    /// Get the start timestamp for an interval
    fn get_interval_start(
        &self,
        timestamp: DateTime<Utc>,
        interval: TimeInterval,
    ) -> DateTime<Utc> {
        match interval {
            TimeInterval::Second1 => {
                // Align to second: remove nanoseconds
                timestamp.with_nanosecond(0).unwrap_or(timestamp)
            }
            TimeInterval::Minute1 => {
                // Align to minute: remove seconds and nanoseconds
                timestamp
                    .with_second(0)
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap_or(timestamp)
            }
            TimeInterval::Minute5 => {
                // Align to 5-minute: round minutes to multiple of 5
                let minute = timestamp.minute();
                let aligned_minute = (minute / 5) * 5;
                timestamp
                    .with_minute(aligned_minute)
                    .and_then(|t| t.with_second(0))
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap_or(timestamp)
            }
            TimeInterval::Minute15 => {
                // Align to 15-minute: round minutes to multiple of 15
                let minute = timestamp.minute();
                let aligned_minute = (minute / 15) * 15;
                timestamp
                    .with_minute(aligned_minute)
                    .and_then(|t| t.with_second(0))
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap_or(timestamp)
            }
            TimeInterval::Hour1 => {
                // Align to hour: remove minutes, seconds and nanoseconds
                timestamp
                    .with_minute(0)
                    .and_then(|t| t.with_second(0))
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap_or(timestamp)
            }
        }
    }

    /// Get K-lines for a token and interval within a time range
    pub fn get_klines(
        &self,
        token: &str,
        interval: TimeInterval,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: Option<usize>,
    ) -> Vec<KLine> {
        let mut result = Vec::new();

        if let Some(token_klines) = self.klines.get(token) {
            if let Some(interval_klines) = token_klines.get(&interval) {
                for kline_ref in interval_klines.iter() {
                    let (timestamp, kline) = kline_ref.pair();
                    if *timestamp >= start && *timestamp <= end {
                        result.push(kline.clone());
                    }
                }
            }
        }

        // Sort by timestamp
        result.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Apply limit if specified
        if let Some(limit) = limit {
            result.truncate(limit);
        }

        result
    }

    /// Get the latest K-line for a token and interval
    pub fn get_latest_kline(&self, token: &str, interval: TimeInterval) -> Option<KLine> {
        if let Some(token_klines) = self.klines.get(token) {
            if let Some(interval_klines) = token_klines.get(&interval) {
                // Find the most recent K-line
                interval_klines
                    .iter()
                    .map(|kline_ref| kline_ref.value().clone())
                    .max_by_key(|kline| kline.timestamp)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get all available tokens
    pub fn get_available_tokens(&self) -> Vec<String> {
        self.klines
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get current open K-line for a token and interval
    pub fn get_current_kline(&self, token: &str, interval: TimeInterval) -> Option<KLine> {
        if let Some(token_klines) = self.klines.get(token) {
            if let Some(interval_klines) = token_klines.get(&interval) {
                // Find the most recent open K-line
                interval_klines
                    .iter()
                    .map(|kline_ref| kline_ref.value().clone())
                    .filter(|kline| !kline.is_closed)
                    .max_by_key(|kline| kline.timestamp)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for KLineService {
    fn default() -> Self {
        Self::new()
    }
}
