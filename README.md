# TurboWire

TurboWire is a WebSocket server for Serverless applications. Since serverless functions are stateless and have a limited lifetime, they're unable to maintain a long-lived connection to the client. TurboWire solves this problem by providing a WebSocket server that can be used by the client to send and receive messages to the server.

You can use TurboWire to build real-time applications like chat applications, notifications, and more. Built using Rust, it's a lightweight and fast WebSocket server that can be used in any project. It also enables secure communication between the client and server using signed WebSocket URLs and rooms.

The project consists of three parts:

- `turbowire-server`: A WebSocket server which the client can connect to. You can deploy this using any Cloud provider that supports Docker.
- `@turbowire/web`: A JavaScript client that runs in the browser and connects to TurboWire server.
- `@turbowire/serverless`: A Node.JS library that lets you create signed WebSocket URLs and broadcast message to rooms.

![image](https://github.com/user-attachments/assets/7ffa145c-44e8-4c4c-8236-d0e62653e0d1)

## Getting Started

### TurboWire Server

You can deploy the server using the provided Dockerfile or use the pre-built image from Docker Hub.

```bash
docker pull arjunkomath/turbowire-server:latest
docker run -d -p 8080:8080 arjunkomath/turbowire-server:latest
```
You can find the required environment variables in the `.env.sample` file.

You can read more about the server in the [server README](./apps/server/README.md).

### Your Project (Client)

You can use the client in your project by installing the package using npm or yarn.

```bash
npm install @turbowire/web
```

You can read more about the client in the [client README](./packages/web/README.md).

### Your Project (Serverless)

You can use the serverless package to create signed WebSocket URLs and broadcast messages to rooms.

```bash
npm install @turbowire/serverless
```

You can read more about the serverless package in the [serverless README](./packages/serverless/README.md).

## Contributing

Contributions are welcome! Please open an issue or submit a PR.
