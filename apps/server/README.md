# TurboWire Server

The TurboWire server is a Rust application that accepts WebSocket connections from the client and has APIs to broadcast messages to rooms.

## Getting Started

Development server

```bash
cargo run
```

Production, use provided Dockerfile or prebuilt image from Docker Hub.

```bash
docker run -d -p 8080:8080 -e BROADCAST_KEY=your_broadcast_key -e SIGNING_KEY=your_signing_key arjunkomath/turbowire-server:latest
```

## Environment Variables

| Variable | Description | Default | Required |
| -------- | -------- | -------- | -------- |
| `BROADCAST_KEY` | The key to use for authenticating broadcast requests | - | Yes |
| `SIGNING_KEY` | The key to use for signing wire | - | Yes |
| `CONNECTION_LIMIT` | The maximum number of concurrent connections | `1000` | No |
| `MESSAGE_WEBHOOK_URL` | The URL to send client messages | - | No |
| `REDIS_URL` | Redis server for message queue | - | No |
| `HOST` | The host address to listen on | `0.0.0.0` | No |
| `PORT` | The port to listen on | `8080` | No |

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




