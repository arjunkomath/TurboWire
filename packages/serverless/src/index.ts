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

    return fetch(`${this.serverUrl}/sign-wire`, {
      method: 'POST',
      body: JSON.stringify({ room }),
      headers: {
        'Content-Type': 'application/json',
        'x-api-key': this.signingKey,
      },
    })
      .then((res) => res.json())
      .then((data) => {
        return data.url;
      });
  }
}
