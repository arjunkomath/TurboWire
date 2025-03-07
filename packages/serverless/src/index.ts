import { createHmac } from "node:crypto";

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
  private wireUrl: string;
  private signingKey: string;

  constructor(wireUrl: string, signingKey: string) {
    if (!wireUrl) {
      throw new Error('wireUrl is required');
    }
    if (!signingKey) {
      throw new Error('signingKey is required');
    }

    this.wireUrl = wireUrl;
    this.signingKey = signingKey;
  }

  getSignedWire(room: string): string {
    if (!room) {
      throw new Error('Room is required');
    }

    const hmac = createHmac('sha256', this.signingKey);
    hmac.update(room);
    const signature = hmac.digest('base64url');

    return `${this.wireUrl}?room=${room}&signature=${signature}`;
  }
}
