# Read Engine

Read Engine is a high-performance data processing system designed to efficiently handle real-time updates and ordering of cryptocurrency-related data. It's built with Rust for optimal speed and reliability.

## Features

- Real-time data processing for cryptocurrency events
- Efficient ordering algorithms for various crypto metrics
- WebSocket support for live updates
- Integration with PostgreSQL for data persistence
- Redis caching for improved performance
- Modular architecture for easy maintenance and scalability

## Rest API

For detailed usage, refer to the [API documentation](https://read-engine.nad.fun/swagger-ui).

## WebSocket RPC Interface

Read Engine provides a WebSocket-based RPC interface for real-time data subscription and updates.

### Connection

To connect to the WebSocket server:
ws://read-engine.nad.fun/ws

### RPC Methods

The WebSocket interface supports the following RPC methods:

1. Order Subscribe
2. Coin Subscribe

3. Order Subscribe
   Subscribe to real-time updates for a specific order type.
   Request:

```json
{
  "jsonrpc": "2.0",
  "method": "order_subscribe",
  "params": {
    "order_type": "creation_time"
  },
  "id": 1
}
```

Response:

```json
{
  "jsonrpc": "2.0",
  "method": "order_subscribe",
  "result": {
    "status": "subscribed",
    "data": {
      "message_type": "ALL",
      "new_token": { ... },
      "new_swap": { ... },
      "order_type": "creation_time",
      "order_token": [ ... ]
    }
  }
}
```

2. Coin Subscribe
   Subscribe to real-time updates for a specific coin.
   Request:

```json
{
  "jsonrpc": "2.0",
  "method": "coin_subscribe",
  "params": {
    "coin_id": "example_coin_id"
  },
  "id": 2
}
```

Response:

```json
{
  "jsonrpc": "2.0",
  "method": "coin_subscribe",
  "result": {
    "status": "subscribed",
    "data": {
      "message_type": "ALL",
      "new_token": { ... },
      "new_swap": { ... },
      "coin": { ... }
    }
  }
}
```

Usage Example
Here's a simple JavaScript example using the WebSocket API:

```javascript
const ws = new WebSocket("ws://read-engine.nad.fun/ws");

ws.onopen = () => {
  console.log("Connected to Read Engine WebSocket");

  // Subscribe to creation time order updates
  ws.send(
    JSON.stringify({
      jsonrpc: "2.0",
      method: "order_subscribe",
      params: { order_type: "creation_time" },
      id: 1,
    })
  );
};

ws.onmessage = (event) => {
  const response = JSON.parse(event.data);
  console.log("Received update:", response);
  // Handle the update in your application
};

ws.onerror = (error) => {
  console.error("WebSocket error:", error);
};

ws.onclose = () => {
  console.log("Disconnected from Read Engine WebSocket");
};
```
