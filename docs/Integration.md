# K-Line Data Service Integration Guide

This document outlines how to integrate the K-Line Data Service with a larger trading platform ecosystem. The service is designed to be modular, lightweight, and can be easily integrated with various components of a trading platform.

## Overview

The K-Line Data Service is a high-performance, real-time data service that provides:
1. **Real-time K-line (candlestick) chart data** with proper time alignment
2. **Transaction data streaming** for live updates with token filtering
3. **Interactive web interface** for monitoring and testing
4. **Configurable mock data generation** for development and testing
5. **RESTful API** with comprehensive error handling
6. **WebSocket streaming** with subscription management

## Architecture Benefits for Integration

### ✅ **High Performance**
- **Sub-millisecond Latency**: In-memory operations with DashMap
- **Concurrent Processing**: Thread-safe operations with Arc<DashMap>
- **Efficient Broadcasting**: WebSocket session management
- **Benchmark Results**: 84K+ transactions/sec processing

### ✅ **Production Ready**
- **Proper Time Alignment**: K-lines align to natural time boundaries
- **Comprehensive Testing**: 27 test cases covering all functionality
- **Docker Support**: Multi-stage builds with health checks
- **Configuration Management**: Environment-based TOML configuration

### ✅ **Developer Experience**
- **Interactive Web UI**: Built-in monitoring interface at `http://localhost:8080/`
- **Built-in Mock Data**: Configurable data generation (100ms intervals)
- **Hot Configuration**: Runtime environment variable overrides
- **Extensive Documentation**: API specs and integration examples

## Integration Points

### 1. Frontend Integration

#### 1.1 Web Interface Integration

The service includes a complete web interface that can be embedded or used as reference:

```html
<!-- Access the built-in web interface -->
<iframe src="http://localhost:8080/" width="100%" height="600px"></iframe>
```

**Features of the built-in interface:**
- Real-time transaction stream with token filtering (ALL, DOGE, SHIB, PEPE)
- Multi-timeframe K-line display (1s, 1m, 5m, 15m, 1h)
- System status monitoring and connection management
- Responsive design for desktop and mobile

#### 1.2 REST API Integration

**Get Available Tokens:**
```javascript
async function getAvailableTokens() {
  const response = await fetch('http://localhost:8080/api/v1/tokens');
  const data = await response.json();
  // Returns: {"tokens":["DOGE","SHIB","PEPE"],"count":3}
  return data.tokens;
}
```

**Get Historical K-line Data:**
```javascript
async function getKlineData(token, interval, limit = 100) {
  const url = `http://localhost:8080/api/v1/klines?token=${token}&interval=${interval}&limit=${limit}`;
  const response = await fetch(url);
  const data = await response.json();
  // Returns: {"token":"DOGE","interval":"1m","data":[...]}
  return data.data;
}
```

**Get Current Open K-line:**
```javascript
async function getCurrentKline(token, interval) {
  const url = `http://localhost:8080/api/v1/klines/current?token=${token}&interval=${interval}`;
  const response = await fetch(url);
  if (response.ok) {
    const data = await response.json();
    // Returns: {"token":"DOGE","interval":"1m","data":{...},"is_open":true}
    return data.data;
  }
  return null; // No current K-line available
}
```

**Health Check:**
```javascript
async function checkServiceHealth() {
  const response = await fetch('http://localhost:8080/api/v1/health');
  const data = await response.json();
  // Returns: {"status":"healthy","service":"k-line-data-service","timestamp":"..."}
  return data.status === 'healthy';
}
```

#### 1.3 WebSocket Real-time Integration

**Complete WebSocket Integration Example:**
```javascript
class KLineWebSocketClient {
  constructor(baseUrl = 'ws://localhost:8080/ws') {
    this.baseUrl = baseUrl;
    this.ws = null;
    this.subscriptions = new Set();
    this.callbacks = new Map();
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5;
  }

  connect() {
    return new Promise((resolve, reject) => {
      this.ws = new WebSocket(this.baseUrl);
      
      this.ws.onopen = () => {
        console.log('WebSocket connected');
        this.reconnectAttempts = 0;
        resolve();
      };
      
      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          this.handleMessage(data);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };
      
      this.ws.onclose = () => {
        console.log('WebSocket disconnected');
        this.attemptReconnect();
      };
      
      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        reject(error);
      };
    });
  }

  handleMessage(data) {
    switch (data.type) {
      case 'transaction':
        this.notifyCallbacks('transaction', data.data);
        break;
      case 'kline':
        this.notifyCallbacks('kline', data.data);
        break;
      case 'subscribed':
        console.log('Subscription confirmed:', data.subscription);
        break;
      case 'error':
        console.error('Server error:', data.message);
        break;
    }
  }

  notifyCallbacks(type, data) {
    const callbacks = this.callbacks.get(type) || [];
    callbacks.forEach(callback => {
      try {
        callback(data);
      } catch (error) {
        console.error('Callback error:', error);
      }
    });
  }

  // Subscribe to all transactions
  subscribeToAllTransactions(callback) {
    this.subscribe({ type: 'all_transactions' });
    this.addCallback('transaction', callback);
  }

  // Subscribe to specific token transactions
  subscribeToTokenTransactions(tokens, callback) {
    this.subscribe({ type: 'transactions', tokens });
    this.addCallback('transaction', callback);
  }

  // Subscribe to K-line updates
  subscribeToKlines(token, interval, callback) {
    this.subscribe({ type: 'klines', token, interval });
    this.addCallback('kline', callback);
  }

  subscribe(subscription) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      const message = {
        action: 'subscribe',
        subscription
      };
      this.ws.send(JSON.stringify(message));
      this.subscriptions.add(JSON.stringify(subscription));
    }
  }

  addCallback(type, callback) {
    if (!this.callbacks.has(type)) {
      this.callbacks.set(type, []);
    }
    this.callbacks.get(type).push(callback);
  }

  attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = Math.pow(2, this.reconnectAttempts) * 1000; // Exponential backoff
      console.log(`Attempting to reconnect in ${delay}ms (attempt ${this.reconnectAttempts})`);
      
      setTimeout(() => {
        this.connect().then(() => {
          // Re-establish subscriptions
          this.subscriptions.forEach(sub => {
            this.ws.send(JSON.stringify({
              action: 'subscribe',
              subscription: JSON.parse(sub)
            }));
          });
        }).catch(console.error);
      }, delay);
    }
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}

// Usage example
const client = new KLineWebSocketClient();

client.connect().then(() => {
  // Subscribe to all transactions
  client.subscribeToAllTransactions((transaction) => {
    console.log('New transaction:', transaction);
    updateTransactionDisplay(transaction);
  });

  // Subscribe to DOGE 1-minute K-lines
  client.subscribeToKlines('DOGE', '1m', (kline) => {
    console.log('DOGE K-line update:', kline);
    updateChart(kline);
  });
}).catch(console.error);
```

#### 1.4 React Integration with Hooks

```jsx
import React, { useState, useEffect, useCallback } from 'react';

// Custom hook for K-line data
function useKLineData(token, interval) {
  const [klineData, setKlineData] = useState([]);
  const [currentKline, setCurrentKline] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  // Fetch initial historical data
  useEffect(() => {
    const fetchInitialData = async () => {
      try {
        setLoading(true);
        const response = await fetch(
          `http://localhost:8080/api/v1/klines?token=${token}&interval=${interval}&limit=100`
        );
        
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        
        const data = await response.json();
        setKlineData(data.data);
        setError(null);
      } catch (err) {
        setError(err.message);
        console.error('Failed to fetch K-line data:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchInitialData();
  }, [token, interval]);

  // WebSocket connection for real-time updates
  useEffect(() => {
    const ws = new WebSocket('ws://localhost:8080/ws');
    
    ws.onopen = () => {
      // Subscribe to K-line updates
      ws.send(JSON.stringify({
        action: 'subscribe',
        subscription: { type: 'klines', token, interval }
      }));
    };
    
    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        
        if (data.type === 'kline' && data.data.token === token && data.data.interval === interval) {
          const newKline = data.data;
          
          if (newKline.is_closed) {
            // Add completed K-line to history
            setKlineData(prev => [...prev, newKline]);
            setCurrentKline(null);
          } else {
            // Update current open K-line
            setCurrentKline(newKline);
          }
        }
      } catch (err) {
        console.error('Failed to parse WebSocket message:', err);
      }
    };
    
    ws.onerror = (err) => {
      console.error('WebSocket error:', err);
      setError('WebSocket connection failed');
    };
    
    return () => {
      ws.close();
    };
  }, [token, interval]);

  return { klineData, currentKline, loading, error };
}

// React component using the hook
function KLineChart({ token, interval }) {
  const { klineData, currentKline, loading, error } = useKLineData(token, interval);

  if (loading) return <div>Loading K-line data...</div>;
  if (error) return <div>Error: {error}</div>;

  return (
    <div className="kline-chart">
      <h3>{token} - {interval} K-lines</h3>
      <div className="chart-container">
        {/* Your charting library integration here */}
        <CandlestickChart 
          data={klineData} 
          currentKline={currentKline}
          token={token}
          interval={interval}
        />
      </div>
      <div className="stats">
        <p>Historical K-lines: {klineData.length}</p>
        <p>Current K-line: {currentKline ? 'Open' : 'None'}</p>
      </div>
    </div>
  );
}
```

### 2. Backend Integration

#### 2.1 Microservice Architecture Integration

**Service Discovery Integration:**
```yaml
# docker-compose.yml for microservice setup
version: '3.8'
services:
  k-line-service:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_ENV=production
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/api/v1/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    networks:
      - trading-platform

  api-gateway:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - k-line-service
    networks:
      - trading-platform

networks:
  trading-platform:
    driver: bridge
```

**Nginx Configuration for API Gateway:**
```nginx
# nginx.conf
upstream k-line-backend {
    server k-line-service:8080;
}

server {
    listen 80;
    
    # Proxy REST API requests
    location /api/v1/ {
        proxy_pass http://k-line-backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
    
    # Proxy WebSocket connections
    location /ws {
        proxy_pass http://k-line-backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
    
    # Serve static files
    location / {
        proxy_pass http://k-line-backend;
        proxy_set_header Host $host;
    }
}
```

#### 2.2 Database Integration Pattern

While the current service uses in-memory storage, here's how to integrate with a database:

```rust
// Example database integration pattern
use sqlx::{PgPool, Row};
use chrono::{DateTime, Utc};

pub struct DatabaseKLineService {
    pool: PgPool,
    memory_service: Arc<KLineService>, // Keep in-memory for real-time
}

impl DatabaseKLineService {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;
        let memory_service = Arc::new(KLineService::new());
        
        Ok(Self {
            pool,
            memory_service,
        })
    }
    
    // Process transaction in both memory and database
    pub async fn process_transaction(&self, transaction: &Transaction) -> Result<(), Box<dyn std::error::Error>> {
        // Process in memory for real-time access
        self.memory_service.process_transaction(transaction);
        
        // Persist to database for historical data
        sqlx::query!(
            "INSERT INTO transactions (token, price, volume, timestamp, is_buy) VALUES ($1, $2, $3, $4, $5)",
            transaction.token,
            transaction.price,
            transaction.volume,
            transaction.timestamp,
            transaction.is_buy
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    // Load historical data from database on startup
    pub async fn load_historical_data(&self, hours: i32) -> Result<(), sqlx::Error> {
        let since = Utc::now() - chrono::Duration::hours(hours as i64);
        
        let transactions = sqlx::query!(
            "SELECT * FROM transactions WHERE timestamp > $1 ORDER BY timestamp",
            since
        )
        .fetch_all(&self.pool)
        .await?;
        
        for row in transactions {
            let transaction = Transaction {
                token: row.token,
                price: row.price,
                volume: row.volume,
                timestamp: row.timestamp,
                is_buy: row.is_buy,
            };
            
            self.memory_service.process_transaction(&transaction);
        }
        
        Ok(())
    }
}
```

### 3. Configuration Integration

#### 3.1 Environment-based Configuration

**Production Configuration (`config/production.toml`):**
```toml
[server]
host = "0.0.0.0"
port = 8080

[data_generation]
enabled = false  # Disable mock data in production
interval_ms = 100
volatility_percentage = 1.0

[tokens]
supported = ["BTC", "ETH", "DOGE", "SHIB", "PEPE"]

[integration]
database_url = "postgresql://user:pass@db:5432/trading"
redis_url = "redis://redis:6379"
external_api_key = "${EXTERNAL_API_KEY}"

[monitoring]
metrics_enabled = true
health_check_interval = 30
log_level = "info"
```

**Docker Environment Variables:**
```bash
# .env file for Docker Compose
RUST_ENV=production
DATABASE_URL=postgresql://user:pass@db:5432/trading
REDIS_URL=redis://redis:6379
EXTERNAL_API_KEY=your_api_key_here
LOG_LEVEL=info
```

### 4. Performance Considerations for Integration

#### Benchmarking Results

Based on comprehensive benchmarking, the service provides:

| Metric | Performance | Recommendation |
|--------|-------------|----------------|
| Transaction Processing | 84,745 ops/sec | Suitable for high-frequency trading |
| Concurrent Processing | 5,988 ops/sec | Use connection pooling |
| K-line Queries | 238,095 ops/sec | Excellent for real-time charts |
| WebSocket Broadcasting | >1,000 msgs/sec | Supports many concurrent users |
| Memory Usage | ~50MB (3 tokens, 24h) | Monitor in production |

#### Scaling Recommendations

1. **Horizontal Scaling**: Deploy multiple instances behind a load balancer
2. **Database Integration**: Use PostgreSQL for persistence, keep in-memory for real-time
3. **Caching**: Implement Redis for frequently accessed historical data
4. **CDN**: Use CDN for static assets and web interface
5. **Message Queue**: Use Redis/RabbitMQ for inter-service communication

## Conclusion

The K-Line Data Service is designed for seamless integration into larger trading platforms. Key integration benefits:

- **🚀 High Performance**: Sub-millisecond latency with 84K+ ops/sec
- **🔧 Easy Integration**: RESTful APIs and WebSocket streaming
- **📊 Built-in Monitoring**: Web interface and health checks
- **🐳 Container Ready**: Docker and Kubernetes support
- **🧪 Well Tested**: 27 comprehensive test cases
- **📚 Documented**: Extensive API documentation and examples

The service can be integrated as a standalone microservice or embedded within larger applications, providing real-time trading data capabilities with minimal operational overhead. 