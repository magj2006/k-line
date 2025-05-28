use actix_web::{test, web, App};
use std::sync::Arc;
use k_line::{KLineService, MockDataGenerator, configure_routes};

#[actix_web::test]
async fn test_get_tokens_endpoint() {
    let service = Arc::new(KLineService::new());
    let generator = MockDataGenerator::new();

    // Generate some test data
    for _ in 0..5 {
        let transaction = generator.generate_random_transaction();
        service.process_transaction(&transaction);
    }

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(service))
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/tokens")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["tokens"].is_array());
    assert!(body["count"].is_number());
}

#[actix_web::test]
async fn test_get_klines_endpoint() {
    let service = Arc::new(KLineService::new());
    let generator = MockDataGenerator::new();

    // Generate test data for DOGE
    for _ in 0..10 {
        if let Some(transaction) = generator.generate_transaction("DOGE") {
            service.process_transaction(&transaction);
        }
    }

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(service))
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/klines?token=DOGE&interval=1m")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["token"], "DOGE");
    assert_eq!(body["interval"], "1m");
    assert!(body["data"].is_array());
}

#[actix_web::test]
async fn test_get_latest_kline_endpoint() {
    let service = Arc::new(KLineService::new());
    let generator = MockDataGenerator::new();

    // Generate test data for DOGE
    if let Some(transaction) = generator.generate_transaction("DOGE") {
        service.process_transaction(&transaction);
    }

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(service))
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/klines/latest?token=DOGE&interval=1m")
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    // The response might be 404 if no data exists, which is valid
    if resp.status().is_success() {
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["token"], "DOGE");
        assert_eq!(body["interval"], "1m");
        assert!(body["data"].is_object() || body["data"].is_null());
    } else {
        assert_eq!(resp.status(), 404);
    }
}

#[actix_web::test]
async fn test_get_current_kline_endpoint() {
    let service = Arc::new(KLineService::new());
    let generator = MockDataGenerator::new();

    // Generate test data for DOGE
    if let Some(transaction) = generator.generate_transaction("DOGE") {
        service.process_transaction(&transaction);
    }

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(service))
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/klines/current?token=DOGE&interval=1m")
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    // The response might be 404 if no data exists, which is valid
    if resp.status().is_success() {
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["token"], "DOGE");
        assert_eq!(body["interval"], "1m");
        assert!(body["data"].is_object());
        assert_eq!(body["is_open"], true);
    } else {
        assert_eq!(resp.status(), 404);
    }
}

#[actix_web::test]
async fn test_invalid_interval() {
    let service = Arc::new(KLineService::new());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(service))
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/klines?token=DOGE&interval=invalid")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].is_string());
} 