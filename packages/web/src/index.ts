export type TurboWireOptions = {
  debug?: boolean;
};

export class TurboWire {
  private ws: WebSocket | null = null;
  private wireUrl: string;
  private debug: boolean;

  /**
   * @param wireUrl - The URL of the TurboWire server
   * @param options - The options for the TurboWire connection
   */
  constructor(wireUrl: string, options?: TurboWireOptions) {
    this.wireUrl = wireUrl;
    this.debug = options?.debug || false;

    if (this.debug) {
      console.log('Initializing TurboWire with URL', this.wireUrl);
    }
  }

  /**
   * Connect to the TurboWire server via a WebSocket
   * @param onMessage - The callback for when a message is received from the server
   * @param onError - The callback for when an error occurs
   */
  connect(onMessage: (message: string) => void, onError?: (error: Event) => void) {
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
  }

  /**
   * Send a message to the TurboWire server
   * @param message - The message to send
   */
  send(message: string): void {
    if (this.ws) {
      if (this.debug) {
        console.log('Sending message to TurboWire server', message);
      }
      this.ws.send(message);
    } else {
      throw new Error('WebSocket is not connected');
    }
  }

  /**
   * Disconnect from the TurboWire server
   */
  disconnect(): void {
    if (this.ws) {
      if (this.debug) {
        console.log('Disconnecting from TurboWire server');
      }
      this.ws.close();
      this.ws = null;
    }
  }
}
