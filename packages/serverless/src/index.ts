import { createHmac } from "node:crypto";

export class TurboWireHub {
  private domain: string;
  private broadcastKey: string;
  private signingKey: string;

  private broadcastUrl: string;
  private wireUrl: string;

  constructor(domain: string, options?: {
    broadcastKey?: string;
    signingKey?: string;
    secure?: boolean;
  }) {
    if (!domain) {
      throw new Error('domain is required');
    }

    const broadcastKey = options?.broadcastKey ?? process.env.TURBOWIRE_BROADCAST_KEY;
    const signingKey = options?.signingKey ?? process.env.TURBOWIRE_SIGNING_KEY;
    const secure = options?.secure ?? true;

    if (!broadcastKey) {
      throw new Error('TurboWire broadcast key is required, either as an option or as the TURBOWIRE_BROADCAST_KEY environment variable');
    }
    if (!signingKey) {
      throw new Error('TurboWire signing key is required, either as an option or as the TURBOWIRE_SIGNING_KEY environment variable');
    }

    this.domain = domain;
    this.broadcastKey = broadcastKey;
    this.signingKey = signingKey;

    this.broadcastUrl = `${secure ? 'https' : 'http'}://${this.domain}/broadcast`;
    this.wireUrl = `${secure ? 'wss' : 'ws'}://${this.domain}`;
  }

  async broadcast(room: string, message: string): Promise<Response> {
    return fetch(this.broadcastUrl, {
      method: 'POST',
      body: JSON.stringify({ room, message }),
      headers: {
        'Content-Type': 'application/json',
        'x-broadcast-key': this.broadcastKey,
      },
    }).then((res) => {
      if (!res.ok) {
        throw new Error('Failed to broadcast message');
      }
      return res.json();
    });
  }

  getSignedWire(room: string): string {
    if (!room) {
      throw new Error('Room is required');
    }

    const hmac = createHmac('sha256', this.signingKey);
    hmac.update(room);
    const signature = hmac.digest('base64url');

    return `${this.wireUrl}?room=${encodeURIComponent(room)}&signature=${signature}`;
  }
}
