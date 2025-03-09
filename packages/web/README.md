# Turbowire Web

This is a helper package to make it easy to work with TurboWire on your frontend.

## Getting Started

```bash
npm install @turbowire/web
```

## Usage

### React Example

Lets you've a React component that needs to receive notifications. Start by creating a signed wire on the server and passing it to the component. Below is an example of a component that receives messsages from the wire and logs them to the console.

```tsx
import { TurboWire } from "@turbowire/web";

useEffect(() => {
    let wire: TurboWire | undefined;

    wire = new TurboWire(signedWire);
    wire.connect((message) => {
        console.log(message);
    });

    return () => {
        wire?.disconnect();
    };
}, [signedWire, checkNotifications]);

```

You can also send messages to the wire using the `send` method.

```tsx
wire.send('message');
```

