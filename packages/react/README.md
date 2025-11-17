# @turbowire/react

React hooks for TurboWire - Real-time WebSocket communication for Serverless applications.

## Installation

```bash
npm install @turbowire/react zod
```

**Peer Dependencies:**

- `react`: ^18.0.0 || ^19.0.0
- `zod`: ^3.0.0 || ^4.0.0

## Quick Start

```tsx
import { useWireEvent } from "@turbowire/react";
import { z } from "zod";

function App() {
  const messageSchema = z.object({
    content: z.string(),
    timestamp: z.number(),
  });

  const notificationSchema = z.object({
    type: z.enum(["info", "warning", "error"]),
    message: z.string(),
  });

  useWireEvent("wss://your-turbowire-server.com?room=lobby&signature=abc123", {
    schema: {
      message: messageSchema,
      notification: notificationSchema,
    },
    message: (data) => {
      console.log("Message received:", data.content);
    },
    notification: (data) => {
      console.log(`[${data.type}]`, data.message);
    },
    debug: true,
  });

  return <div>Your App</div>;
}
```

## API Reference

### `useWireEvent<T>(wireUrl, options)`

A React hook that manages WebSocket connections and event handlers.

#### Parameters

- **`wireUrl`** (string): The WebSocket URL (typically a signed URL from `@turbowire/serverless`)
- **`options`** (WireEventOptions): Configuration options

#### Options

| Option          | Type                      | Default      | Description                       |
| --------------- | ------------------------- | ------------ | --------------------------------- |
| `schema`        | `Record<string, ZodType>` | **Required** | Zod schemas for each event type   |
| `debug`         | `boolean`                 | `false`      | Enable debug logging              |
| `maxRetries`    | `number`                  | `10`         | Maximum reconnection attempts     |
| `retryInterval` | `number`                  | `3000`       | Delay between retries (ms)        |
| `[eventName]`   | `(data) => void`          | -            | Event handler for each schema key |

## How It Works

1. **Connection**: The hook creates a TurboWire connection when the component mounts
2. **Event Registration**: All event handlers are automatically registered based on the schema keys
3. **Type Safety**: Zod validates incoming messages and provides type inference
4. **Cleanup**: On unmount, all handlers are removed and the connection is closed
5. **Reconnection**: If the connection drops, automatic reconnection is attempted

## Integration with Serverless

Use with `@turbowire/serverless` to create signed URLs:

```tsx
import { useWireEvent } from "@turbowire/react";
import { useEffect, useState } from "react";

function App() {
  const [wsUrl, setWsUrl] = useState(null);

  useEffect(() => {
    fetch("/api/get-ws-url")
      .then((res) => res.json())
      .then((data) => setWsUrl(data.url));
  }, []);

  if (!wsUrl) return <div>Connecting...</div>;

  return <ChatComponent wsUrl={wsUrl} />;
}
```

Server-side (Next.js API route example):

```typescript
import { createTurboWireHub } from "@turbowire/serverless";
import { z } from "zod";

const schema = {
  message: z.object({
    content: z.string(),
    timestamp: z.number(),
  }),
};

const hub = createTurboWireHub("your-turbowire-server.com", {
  schema,
  broadcastKey: process.env.TURBOWIRE_BROADCAST_KEY,
  signingKey: process.env.TURBOWIRE_SIGNING_KEY,
});

export async function GET() {
  const url = hub.getSignedWire("chat-room-1");
  return Response.json({ url });
}
```

## TypeScript

The hook provides full type inference for event handlers based on your Zod schemas:

```tsx
const schema = {
  userUpdate: z.object({
    id: z.string(),
    name: z.string(),
    age: z.number(),
  }),
};

useWireEvent(url, {
  schema,
  userUpdate: (data) => {
    console.log(data.id);
    console.log(data.name);
    console.log(data.age);
  },
});
```

## License

MIT

## Related Packages

- [`@turbowire/web`](../web) - Core WebSocket client
- [`@turbowire/serverless`](../serverless) - Server-side utilities for serverless functions
