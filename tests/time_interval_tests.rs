use chrono::{Timelike, Utc, TimeZone};
use k_line::models::{TimeInterval, Transaction};
use k_line::services::KLineService;

#[tokio::test]
async fn test_hour_interval_alignment() {
    let service = KLineService::new();
    
    // Test time: 2024-01-15 14:35:42.123456789 UTC
    let test_time = Utc.with_ymd_and_hms(2024, 1, 15, 14, 35, 42)
        .unwrap()
        .with_nanosecond(123456789)
        .unwrap();
    
    let transaction = Transaction {
        token: "DOGE".to_string(),
        price: 0.15,
        volume: 100.0,
        timestamp: test_time,
        is_buy: true,
    };
    
    service.process_transaction(&transaction);
    
    // Get 1-hour K-line
    let klines = service.get_klines(
        "DOGE",
        TimeInterval::Hour1,
        test_time - chrono::Duration::hours(1),
        test_time + chrono::Duration::hours(1),
        None,
    );
    
    assert_eq!(klines.len(), 1);
    let kline = &klines[0];
    
    // Verify K-line timestamp should align to hour: 14:00:00.000
    let expected_time = Utc.with_ymd_and_hms(2024, 1, 15, 14, 0, 0).unwrap();
    assert_eq!(kline.timestamp, expected_time);
    
    println!("Original time: {}", test_time);
    println!("Aligned time: {}", kline.timestamp);
}

#[tokio::test]
async fn test_minute_interval_alignment() {
    let service = KLineService::new();
    
    // Test time: 2024-01-15 14:35:42.123456789 UTC
    let test_time = Utc.with_ymd_and_hms(2024, 1, 15, 14, 35, 42)
        .unwrap()
        .with_nanosecond(123456789)
        .unwrap();
    
    let transaction = Transaction {
        token: "DOGE".to_string(),
        price: 0.15,
        volume: 100.0,
        timestamp: test_time,
        is_buy: true,
    };
    
    service.process_transaction(&transaction);
    
    // Get K-line
    let klines = service.get_klines(
        "DOGE",
        TimeInterval::Minute1,
        test_time - chrono::Duration::minutes(1),
        test_time + chrono::Duration::minutes(1),
        None,
    );
    
    assert_eq!(klines.len(), 1);
    let kline = &klines[0];
    
    // Verify K-line timestamp alignment
    let expected_time = Utc.with_ymd_and_hms(2024, 1, 15, 14, 35, 0).unwrap();
    assert_eq!(kline.timestamp, expected_time);
    
    println!("Original time: {}", test_time);
    println!("Aligned time: {}", kline.timestamp);
}

#[tokio::test]
async fn test_5minute_interval_alignment() {
    let service = KLineService::new();
    
    // Test time: 2024-01-15 14:37:42.123456789 UTC
    let test_time = Utc.with_ymd_and_hms(2024, 1, 15, 14, 37, 42)
        .unwrap()
        .with_nanosecond(123456789)
        .unwrap();
    
    let transaction = Transaction {
        token: "DOGE".to_string(),
        price: 0.15,
        volume: 100.0,
        timestamp: test_time,
        is_buy: true,
    };
    
    service.process_transaction(&transaction);
    
    // Get K-line
    let klines = service.get_klines(
        "DOGE",
        TimeInterval::Minute5,
        test_time - chrono::Duration::minutes(10),
        test_time + chrono::Duration::minutes(10),
        None,
    );
    
    assert_eq!(klines.len(), 1);
    let kline = &klines[0];
    
    // Verify K-line timestamp alignment
    let expected_time = Utc.with_ymd_and_hms(2024, 1, 15, 14, 35, 0).unwrap();
    assert_eq!(kline.timestamp, expected_time);
    
    println!("Original time: {}", test_time);
    println!("Aligned time: {}", kline.timestamp);
}

#[tokio::test]
async fn test_15minute_interval_alignment() {
    let service = KLineService::new();
    
    // Test time: 2024-01-15 14:37:42.123456789 UTC
    let test_time = Utc.with_ymd_and_hms(2024, 1, 15, 14, 37, 42)
        .unwrap()
        .with_nanosecond(123456789)
        .unwrap();
    
    let transaction = Transaction {
        token: "DOGE".to_string(),
        price: 0.15,
        volume: 100.0,
        timestamp: test_time,
        is_buy: true,
    };
    
    service.process_transaction(&transaction);
    
    // Get K-line
    let klines = service.get_klines(
        "DOGE",
        TimeInterval::Minute15,
        test_time - chrono::Duration::minutes(30),
        test_time + chrono::Duration::minutes(30),
        None,
    );
    
    assert_eq!(klines.len(), 1);
    let kline = &klines[0];
    
    // Verify K-line timestamp alignment
    let expected_time = Utc.with_ymd_and_hms(2024, 1, 15, 14, 30, 0).unwrap();
    assert_eq!(kline.timestamp, expected_time);
    
    println!("Original time: {}", test_time);
    println!("Aligned time: {}", kline.timestamp);
}

#[tokio::test]
async fn test_second_interval_alignment() {
    let service = KLineService::new();
    
    // Test time: 2024-01-15 14:35:42.123456789 UTC
    let test_time = Utc.with_ymd_and_hms(2024, 1, 15, 14, 35, 42)
        .unwrap()
        .with_nanosecond(123456789)
        .unwrap();
    
    let transaction = Transaction {
        token: "DOGE".to_string(),
        price: 0.15,
        volume: 100.0,
        timestamp: test_time,
        is_buy: true,
    };
    
    service.process_transaction(&transaction);
    
    // Get K-line
    let klines = service.get_klines(
        "DOGE",
        TimeInterval::Second1,
        test_time - chrono::Duration::seconds(1),
        test_time + chrono::Duration::seconds(1),
        None,
    );
    
    assert_eq!(klines.len(), 1);
    let kline = &klines[0];
    
    // Verify K-line timestamp alignment
    let expected_time = Utc.with_ymd_and_hms(2024, 1, 15, 14, 35, 42).unwrap();
    assert_eq!(kline.timestamp, expected_time);
    
    println!("Original time: {}", test_time);
    println!("Aligned time: {}", kline.timestamp);
}

#[tokio::test]
async fn test_multiple_transactions_same_interval() {
    let service = KLineService::new();
    
    // Create transactions at different times within the same hour
    let base_time = Utc.with_ymd_and_hms(2024, 1, 15, 14, 0, 0).unwrap();
    
    let transactions = vec![
        (base_time + chrono::Duration::minutes(5), 0.15, 100.0),
        (base_time + chrono::Duration::minutes(25), 0.16, 200.0),
        (base_time + chrono::Duration::minutes(45), 0.14, 150.0),
    ];
    
    for (timestamp, price, volume) in transactions {
        let transaction = Transaction {
            token: "DOGE".to_string(),
            price,
            volume,
            timestamp,
            is_buy: true,
        };
        service.process_transaction(&transaction);
    }
    
    // Get K-line
    let klines = service.get_klines(
        "DOGE",
        TimeInterval::Hour1,
        base_time - chrono::Duration::hours(1),
        base_time + chrono::Duration::hours(2),
        None,
    );
    
    // Should have only one K-line since all transactions are within the same hour
    assert_eq!(klines.len(), 1);
    let kline = &klines[0];
    
    // Verify K-line timestamp aligns to hour
    assert_eq!(kline.timestamp, base_time);
    
    // Verify OHLC data
    assert_eq!(kline.open, 0.15);   // First transaction price
    assert_eq!(kline.close, 0.14);  // Last transaction price
    assert_eq!(kline.high, 0.16);   // Highest price
    assert_eq!(kline.low, 0.14);    // Lowest price
    assert_eq!(kline.volume, 450.0); // Total volume
    
    println!("K-line data: {:?}", kline);
} 