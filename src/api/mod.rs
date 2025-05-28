pub mod rest;
pub mod websocket;

// Re-export for convenience
pub use rest::configure_routes;
pub use websocket::{configure_websocket_routes, WsManager};
