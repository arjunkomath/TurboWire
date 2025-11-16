import { TurboWire } from "@turbowire/web";
import { useEffect, useRef } from "react";
import type { z } from "zod";

type SchemaDefinition = Record<string, z.ZodType>;

type InferSchemaType<T extends SchemaDefinition> = {
  [K in keyof T]: z.infer<T[K]>;
};

type EventHandlers<T extends SchemaDefinition> = {
  [K in keyof InferSchemaType<T>]?: (data: InferSchemaType<T>[K]) => void;
};

export type WireEventOptions<T extends SchemaDefinition> = {
  schema: T;
  debug?: boolean;
  maxRetries?: number;
  retryInterval?: number;
} & EventHandlers<T>;

export function useWireEvent<T extends SchemaDefinition>(
  wireUrl: string,
  options: WireEventOptions<T>,
): void {
  const turboWireRef = useRef<TurboWire<T> | null>(null);

  useEffect(() => {
    const { schema, debug, maxRetries, retryInterval, ...eventHandlers } =
      options;

    turboWireRef.current = new TurboWire(wireUrl, {
      schema,
      debug,
      maxRetries,
      retryInterval,
    });

    const registeredHandlers: Array<{
      event: keyof InferSchemaType<T>;
      handler: (data: InferSchemaType<T>[keyof InferSchemaType<T>]) => void;
    }> = [];

    const eventKeys = Object.keys(schema);
    for (const event of eventKeys) {
      const handler = (eventHandlers as Record<string, unknown>)[event];
      if (typeof handler === "function") {
        const typedHandler = handler as (
          data: InferSchemaType<T>[keyof InferSchemaType<T>]
        ) => void;
        const typedEvent = event as keyof InferSchemaType<T>;

        turboWireRef.current.on(typedEvent, typedHandler);
        registeredHandlers.push({ event: typedEvent, handler: typedHandler });
      }
    }

    turboWireRef.current.connect();

    return () => {
      for (const { event, handler } of registeredHandlers) {
        turboWireRef.current?.off(event, handler);
      }
      turboWireRef.current?.disconnect();
      turboWireRef.current = null;
    };
  }, [wireUrl, options]);
}
