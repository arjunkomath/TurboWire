# TurboWire Serverless

Server-side package for broadcasting messages and creating signed wire connections with type safety.

## Getting Started

```bash
npm install @turbowire/serverless zod
```

Set these environment variables or pass them to the constructor:

- `TURBOWIRE_DOMAIN` - Your TurboWire server domain (e.g. `wire.managee.xyz`)
- `TURBOWIRE_BROADCAST_KEY` - Broadcast key
- `TURBOWIRE_SIGNING_KEY` - Signing key

## Usage

### Define your schema

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

### Broadcast messages to rooms

```ts
import { createTurboWireHub } from "@turbowire/serverless";

const hub = createTurboWireHub(process.env.TURBOWIRE_DOMAIN!, {
  schema,
});

await hub.broadcast("room-name").userJoined({
  userId: "123",
  name: "Alice",
});

await hub.broadcast("room-name").chatMessage({
  text: "Hello!",
  timestamp: Date.now(),
});
```

Events are fully typed and validated at runtime. TypeScript autocompletes event names and validates payloads, while zod ensures data integrity.

### Create signed wire URLs

```ts
const wireUrl = hub.getSignedWire("room-name");
```

Pass this URL to your frontend to establish a connection.

### Type helpers

Extract types from your schema:

```ts
import type { EventNames, EventPayload } from "@turbowire/serverless";

type Events = EventNames<typeof schema>;
type UserJoinedPayload = EventPayload<typeof schema, "userJoined">;
```
