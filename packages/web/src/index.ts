export type TurboWireOptions = {
  debug?: boolean;
  maxRetries?: number;
  retryInterval?: number;
};

export class TurboWire {
  private ws: WebSocket | null = null;

  private wireUrl: string;
  private messageCallback?: (message: string) => void;
  private errorCallback?: (error: Event) => void;

  private retryCount = 0;
  private maxRetries: number;
  private retryInterval: number;
  private retryTimeout?: number;

  private debug: boolean;

  /**
   * @param wireUrl - The URL of the TurboWire server
   * @param options - The options for the TurboWire connection
   */
  constructor(wireUrl: string, options?: TurboWireOptions) {
    this.wireUrl = wireUrl;
    this.debug = options?.debug || false;
    this.maxRetries = options?.maxRetries ?? 10;
    this.retryInterval = options?.retryInterval ?? 3000;

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
    this.messageCallback = onMessage;
    this.errorCallback = onError;

    this.establishConnection();
  }

  private establishConnection(): void {
    if (this.debug) {
      console.log(`Connecting to TurboWire server (attempt ${this.retryCount + 1})`);
    }

    this.ws = new WebSocket(this.wireUrl);

    this.ws.onerror = (event: Event) => {
      if (this.debug) {
        console.error('Error on TurboWire connection', event);
      }
      this.errorCallback?.(event);
    };

    this.ws.onopen = () => {
      if (this.debug) {
        console.log('Connected to TurboWire server');
      }
      this.retryCount = 0;

      if (this.ws) {
        this.ws.onmessage = (event: MessageEvent) => {
          if (this.debug) {
            console.log('Received message from TurboWire server', event.data);
          }
          this.messageCallback?.(event.data);
        };
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

      if (!event.wasClean && this.retryCount < this.maxRetries) {
        this.retryCount++;
        if (this.debug) {
          console.log(
            `Attempting reconnection in ${this.retryInterval}ms (attempt ${this.retryCount}/${this.maxRetries})`
          );
        }
        this.retryTimeout = setTimeout(() => {
          this.establishConnection();
        }, this.retryInterval);
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
    if (this.retryTimeout) {
      clearTimeout(this.retryTimeout);
      this.retryTimeout = undefined;
    }

    if (this.ws) {
      if (this.debug) {
        console.log('Disconnecting from TurboWire server');
      }
      this.ws.close();
      this.ws = null;
    }

    this.retryCount = 0;
  }
}
