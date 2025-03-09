# Turbowire Serverless

This is a helper package to make it easy to work with TurboWire.

## Getting Started

```bash
npm install @turbowire/serverless
```

Make sure you've set the following environment variables, or passed them to the constructor:

- `TURBOWIRE_DOMAIN` - The domain of the TurboWire server, e.g. `wire.managee.xyz`
- `TURBOWIRE_BROADCAST_KEY` - The broadcast key for the TurboWire server
- `TURBOWIRE_SIGNING_KEY` - The signing key for the TurboWire server

## Usage

### Create signed wire

```ts
import { TurboWireHub } from "@turbowire/serverless";

const turbowire = new TurboWireHub(process.env.TURBOWIRE_DOMAIN!); // TURBOWIRE_DOMAIN is the domain of the TurboWire server, e.g. wire.managee.xyz
const wireUrl = turbowire.getSignedWire('room-name');
```

### Broadcast message

```ts
import { TurboWireHub } from "@turbowire/serverless";

const turbowire = new TurboWireHub(process.env.TURBOWIRE_DOMAIN!); // TURBOWIRE_DOMAIN is the domain of the TurboWire server, e.g. wire.managee.xyz
await turbowire.broadcast('room-name', 'message');
```

For full list of options, check function docs.


