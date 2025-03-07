export class TurboWire {
  private ws: WebSocket | null = null;
  private wireUrl: string;
  private debug: boolean;

  constructor(wireUrl: string, options: { debug?: boolean } = {}) {
    this.wireUrl = wireUrl;
    this.debug = options.debug || false;
  }

  connect(
    onMessage: (message: string) => void,
    onError: (error: Event) => void
  ): Promise<{
    disconnect: () => void;
  }> {
    return new Promise((resolve, reject) => {
      try {
        if (this.debug) {
          console.log('Connecting to TurboWire server', this.wireUrl);
        }
        this.ws = new WebSocket(this.wireUrl);

        this.ws.onopen = () => {
          if (this.debug) {
            console.log('Connected to TurboWire server');
          }

          if (this.ws) {
            this.ws.onmessage = (event: MessageEvent) => {
              onMessage(event.data);
            };

            this.ws.onerror = (event: Event) => {
              onError(event);
            };
          }

          resolve({
            disconnect: () => {
              this.disconnect();
            },
          });
        };

        this.ws.onclose = () => {
          if (this.debug) {
            console.log('Disconnected from TurboWire server');
          }
        };
      } catch (error) {
        reject(error);
      }
    });
  }

  private disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}
