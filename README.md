# K-line Data Service

A high-performance real-time data service for meme token trading platform built in Rust, providing K-line (candlestick) chart data and real-time transaction streaming with a modern web interface.

## 🚀 Features

### Core Functionality
- **Real-time K-line Data**: Provides candlestick chart data for meme tokens (DOGE, SHIB, PEPE)
- **Multiple Time Intervals**: Supports 1s, 1m, 5m, 15m, 1h intervals with proper time alignment
- **Real-time Transaction Streaming**: WebSocket-based live transaction feed
- **Interactive Web Interface**: Modern HTML5 interface with real-time data visualization
- **Mock Data Generation**: Built-in configurable data generator for testing and demonstration
- **Configuration Management**: TOML-based configuration with environment support

### 🌐 Web Interface
Access the interactive web interface at `http://localhost:8080/`:
- **Real-time Transaction Stream**: Filter by token (ALL, DOGE, SHIB, PEPE)
- **Multi-timeframe K-line Display**: Switch between different intervals
- **System Status Monitoring**: Connection status and subscription management
- **Responsive Design**: Works on desktop and mobile devices

## 📡 API Endpoints

### REST API
- `GET /api/v1/klines` - Get historical K-line data with filtering
- `GET /api/v1/klines/latest` - Get the latest completed K-line
- `GET /api/v1/klines/current` - Get current open K-line
- `GET /api/v1/tokens` - Get list of available tokens
- `GET /api/v1/stats` - Get service statistics
- `GET /api/v1/health` - Health check endpoint

### WebSocket API
- `WS /ws` - Real-time data streaming endpoint

### WebSocket Subscriptions

The WebSocket API supports three types of subscriptions:

1. **All Transactions**: Receive all transaction updates
   ```json
   {"action":"subscribe","subscription":{"type":"all_transactions"}}
   ```

2. **Token-specific Transactions**: Receive transactions for specific tokens
   ```json
   {"action":"subscribe","subscription":{"type":"transactions","tokens":["DOGE","SHIB"]}}
   ```

3. **K-line Updates**: Receive real-time K-line updates for specific token/interval
   ```json
   {"action":"subscribe","subscription":{"type":"klines","token":"DOGE","interval":"1m"}}
   ```

## 🏗️ Project Structure

```
src/
├── main.rs                 # Application entry point with dependency injection
├── lib.rs                  # Library exports
├── config.rs               # Configuration management
├── models/                 # Data models
│   ├── mod.rs             # Module exports
│   ├── kline.rs           # K-line data structure with time alignment
│   ├── transaction.rs     # Transaction data structure
│   └── time_interval.rs   # Time interval enum with proper parsing
├── services/              # Business logic
│   ├── mod.rs             # Module exports
│   ├── kline.rs           # K-line data management with DashMap
│   └── mock_data.rs       # Configurable mock data generation
└── api/                   # API layer
    ├── mod.rs             # Module exports
    ├── rest.rs            # REST API endpoints with proper error handling
    └── websocket.rs       # WebSocket implementation with session management

config/                     # Configuration files
├── default.toml           # Default configuration
├── development.toml       # Development environment
└── production.toml        # Production environment

tests/                      # Test suites
├── api_tests.rs           # API endpoint tests
├── kline_tests.rs         # K-line service tests
└── time_interval_tests.rs # Time alignment tests

benches/                    # Performance benchmarks
└── performance.rs         # Benchmark suite

websocket_test.html         # Interactive web interface
docker-compose.yml          # Docker deployment configuration
Dockerfile                  # Multi-stage Docker build
```

## 🚀 Setup and Running

### Prerequisites
- Rust 1.82+ 
- Cargo

### Installation
```bash
git clone <repository-url>
cd k-line
cargo build --release
```

### Configuration

The service uses a hierarchical TOML configuration system for easy management across different environments.

### Configuration Files

The service loads configuration in the following order:

1. **Base Configuration**: `config/default.toml` - Default settings
2. **Environment Configuration**: `config/{environment}.toml` - Environment-specific overrides  
3. **Environment Variables**: Optional runtime overrides (see below)

#### Example Configuration

**Base configuration (`config/default.toml`):**
```toml
[server]
host = "0.0.0.0"
port = 8080

[tokens]
[[tokens.supported_tokens]]
symbol = "DOGE"
base_price = 0.15
volatility = 5.0

[[tokens.supported_tokens]]
symbol = "SHIB"
base_price = 0.00005
volatility = 8.0

[data_generation]
enabled = true
interval_ms = 100
volatility = 0.02
volume_range = [100.0, 1000.0]

[performance]
worker_threads = 4
websocket_heartbeat_interval = 5
client_timeout = 10
kline_retention_hours = 24
max_websocket_connections = 1000
```

**Development configuration (`config/development.toml`):**
```toml
[server]
host = "127.0.0.1"  # Override for local development
workers = 2

[logging]
level = "debug"
file_output = false

[performance]
worker_threads = 2
max_websocket_connections = 100
```

### Environment Selection

```bash
# Use development configuration (default)
cargo run

# Use production configuration
RUST_ENV=production cargo run

# Use custom environment
RUST_ENV=staging cargo run
```

### Running the Service

#### Option 1: Direct Cargo Run
```bash
# Development mode (default)
cargo run

# Production mode
RUST_ENV=production cargo run
```

#### Option 2: Docker (Recommended)
```bash
# Build and run with Docker Compose
docker-compose up --build

# Run in background
docker-compose up -d --build

# View logs
docker-compose logs -f k-line

# Stop the service
docker-compose down
```

The service will start on `http://localhost:8080`

### 🌐 Accessing the Web Interface

1. **Open your browser** and navigate to `http://localhost:8080/websocket_test.html`
2. **Click "Connect"** to establish WebSocket connection
3. **Monitor real-time data**:
   - Transaction stream with token filtering
   - K-line data with interval switching
   - System status and connection info

## 📊 API Usage Examples

### REST API

#### Get Available Tokens
```bash
curl http://localhost:8080/api/v1/tokens
# Response: {"tokens":["DOGE","SHIB","PEPE"],"count":3}
```

#### Get K-line Data
```bash
curl "http://localhost:8080/api/v1/klines?token=DOGE&interval=1m&limit=10"
# Response: {"token":"DOGE","interval":"1m","data":[...]}
```

#### Get Current Open K-line
```bash
curl "http://localhost:8080/api/v1/klines/current?token=DOGE&interval=1m"
# Response: {"token":"DOGE","interval":"1m","data":{...},"is_open":true}
```

#### Health Check
```bash
curl http://localhost:8080/api/v1/health
# Response: {"status":"healthy","service":"k-line-data-service","timestamp":"..."}
```

### WebSocket API

Connect to `ws://localhost:8080/ws` and send subscription messages:

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = function() {
    // Subscribe to all transactions
    ws.send(JSON.stringify({
        action: 'subscribe',
        subscription: { type: 'all_transactions' }
    }));

    // Subscribe to DOGE 1-minute K-lines
    ws.send(JSON.stringify({
        action: 'subscribe',
        subscription: { type: 'klines', token: 'DOGE', interval: '1m' }
    }));
};

ws.onmessage = function(event) {
    const data = JSON.parse(event.data);
    console.log('Received:', data);
};
```

## 📋 Data Models

### K-line Structure
```json
{
    "token": "DOGE",
    "timestamp": "2025-05-28T04:00:00Z",
    "interval": "1m",
    "open": 0.15,
    "high": 0.16,
    "low": 0.14,
    "close": 0.155,
    "volume": 1000.0,
    "is_closed": false
}
```

### Transaction Structure
```json
{
    "token": "DOGE",
    "price": 0.15,
    "volume": 100.0,
    "timestamp": "2025-05-28T04:00:00Z",
    "is_buy": true
}
```

## 🏛️ Architecture

### Real-time Data Flow
1. **Mock Data Generator** creates random transactions every 100ms (configurable)
2. **K-line Service** processes transactions and updates K-lines for all intervals simultaneously
3. **Time Alignment** ensures K-lines align to natural time boundaries
4. **WebSocket Manager** broadcasts updates to subscribed clients with session management
5. **REST API** provides historical data access with proper error handling

### Technical Implementation
- **Storage**: Direct `DashMap` usage for high-performance concurrent access
- **Memory Management**: In-memory storage with configurable retention policies
- **Concurrency**: Lock-free data structures for optimal performance
- **Time Handling**: Precise interval alignment using UTC timestamps
- **Error Handling**: Comprehensive error propagation and logging

### Performance Characteristics
Based on benchmark results:
- **Single Transaction Processing**: ~11.8 µs
- **Concurrent Transaction Processing**: ~167 µs  
- **K-line Query**: ~4.2 µs
- **High-frequency Updates**: ~1.17 ms
- **WebSocket Broadcasting**: Sub-millisecond latency

## 🧪 Testing

Run the comprehensive test suite:
```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test api_tests
cargo test --test kline_tests
cargo test --test time_interval_tests

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

**Test Coverage:**
- ✅ 7 Unit tests (models, config)
- ✅ 5 API tests (REST endpoints)
- ✅ 9 K-line service tests
- ✅ 6 Time alignment tests
- ✅ Performance benchmarks

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📞 Support

For questions and support:
- Create an issue in the repository
- Check the documentation in the `docs/` directory
- Review the test cases for usage examples 
