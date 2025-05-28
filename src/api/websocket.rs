use actix::{Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::models::{KLine, TimeInterval, Transaction};
use crate::services::KLineService;

/// WebSocket connection heartbeat interval
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// Client timeout duration
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// WebSocket subscription types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SubscriptionType {
    /// Subscribe to real-time transactions for specific tokens
    #[serde(rename = "transactions")]
    Transactions { tokens: Vec<String> },
    /// Subscribe to real-time K-line updates for specific token and interval
    #[serde(rename = "klines")]
    KLines { token: String, interval: String },
    /// Subscribe to all transactions
    #[serde(rename = "all_transactions")]
    AllTransactions,
}

/// WebSocket message types from client
#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum ClientMessage {
    /// Subscribe to data streams
    #[serde(rename = "subscribe")]
    Subscribe { subscription: SubscriptionType },
    /// Unsubscribe from data streams
    #[serde(rename = "unsubscribe")]
    Unsubscribe { subscription: SubscriptionType },
    /// Ping message for heartbeat
    #[serde(rename = "ping")]
    Ping,
}

/// WebSocket message types to client
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Real-time transaction data
    #[serde(rename = "transaction")]
    Transaction { data: Transaction },
    /// Real-time K-line update
    #[serde(rename = "kline")]
    KLine { data: KLine },
    /// Subscription confirmation
    #[serde(rename = "subscribed")]
    Subscribed { subscription: SubscriptionType },
    /// Unsubscription confirmation
    #[serde(rename = "unsubscribed")]
    Unsubscribed { subscription: SubscriptionType },
    /// Pong response
    #[serde(rename = "pong")]
    Pong,
    /// Error message
    #[serde(rename = "error")]
    Error { message: String },
}

/// WebSocket session
pub struct WsSession {
    /// Unique session ID
    id: Uuid,
    /// Last heartbeat time
    hb: Instant,
    /// Current subscriptions
    subscriptions: Vec<SubscriptionType>,
    /// Reference to the WebSocket manager
    manager: Arc<RwLock<WsManager>>,
}

impl WsSession {
    pub fn new(manager: Arc<RwLock<WsManager>>, _kline_service: Arc<KLineService>) -> Self {
        let id = Uuid::new_v4();
        
        // Register this session with the manager
        if let Ok(mut mgr) = manager.write() {
            mgr.add_session(id);
        }

        Self {
            id,
            hb: Instant::now(),
            subscriptions: Vec::new(),
            manager,
        }
    }

    /// Start heartbeat process
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("WebSocket client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }

    /// Send message to client
    fn send_message(&self, msg: ServerMessage, ctx: &mut ws::WebsocketContext<Self>) {
        if let Ok(json) = serde_json::to_string(&msg) {
            ctx.text(json);
        }
    }

    /// Handle subscription
    fn handle_subscribe(&mut self, subscription: SubscriptionType, ctx: &mut ws::WebsocketContext<Self>) {
        // Validate subscription
        if let SubscriptionType::KLines { ref interval, .. } = subscription {
            if interval.parse::<TimeInterval>().is_err() {
                self.send_message(
                    ServerMessage::Error {
                        message: format!("Invalid interval: {}", interval),
                    },
                    ctx,
                );
                return;
            }
        }

        // Add subscription
        self.subscriptions.push(subscription.clone());

        // Register subscription with manager
        if let Ok(mut manager) = self.manager.write() {
            manager.add_subscription(self.id, subscription.clone());
        }

        // Send confirmation
        self.send_message(ServerMessage::Subscribed { subscription }, ctx);
    }

    /// Handle unsubscription
    fn handle_unsubscribe(&mut self, subscription: SubscriptionType, ctx: &mut ws::WebsocketContext<Self>) {
        // Remove subscription
        self.subscriptions.retain(|s| !subscription_matches(s, &subscription));

        // Unregister subscription with manager
        if let Ok(mut manager) = self.manager.write() {
            manager.remove_subscription(self.id, &subscription);
        }

        // Send confirmation
        self.send_message(ServerMessage::Unsubscribed { subscription }, ctx);
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        
        // Set the session address in the manager
        if let Ok(mut manager) = self.manager.write() {
            manager.set_session_addr(self.id, ctx.address());
        }
        
        println!("WebSocket session {} started", self.id);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // Remove session from manager
        if let Ok(mut manager) = self.manager.write() {
            manager.remove_session(self.id);
        }
        println!("WebSocket session {} stopped", self.id);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                self.hb = Instant::now();
                
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(ClientMessage::Subscribe { subscription }) => {
                        self.handle_subscribe(subscription, ctx);
                    }
                    Ok(ClientMessage::Unsubscribe { subscription }) => {
                        self.handle_unsubscribe(subscription, ctx);
                    }
                    Ok(ClientMessage::Ping) => {
                        self.send_message(ServerMessage::Pong, ctx);
                    }
                    Err(e) => {
                        self.send_message(
                            ServerMessage::Error {
                                message: format!("Invalid message format: {}", e),
                            },
                            ctx,
                        );
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                self.send_message(
                    ServerMessage::Error {
                        message: "Binary messages not supported".to_string(),
                    },
                    ctx,
                );
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

/// Message for broadcasting transactions
#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastTransaction(pub Transaction);

/// Message for broadcasting K-line updates
#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastKLine(pub KLine);

impl Handler<BroadcastTransaction> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: BroadcastTransaction, ctx: &mut Self::Context) {
        let transaction = msg.0;
        
        // Check if this session is subscribed to this transaction
        for subscription in &self.subscriptions {
            match subscription {
                SubscriptionType::AllTransactions => {
                    self.send_message(ServerMessage::Transaction { data: transaction.clone() }, ctx);
                    break;
                }
                SubscriptionType::Transactions { tokens } => {
                    if tokens.contains(&transaction.token) {
                        self.send_message(ServerMessage::Transaction { data: transaction.clone() }, ctx);
                        break;
                    }
                }
                _ => {}
            }
        }
    }
}

impl Handler<BroadcastKLine> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: BroadcastKLine, ctx: &mut Self::Context) {
        let kline = msg.0;
        
        // Check if this session is subscribed to this K-line
        for subscription in &self.subscriptions {
            if let SubscriptionType::KLines { token, interval } = subscription {
                if token == &kline.token && interval == kline.interval.as_str() {
                    self.send_message(ServerMessage::KLine { data: kline.clone() }, ctx);
                    break;
                }
            }
        }
    }
}

/// WebSocket manager for handling multiple sessions
#[derive(Debug)]
pub struct WsManager {
    /// Active sessions
    sessions: HashMap<Uuid, actix::Addr<WsSession>>,
    /// Session subscriptions
    subscriptions: HashMap<Uuid, Vec<SubscriptionType>>,
}

impl WsManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            subscriptions: HashMap::new(),
        }
    }

    /// Add a new session
    pub fn add_session(&mut self, session_id: Uuid) {
        self.subscriptions.insert(session_id, Vec::new());
    }

    /// Remove a session
    pub fn remove_session(&mut self, session_id: Uuid) {
        self.sessions.remove(&session_id);
        self.subscriptions.remove(&session_id);
    }

    /// Add session address
    pub fn set_session_addr(&mut self, session_id: Uuid, addr: actix::Addr<WsSession>) {
        self.sessions.insert(session_id, addr);
    }

    /// Add subscription for a session
    pub fn add_subscription(&mut self, session_id: Uuid, subscription: SubscriptionType) {
        if let Some(subs) = self.subscriptions.get_mut(&session_id) {
            subs.push(subscription);
        }
    }

    /// Remove subscription for a session
    pub fn remove_subscription(&mut self, session_id: Uuid, subscription: &SubscriptionType) {
        if let Some(subs) = self.subscriptions.get_mut(&session_id) {
            subs.retain(|s| !subscription_matches(s, subscription));
        }
    }

    /// Broadcast transaction to all relevant sessions
    pub fn broadcast_transaction(&self, transaction: &Transaction) {
        for (session_id, addr) in &self.sessions {
            if let Some(subscriptions) = self.subscriptions.get(session_id) {
                let should_send = subscriptions.iter().any(|sub| match sub {
                    SubscriptionType::AllTransactions => true,
                    SubscriptionType::Transactions { tokens } => tokens.contains(&transaction.token),
                    _ => false,
                });

                if should_send {
                    addr.do_send(BroadcastTransaction(transaction.clone()));
                }
            }
        }
    }

    /// Broadcast K-line update to all relevant sessions
    pub fn broadcast_kline(&self, kline: &KLine) {
        for (session_id, addr) in &self.sessions {
            if let Some(subscriptions) = self.subscriptions.get(session_id) {
                let should_send = subscriptions.iter().any(|sub| match sub {
                    SubscriptionType::KLines { token, interval } => {
                        token == &kline.token && interval == kline.interval.as_str()
                    }
                    _ => false,
                });

                if should_send {
                    addr.do_send(BroadcastKLine(kline.clone()));
                }
            }
        }
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for WsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if two subscriptions match
fn subscription_matches(a: &SubscriptionType, b: &SubscriptionType) -> bool {
    match (a, b) {
        (SubscriptionType::AllTransactions, SubscriptionType::AllTransactions) => true,
        (
            SubscriptionType::Transactions { tokens: tokens_a },
            SubscriptionType::Transactions { tokens: tokens_b },
        ) => tokens_a == tokens_b,
        (
            SubscriptionType::KLines { token: token_a, interval: interval_a },
            SubscriptionType::KLines { token: token_b, interval: interval_b },
        ) => token_a == token_b && interval_a == interval_b,
        _ => false,
    }
}

/// WebSocket endpoint handler
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    manager: web::Data<Arc<RwLock<WsManager>>>,
    kline_service: web::Data<Arc<KLineService>>,
) -> Result<HttpResponse> {
    let session = WsSession::new(manager.get_ref().clone(), kline_service.get_ref().clone());
    let _session_id = session.id;
    
    let resp = ws::start(session, &req, stream)?;
    
    // Note: We can't set the session address here because ws::start consumes the session
    // The session address will be set when the session starts via the Actor::started method
    
    Ok(resp)
}

/// Configure WebSocket routes
pub fn configure_websocket_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/ws", web::get().to(websocket_handler));
} 