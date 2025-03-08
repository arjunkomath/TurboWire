export type TurboWireOptions = {
  debug?: boolean;
};

export type TurboWireConnection = {
  disconnect: () => void;
  send: (message: string) => void;
};

export class TurboWire {
  private ws: WebSocket | null = null;
  private wireUrl: string;
  private debug: boolean;

  constructor(wireUrl: string, options?: TurboWireOptions) {
    this.wireUrl = wireUrl;
    this.debug = options?.debug || false;

    if (this.debug) {
      console.log('Initializing TurboWire with URL', this.wireUrl);
    }
  }

  connect(
    onMessage: (message: string) => void,
    onError?: (error: Event) => void
  ): TurboWireConnection {
    if (this.debug) {
      console.log('Connecting to TurboWire server', this.wireUrl);
    }
    this.ws = new WebSocket(this.wireUrl);

    this.ws.onerror = (event: Event) => {
      if (this.debug) {
        console.error('Error on TurboWire connection', event);
      }
      onError?.(event);
    };

    this.ws.onopen = () => {
      if (this.debug) {
        console.log('Connected to TurboWire server');
      }

      if (this.ws) {
        this.ws.onmessage = (event: MessageEvent) => {
          if (this.debug) {
            console.log('Received message from TurboWire server', event.data);
          }
          onMessage(event.data);
        };
      } else {
        throw new Error('WebSocket is not connected');
      }
    };

    this.ws.onclose = (event: CloseEvent) => {
      if (this.debug) {
        console.log(
          'Disconnected from TurboWire server',
          `Code: ${event.code}`,
          `Reason: ${event.reason || 'No reason provided'}`,
          `Clean: ${event.wasClean}`
        );
      }
    };

    return {
      disconnect: () => {
        this.disconnect();
      },
      send: (message: string) => {
        this.send(message);
      },
    };
  }

  private send(message: string): void {
    if (this.ws) {
      if (this.debug) {
        console.log('Sending message to TurboWire server', message);
      }
      this.ws.send(message);
    } else {
      throw new Error('WebSocket is not connected');
    }
  }

  private disconnect(): void {
    if (this.ws) {
      if (this.debug) {
        console.log('Disconnecting from TurboWire server');
      }
      this.ws.close();
      this.ws = null;
    }
  }
}
