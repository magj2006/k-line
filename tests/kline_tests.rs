use chrono::{Duration, Utc};
use k_line::{KLine, KLineService, MockDataGenerator, TimeInterval, Transaction};

#[test]
fn test_kline_creation() {
    let kline = KLine::new(
        "DOGE".to_string(),
        Utc::now(),
        TimeInterval::Minute1,
        0.15,
        100.0,
    );

    assert_eq!(kline.token, "DOGE");
    assert_eq!(kline.interval, TimeInterval::Minute1);
    assert_eq!(kline.open, 0.15);
    assert_eq!(kline.high, 0.15);
    assert_eq!(kline.low, 0.15);
    assert_eq!(kline.close, 0.15);
    assert_eq!(kline.volume, 100.0);
    assert!(!kline.is_closed);
}

#[test]
fn test_kline_update() {
    let mut kline = KLine::new(
        "DOGE".to_string(),
        Utc::now(),
        TimeInterval::Minute1,
        0.15,
        100.0,
    );

    // Update with higher price
    kline.update(0.16, 50.0);
    assert_eq!(kline.high, 0.16);
    assert_eq!(kline.close, 0.16);
    assert_eq!(kline.volume, 150.0);

    // Update with lower price
    kline.update(0.14, 25.0);
    assert_eq!(kline.low, 0.14);
    assert_eq!(kline.close, 0.14);
    assert_eq!(kline.volume, 175.0);
}

#[test]
fn test_time_interval_parsing() {
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
fn test_kline_service_basic() {
    let service = KLineService::new();
    let transaction = Transaction::new("DOGE".to_string(), 0.15, 100.0, true);

    // Process transaction
    service.process_transaction(&transaction);

    // Check that K-lines were created for all intervals
    let kline = service.get_latest_kline("DOGE", TimeInterval::Minute1);
    assert!(kline.is_some());

    let kline = kline.unwrap();
    assert_eq!(kline.token, "DOGE");
    assert_eq!(kline.open, 0.15);
    assert_eq!(kline.close, 0.15);
    assert_eq!(kline.volume, 100.0);
}

#[test]
fn test_kline_service_multiple_transactions() {
    let service = KLineService::new();

    // Create multiple transactions
    let t1 = Transaction::new("DOGE".to_string(), 0.15, 100.0, true);
    let t2 = Transaction::new("DOGE".to_string(), 0.16, 50.0, true);
    let t3 = Transaction::new("DOGE".to_string(), 0.14, 75.0, false);

    // Process transactions
    service.process_transaction(&t1);
    service.process_transaction(&t2);
    service.process_transaction(&t3);

    // Check aggregated K-line
    let kline = service.get_latest_kline("DOGE", TimeInterval::Minute1);
    assert!(kline.is_some());

    let kline = kline.unwrap();
    assert_eq!(kline.open, 0.15); // First price
    assert_eq!(kline.high, 0.16); // Highest price
    assert_eq!(kline.low, 0.14); // Lowest price
    assert_eq!(kline.close, 0.14); // Last price
    assert_eq!(kline.volume, 225.0); // Sum of volumes
}

#[test]
fn test_kline_service_get_klines() {
    let service = KLineService::new();
    let now = Utc::now();

    // Create transaction
    let transaction = Transaction::new("DOGE".to_string(), 0.15, 100.0, true);
    service.process_transaction(&transaction);

    // Get K-lines for the last hour
    let start = now - Duration::hours(1);
    let end = now + Duration::hours(1);
    let klines = service.get_klines("DOGE", TimeInterval::Minute1, start, end, None);

    assert!(!klines.is_empty());
    assert_eq!(klines[0].token, "DOGE");
}

#[test]
fn test_kline_service_available_tokens() {
    let service = KLineService::new();

    // Initially no tokens
    assert!(service.get_available_tokens().is_empty());

    // Add transactions for different tokens
    let t1 = Transaction::new("DOGE".to_string(), 0.15, 100.0, true);
    let t2 = Transaction::new("SHIB".to_string(), 0.00001, 1000.0, true);

    service.process_transaction(&t1);
    service.process_transaction(&t2);

    // Check available tokens
    let tokens = service.get_available_tokens();
    assert_eq!(tokens.len(), 2);
    assert!(tokens.contains(&"DOGE".to_string()));
    assert!(tokens.contains(&"SHIB".to_string()));
}

#[test]
fn test_mock_data_generator() {
    let generator = MockDataGenerator::new();

    // Test generating transaction for specific token
    let transaction = generator.generate_transaction("DOGE");
    assert!(transaction.is_some());

    let transaction = transaction.unwrap();
    assert_eq!(transaction.token, "DOGE");
    assert!(transaction.price > 0.0);
    assert!(transaction.volume > 0.0);

    // Test generating random transaction
    let random_transaction = generator.generate_random_transaction();
    assert!(random_transaction.price > 0.0);
    assert!(random_transaction.volume > 0.0);

    // Test available tokens
    let tokens = generator.get_available_tokens();
    assert!(!tokens.is_empty());
    assert!(tokens.contains(&"DOGE".to_string()));
}

#[test]
fn test_mock_data_historical() {
    let generator = MockDataGenerator::new();
    let transactions = generator.generate_historical_data("DOGE", 10);

    assert_eq!(transactions.len(), 10);
    for transaction in transactions {
        assert_eq!(transaction.token, "DOGE");
        assert!(transaction.price > 0.0);
        assert!(transaction.volume > 0.0);
    }
}
