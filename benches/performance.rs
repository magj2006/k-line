use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use k_line::models::{TimeInterval, Transaction};
use k_line::services::KLineService;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn benchmark_single_transaction_processing(c: &mut Criterion) {
    let service = Arc::new(KLineService::new());

    c.bench_function("process_single_transaction", |b| {
        b.iter(|| {
            let transaction = Transaction {
                token: "DOGE".to_string(),
                price: black_box(0.15),
                volume: black_box(100.0),
                timestamp: Utc::now(),
                is_buy: true,
            };
            service.process_transaction(black_box(&transaction));
        })
    });
}

fn benchmark_concurrent_transaction_processing(c: &mut Criterion) {
    let service = Arc::new(KLineService::new());

    c.bench_function("process_concurrent_transactions", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let service = Arc::clone(&service);
                    thread::spawn(move || {
                        let transaction = Transaction {
                            token: format!("TOKEN{}", i % 3),
                            price: 0.15 + (i as f64 * 0.01),
                            volume: 100.0 + (i as f64 * 10.0),
                            timestamp: Utc::now(),
                            is_buy: i % 2 == 0,
                        };
                        service.process_transaction(&transaction);
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}

fn benchmark_kline_retrieval(c: &mut Criterion) {
    let service = Arc::new(KLineService::new());

    // Pre-populate some data
    for i in 0..1000 {
        let transaction = Transaction {
            token: "DOGE".to_string(),
            price: 0.15 + (i as f64 * 0.0001),
            volume: 100.0,
            timestamp: Utc::now() - chrono::Duration::seconds(i),
            is_buy: i % 2 == 0,
        };
        service.process_transaction(&transaction);
    }

    c.bench_function("get_klines", |b| {
        b.iter(|| {
            let now = Utc::now();
            let start = now - chrono::Duration::hours(1);
            let klines = service.get_klines(
                black_box("DOGE"),
                black_box(TimeInterval::Minute1),
                black_box(start),
                black_box(now),
                black_box(Some(100)),
            );
            black_box(klines);
        })
    });
}

fn benchmark_high_frequency_updates(c: &mut Criterion) {
    let service = Arc::new(KLineService::new());

    c.bench_function("high_frequency_updates", |b| {
        b.iter(|| {
            // Simulate high-frequency trading scenario: 100 transactions in 1 second
            for i in 0..100 {
                let transaction = Transaction {
                    token: "DOGE".to_string(),
                    price: 0.15 + (i as f64 * 0.00001),
                    volume: 10.0 + (i as f64),
                    timestamp: Utc::now(),
                    is_buy: i % 2 == 0,
                };
                service.process_transaction(black_box(&transaction));
            }
        })
    });
}

fn benchmark_memory_usage(c: &mut Criterion) {
    c.bench_function("memory_intensive_operations", |b| {
        b.iter(|| {
            let service = Arc::new(KLineService::new());

            // Create many transactions with different timestamps
            for i in 0..1000 {
                let transaction = Transaction {
                    token: format!("TOKEN{}", i % 10),
                    price: 0.15 + (i as f64 * 0.0001),
                    volume: 100.0,
                    timestamp: Utc::now() - chrono::Duration::seconds(i * 60), // One per minute
                    is_buy: i % 2 == 0,
                };
                service.process_transaction(&transaction);
            }

            // Query all data
            let now = Utc::now();
            let start = now - chrono::Duration::hours(24);
            for token_id in 0..10 {
                let token = format!("TOKEN{}", token_id);
                for interval in [
                    TimeInterval::Second1,
                    TimeInterval::Minute1,
                    TimeInterval::Minute5,
                    TimeInterval::Minute15,
                    TimeInterval::Hour1,
                ] {
                    let _klines = service.get_klines(&token, interval, start, now, None);
                }
            }

            black_box(service);
        })
    });
}

fn benchmark_websocket_simulation(c: &mut Criterion) {
    let service = Arc::new(KLineService::new());

    c.bench_function("websocket_broadcast_simulation", |b| {
        b.iter(|| {
            // Simulate WebSocket broadcast scenario
            let handles: Vec<_> = (0..50)
                .map(|i| {
                    let service = Arc::clone(&service);
                    thread::spawn(move || {
                        // Each thread simulates a WebSocket client subscription
                        for j in 0..20 {
                            let transaction = Transaction {
                                token: "DOGE".to_string(),
                                price: 0.15 + (j as f64 * 0.0001),
                                volume: 100.0,
                                timestamp: Utc::now(),
                                is_buy: (i + j) % 2 == 0,
                            };
                            service.process_transaction(&transaction);

                            // Simulate client query
                            let _current = service.get_current_kline("DOGE", TimeInterval::Second1);

                            // Brief delay to simulate network latency
                            thread::sleep(Duration::from_micros(100));
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}

criterion_group!(
    benches,
    benchmark_single_transaction_processing,
    benchmark_concurrent_transaction_processing,
    benchmark_kline_retrieval,
    benchmark_high_frequency_updates,
    benchmark_memory_usage,
    benchmark_websocket_simulation
);

criterion_main!(benches);
