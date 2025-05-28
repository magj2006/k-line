use actix_web::{web, App, HttpServer, middleware::Logger};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio::task;

use k_line::{
    KLineService, MockDataGenerator, WsManager,
    configure_routes, configure_websocket_routes,
    config::Config
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init();

    // Load configuration
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("Failed to load configuration: {}", e);
        eprintln!("Using default configuration");
        Config::default()
    });

    println!("Configuration loaded:");
    println!("  Server: {}:{}", config.server.host, config.server.port);
    println!("  Supported tokens: {:?}", config.get_supported_tokens());
    println!("  Data generation enabled: {}", config.data_generation.enabled);
    println!("  Data generation interval: {}ms", config.data_generation.interval_ms);
    println!("  Volatility: {:.2}%", config.data_generation.volatility * 100.0);

    // Create services
    let kline_service = Arc::new(KLineService::new());
    let ws_manager = Arc::new(RwLock::new(WsManager::new()));
    
    // Create mock data generator with configuration
    let mock_generator = MockDataGenerator::new_with_config(&config);
    
    // Start mock data generation in background if enabled
    if config.data_generation.enabled {
        let kline_service_clone = kline_service.clone();
        let ws_manager_clone = ws_manager.clone();
        let generation_interval = config.data_generation.interval_ms;
        
        task::spawn(async move {
            mock_generator.start_continuous_generation(
                move |transaction| {
                    // Process transaction and update K-lines
                    kline_service_clone.process_transaction(&transaction);
                    
                    // Broadcast transaction to WebSocket clients
                    if let Ok(manager) = ws_manager_clone.read() {
                        manager.broadcast_transaction(&transaction);
                    }
                    
                    // Get updated K-lines and broadcast them
                    for interval in ["1s", "1m", "5m", "15m", "1h"] {
                        if let Ok(interval_enum) = k_line::TimeInterval::from_str(interval) {
                            if let Some(kline) = kline_service_clone.get_current_kline(&transaction.token, interval_enum) {
                                if let Ok(manager) = ws_manager_clone.read() {
                                    manager.broadcast_kline(&kline);
                                }
                            }
                        }
                    }
                    
                    println!("Processed transaction: {} {} @ {}", 
                        transaction.token, 
                        transaction.volume, 
                        transaction.price
                    );
                },
                generation_interval,
            ).await;
        });
    } else {
        println!("Mock data generation is disabled");
    }

    let server_address = format!("{}:{}", config.server.host, config.server.port);
    println!("Starting K-line data service on http://{}", server_address);
    println!("Available endpoints:");
    println!("  REST API:");
    println!("    GET /api/v1/klines?token=DOGE&interval=1m");
    println!("    GET /api/v1/klines/latest?token=DOGE&interval=1m");
    println!("    GET /api/v1/klines/current?token=DOGE&interval=1m");
    println!("    GET /api/v1/tokens");
    println!("  WebSocket:");
    println!("    WS  /ws");
    println!();
    println!("WebSocket subscription examples:");
    println!("  Subscribe to all transactions: {{\"action\":\"subscribe\",\"subscription\":{{\"type\":\"all_transactions\"}}}}");
    println!("  Subscribe to DOGE transactions: {{\"action\":\"subscribe\",\"subscription\":{{\"type\":\"transactions\",\"tokens\":[\"DOGE\"]}}}}");
    println!("  Subscribe to DOGE 1m K-lines: {{\"action\":\"subscribe\",\"subscription\":{{\"type\":\"klines\",\"token\":\"DOGE\",\"interval\":\"1m\"}}}}");

    // Configure server based on configuration
    let workers = config.server.workers;
    let server_config = config.clone();

    // Start HTTP server with configuration
    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(kline_service.clone()))
            .app_data(web::Data::new(ws_manager.clone()))
            .app_data(web::Data::new(server_config.clone()))
            .wrap(Logger::default())
            .configure(configure_routes)
            .configure(configure_websocket_routes)
    });

    if let Some(workers) = workers {
        server = server.workers(workers);
    }

    server
        .bind(&server_address)?
        .run()
        .await
}
