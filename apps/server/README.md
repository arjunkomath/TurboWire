# TurboWire Server

The TurboWire server is a Rust application that accepts WebSocket connections from the client and has APIs to broadcast messages to rooms.

## Getting Started

```bash
cargo run
```

## Environment Variables

| Variable | Description |
| -------- | -------- |
| `CONNECTION_LIMIT` | The maximum number of connections to allow |
| `MESSAGE_WEBHOOK_URL` | The URL to send messages to |
| `BROADCAST_KEY` | The key to use for authenticating broadcast requests |
| `SIGNING_KEY` | The key to use for signing messages |
| `HOST` | The host address to listen on |
| `PORT` | The port to listen on |

## Endpoints

### GET `/`

WebSocket endpoint that the client connects to. This endpoint requires a room name and a valid signature.
Signature is a cryptographic hash of the room name using HMAC-SHA256 with the signing key and is base64url encoded without padding.

### POST `/broadcast`

Endpoint that the server can use to broadcast messages to connected clients.
Broadcast requests are authenticated using the `BROADCAST_KEY` and need to include a `room` and `message` in the body.

```curl
curl --request POST \
  --url https://your-server.com/broadcast \
  --header 'Content-Type: application/json' \
  --header 'x-broadcast-key: some-key' \
  --data '{
  "message": "hello world",
  "room": "notifications_user_123"
}'
```

### GET `/health`

Endpoint that can be used to check the health of the server.




