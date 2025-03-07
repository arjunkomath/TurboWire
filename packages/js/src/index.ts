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

export class TurboWireBroadcaster {
  private serverUrl: string;
  private apiKey: string;

  constructor(serverUrl: string, apiKey: string) {
    if (!serverUrl) {
      throw new Error('serverUrl is required');
    }
    if (!apiKey) {
      throw new Error('apiKey is required');
    }

    this.serverUrl = serverUrl;
    this.apiKey = apiKey;
  }

  broadcast(message: string, room?: string | undefined): Promise<Response> {
    return fetch(`${this.serverUrl}/broadcast`, {
      method: 'POST',
      body: JSON.stringify({ message, room }),
      headers: {
        'Content-Type': 'application/json',
        'x-api-key': this.apiKey,
      },
    });
  }
}

export class TurboWireSigner {
  private serverUrl: string;
  private signingKey: string;

  constructor(serverUrl: string, signingKey: string) {
    if (!serverUrl) {
      throw new Error('serverUrl is required');
    }
    if (!signingKey) {
      throw new Error('signingKey is required');
    }

    this.serverUrl = serverUrl;
    this.signingKey = signingKey;
  }

  async getSignedWire(room: string): Promise<string> {
    if (!room) {
      throw new Error('Room is required');
    }

    return fetch(`${this.serverUrl}/client-token`, {
      method: 'POST',
      body: JSON.stringify({ room }),
      headers: {
        'Content-Type': 'application/json',
        'x-api-key': this.signingKey,
      },
    })
      .then((res) => res.json())
      .then((data) => {
        return data.signed_url;
      });
  }
}
