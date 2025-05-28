use actix_web::{web, HttpResponse, Result};
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use crate::services::KLineService;
use crate::models::TimeInterval;

/// Get K-line data for a specific token and interval
pub async fn get_klines(
    kline_service: web::Data<Arc<KLineService>>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let token = query.get("token").unwrap_or(&"DOGE".to_string()).clone();
    let interval_str = query.get("interval").unwrap_or(&"1m".to_string()).clone();
    
    let interval = match TimeInterval::from_str(&interval_str) {
        Ok(interval) => interval,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Invalid interval. Supported: 1s, 1m, 5m, 15m, 1h"
            })));
        }
    };

    let limit: usize = query
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
        .min(1000); // Maximum 1000 records

    // Set default time range (last 24 hours)
    let end = chrono::Utc::now();
    let start = end - chrono::Duration::hours(24);

    let klines = kline_service.get_klines(&token, interval, start, end, Some(limit));
    
    Ok(HttpResponse::Ok().json(json!({
        "token": token,
        "interval": interval_str,
        "data": klines
    })))
}

/// Get the latest completed K-line for a specific token and interval
pub async fn get_latest_kline(
    kline_service: web::Data<Arc<KLineService>>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let token = query.get("token").unwrap_or(&"DOGE".to_string()).clone();
    let interval_str = query.get("interval").unwrap_or(&"1m".to_string()).clone();
    
    let interval = match TimeInterval::from_str(&interval_str) {
        Ok(interval) => interval,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Invalid interval. Supported: 1s, 1m, 5m, 15m, 1h"
            })));
        }
    };

    match kline_service.get_latest_kline(&token, interval) {
        Some(kline) => Ok(HttpResponse::Ok().json(json!({
            "token": token,
            "interval": interval_str,
            "data": kline
        }))),
        None => Ok(HttpResponse::NotFound().json(json!({
            "error": "No K-line data found for the specified token and interval"
        })))
    }
}

/// Get the current (open) K-line for a specific token and interval
pub async fn get_current_kline(
    kline_service: web::Data<Arc<KLineService>>,
    query: web::Query<HashMap<String, String>>,
) -> Result<HttpResponse> {
    let token = query.get("token").unwrap_or(&"DOGE".to_string()).clone();
    let interval_str = query.get("interval").unwrap_or(&"1m".to_string()).clone();
    
    let interval = match TimeInterval::from_str(&interval_str) {
        Ok(interval) => interval,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": "Invalid interval. Supported: 1s, 1m, 5m, 15m, 1h"
            })));
        }
    };

    match kline_service.get_current_kline(&token, interval) {
        Some(kline) => Ok(HttpResponse::Ok().json(json!({
            "token": token,
            "interval": interval_str,
            "data": kline,
            "is_open": true
        }))),
        None => Ok(HttpResponse::NotFound().json(json!({
            "error": "No current K-line data found for the specified token and interval"
        })))
    }
}

/// Get list of supported tokens
pub async fn get_tokens(
    kline_service: web::Data<Arc<KLineService>>,
) -> Result<HttpResponse> {
    let tokens = kline_service.get_available_tokens();
    
    Ok(HttpResponse::Ok().json(json!({
        "tokens": tokens,
        "count": tokens.len()
    })))
}

/// Health check endpoint
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "k-line-data-service",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Get service statistics
pub async fn get_stats(
    kline_service: web::Data<Arc<KLineService>>,
) -> Result<HttpResponse> {
    let tokens = kline_service.get_available_tokens();
    
    Ok(HttpResponse::Ok().json(json!({
        "statistics": {
            "total_tokens": tokens.len(),
            "supported_tokens": tokens,
            "supported_intervals": ["1s", "1m", "5m", "15m", "1h"]
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Configure REST API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/klines", web::get().to(get_klines))
            .route("/klines/latest", web::get().to(get_latest_kline))
            .route("/klines/current", web::get().to(get_current_kline))
            .route("/tokens", web::get().to(get_tokens))
            .route("/stats", web::get().to(get_stats))
            .route("/health", web::get().to(health_check))
    );
    
    // Serve static files
    cfg.route("/", web::get().to(serve_index))
        .route("/websocket_test.html", web::get().to(serve_index));
}

/// Serve the main HTML file
async fn serve_index() -> Result<HttpResponse> {
    // Try Docker path first, then local path
    let paths = ["/app/websocket_test.html", "./websocket_test.html"];
    
    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            return Ok(HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(content));
        }
    }
    
    Ok(HttpResponse::NotFound().body("HTML file not found"))
} 