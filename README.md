# TurboWire

TurboWire is a WebSocket server for Serverless applications. Since serverless functions are stateless and have a limited lifetime, they're unable to maintain a long-lived connection to the client. TurboWire solves this problem by providing a WebSocket server that can be used by the client to send and receive messages to the server.

You can use TurboWire to build real-time applications like chat applications, notifications, and more. Built using Rust, it's a lightweight and fast WebSocket server that can be used in any project. It also enables secure communication between the client and server using signed WebSocket URLs and rooms.

The project consists of three parts:

- `turbowire-server`: A WebSocket server which the client can connect to. You can deploy this using any Cloud provider that supports Docker.
- `@turbowire/web`: A JavaScript client that runs in the browser and connects to TurboWire server.
- `@turbowire/serverless`: A Node.JS library that lets you create signed WebSocket URLs and broadcast message to rooms.

![image](https://github.com/user-attachments/assets/2e3f2cbb-95e0-42da-ac32-99794923436a)

## Getting Started

### TurboWire Server

You can deploy the server using the provided Dockerfile or use the pre-built image from Docker Hub.

```bash
docker pull arjunkomath/turbowire-server:latest
docker run -d -p 8080:8080 arjunkomath/turbowire-server:latest
```
You can find the required environment variables in the `.env.sample` file.

### Your Project (Client)

You can use the client in your project by installing the package using npm or yarn.

```bash
npm install @turbowire/web
```

### Your Project (Serverless)

You can use the serverless package to create signed WebSocket URLs and broadcast messages to rooms.

```bash
npm install @turbowire/serverless
```
