const WebSocket = require('ws');

console.log('Testing WebSocket connection to ws://localhost:8080/ws');

const ws = new WebSocket('ws://localhost:8080/ws');

ws.on('open', function open() {
    console.log('✅ WebSocket connected successfully!');
    
    // Send a subscription message
    const subscribeMessage = {
        action: 'subscribe',
        subscription: {
            type: 'all_transactions'
        }
    };
    
    console.log('📤 Sending subscription:', JSON.stringify(subscribeMessage));
    ws.send(JSON.stringify(subscribeMessage));
    
    // Close after 5 seconds
    setTimeout(() => {
        console.log('🔌 Closing connection...');
        ws.close();
    }, 5000);
});

ws.on('message', function message(data) {
    console.log('📥 Received:', data.toString());
});

ws.on('error', function error(err) {
    console.error('❌ WebSocket error:', err.message);
});

ws.on('close', function close() {
    console.log('🔌 WebSocket connection closed');
    process.exit(0);
});

// Timeout after 10 seconds
setTimeout(() => {
    console.error('⏰ Connection timeout');
    process.exit(1);
}, 10000); 