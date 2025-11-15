# TurboWire Web

Frontend package for real-time messaging with type safety and automatic reconnection.

## Getting Started

```bash
npm install @turbowire/web zod
```

## Usage

### Define your schema

Share the same schema between frontend and backend:

```ts
import { z } from "zod";

const schema = {
  userJoined: z.object({
    userId: z.string(),
    name: z.string(),
  }),
  chatMessage: z.object({
    text: z.string(),
    timestamp: z.number(),
  }),
};
```

### React example

```tsx
import { TurboWire } from "@turbowire/web";
import { useEffect } from "react";

function ChatRoom({ signedWire }) {
  useEffect(() => {
    const wire = new TurboWire(signedWire, { schema });

    const handleUserJoined = (data) => {
      console.log(`${data.name} joined`);
    };

    const handleChatMessage = (data) => {
      console.log(data.text);
    };

    wire.on("userJoined", handleUserJoined);
    wire.on("chatMessage", handleChatMessage);
    wire.connect();

    return () => {
      wire.off("userJoined", handleUserJoined);
      wire.off("chatMessage", handleChatMessage);
      wire.disconnect();
    };
  }, [signedWire]);
}
```

### Send messages from client

```ts
wire.emit("chatMessage", {
  text: "Hello!",
  timestamp: Date.now(),
});
```

### Remove event handlers

```ts
const handler = (data) => console.log(data.text);

wire.on('chatMessage', handler);
wire.off('chatMessage', handler);
```

To remove all handlers for an event:

```ts
wire.off('chatMessage');
```

Events are fully typed and validated at runtime. The connection automatically reconnects if it drops.

### Type helpers

Share types across your app:

```ts
import type { EventNames, EventPayload } from "@turbowire/web";

type Events = EventNames<typeof schema>;
type ChatMessagePayload = EventPayload<typeof schema, "chatMessage">;
```
