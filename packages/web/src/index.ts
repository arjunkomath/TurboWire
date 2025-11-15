import type { z } from "zod";

type SchemaDefinition = Record<string, z.ZodType>;

type InferSchemaType<T extends SchemaDefinition> = {
  [K in keyof T]: z.infer<T[K]>;
};

type EventHandlersMap<T extends SchemaDefinition> = {
  [K in keyof InferSchemaType<T>]?: Set<(data: InferSchemaType<T>[K]) => void>;
};

export type TurboWireOptions<T extends SchemaDefinition> = {
  /**
   * Enable debug logging
   */
  debug?: boolean;
  /**
   * Maximum number of reconnection attempts
   */
  maxRetries?: number;
  /**
   * Interval between reconnection attempts in milliseconds
   */
  retryInterval?: number;
  /**
   * Zod schema for runtime validation and type inference
   */
  schema: T;
};

export class TurboWire<T extends SchemaDefinition> {
  private ws: WebSocket | undefined = undefined;

  private wireUrl: string;
  private eventHandlers: EventHandlersMap<T> = {};
  private errorCallback?: (error: Event) => void;
  private connectCallback?: () => void;
  private schema: T;

  private retryCount = 0;
  private maxRetries: number;
  private retryInterval: number;
  private retryTimeout?: number;
  private intentionalDisconnect = false;

  private debug: boolean;

  /**
   * @param wireUrl - The URL of the TurboWire server
   * @param options - The options for the TurboWire connection
   */
  constructor(wireUrl: string, options: TurboWireOptions<T>) {
    this.wireUrl = wireUrl;
    this.debug = options.debug || false;
    this.maxRetries = options.maxRetries ?? 10;
    this.retryInterval = options.retryInterval ?? 3000;
    this.schema = options.schema;

    if (this.debug) {
      console.log("Initializing TurboWire with URL", this.wireUrl);
    }
  }

  on<K extends keyof InferSchemaType<T>>(
    event: K,
    handler: (data: InferSchemaType<T>[K]) => void,
  ): void {
    if (!this.eventHandlers[event]) {
      this.eventHandlers[event] = new Set();
    }

    this.eventHandlers[event]?.add(handler);
  }

  off<K extends keyof InferSchemaType<T>>(
    event: K,
    handler?: (data: InferSchemaType<T>[K]) => void,
  ): void {
    if (!handler) {
      delete this.eventHandlers[event];
      return;
    }

    const handlers = this.eventHandlers[event];
    if (handlers) {
      handlers.delete(handler);
      if (handlers.size === 0) {
        delete this.eventHandlers[event];
      }
    }
  }

  connect(onConnect?: () => void, onError?: (error: Event) => void): void {
    this.intentionalDisconnect = false;
    this.connectCallback = onConnect;
    this.errorCallback = onError;
    this.establishConnection();
  }

  private establishConnection(): void {
    if (this.debug) {
      console.log(
        `Connecting to TurboWire server (attempt ${this.retryCount + 1})`,
      );
    }

    this.ws = new WebSocket(this.wireUrl);

    this.ws.onerror = (event: Event) => {
      if (this.debug) {
        console.error("Error on TurboWire connection", event);
      }
      this.errorCallback?.(event);
    };

    this.ws.onopen = () => {
      if (this.debug) {
        console.log("Connected to TurboWire server");
      }
      this.retryCount = 0;
      this.connectCallback?.();

      if (this.ws) {
        this.ws.onmessage = (event: MessageEvent) => {
          if (this.debug) {
            console.log("Received message from TurboWire server", event.data);
          }
          this.handleMessage(event.data);
        };
      }
    };

    this.ws.onclose = (event: CloseEvent) => {
      if (this.debug) {
        console.log(
          "Disconnected from TurboWire server",
          `Code: ${event.code}`,
          `Reason: ${event.reason || "No reason provided"}`,
          `Clean: ${event.wasClean}`,
        );
      }

      this.ws = undefined;

      if (this.intentionalDisconnect || event.code === 1000) {
        return;
      }

      if (!event.wasClean && this.retryCount < this.maxRetries) {
        this.retryCount++;
        if (this.debug) {
          console.log(
            `Attempting reconnection in ${this.retryInterval}ms (attempt ${this.retryCount}/${this.maxRetries})`,
          );
        }
        this.retryTimeout = setTimeout(() => {
          this.establishConnection();
        }, this.retryInterval);
      }
    };
  }

  private handleMessage(rawMessage: string): void {
    try {
      const { event, data } = JSON.parse(rawMessage);

      if (!event) {
        if (this.debug) {
          console.warn("Received message without event type", rawMessage);
        }
        return;
      }

      if (this.schema[event]) {
        const validation = this.schema[event].safeParse(data);
        if (!validation.success) {
          if (this.debug) {
            console.error(
              `Validation failed for event "${String(event)}"`,
              validation.error.message,
            );
          }
          return;
        }
      }

      const handlers = this.eventHandlers[event];
      if (handlers) {
        for (const handler of handlers) {
          try {
            handler(data);
          } catch (error) {
            if (this.debug) {
              console.error(
                `Error in event handler for "${String(event)}"`,
                error,
              );
            }
          }
        }
      } else if (this.debug) {
        console.warn(`No handler registered for event "${String(event)}"`);
      }
    } catch (error) {
      if (this.debug) {
        console.error("Failed to parse message", rawMessage, error);
      }
    }
  }

  emit<K extends keyof InferSchemaType<T>>(
    event: K,
    data: InferSchemaType<T>[K],
  ): void {
    if (!this.ws) {
      throw new Error("WebSocket is not connected");
    }

    const message = { event, data };

    if (this.schema[event]) {
      const validation = this.schema[event].safeParse(data);
      if (!validation.success) {
        throw new Error(
          `Validation failed for event "${String(event)}": ${validation.error.message}`,
        );
      } else {
        message.data = validation.data;
      }
    }

    if (this.debug) {
      console.log("Emitting event to TurboWire server", event, data);
    }

    this.ws.send(JSON.stringify(message));
  }

  /**
   * Disconnect from the TurboWire server
   */
  disconnect(): void {
    this.intentionalDisconnect = true;

    if (this.retryTimeout) {
      clearTimeout(this.retryTimeout);
      this.retryTimeout = undefined;
    }

    if (this.ws) {
      if (this.debug) {
        console.log("Disconnecting from TurboWire server");
      }
      this.ws.close(1000, "Client disconnect");
    }

    this.retryCount = 0;
  }
}
