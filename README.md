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

## Benchmark

Test on a 1vCPU, 2GB RAM instance on Railway, 500 concurrent connections over home internet.

```bash
     ✓ status is 101

     checks................: 100.00% 2500 out of 2500
     data_received.........: 8.9 MB  246 kB/s
     data_sent.............: 2.0 MB  55 kB/s
     iteration_duration....: avg=6.81s    min=6.3s     med=6.38s    max=10.58s p(90)=8.68s p(95)=9.7s
     iterations............: 2500    69.097428/s
     vus...................: 30      min=30           max=500
     vus_max...............: 500     min=500          max=500
     ws_connecting.........: avg=816.22ms min=301.21ms med=381.38ms max=4.58s  p(90)=2.68s p(95)=3.7s
     ws_msgs_sent..........: 2500    69.097428/s
     ws_session_duration...: avg=5.81s    min=5.3s     med=5.38s    max=9.58s  p(90)=7.68s p(95)=8.7s
     ws_sessions...........: 2500    69.097428/s


running (0m36.2s), 000/500 VUs, 2500 complete and 0 interrupted iterations
```

## Contributing

Contributions are welcome! Please open an issue or submit a PR.
