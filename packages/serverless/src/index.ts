import { createHmac } from 'node:crypto';

export type TurboWireOptions = {
  /**
   * Optional override for the broadcast endpoint
   * Useful if you've set up a private endpoint for server-to-server communication
   */
  broadcastUrl?: string;
  /**
   * Optional override for the wire endpoint
   */
  wireUrl?: string;
  /**
   * The broadcast key for the TurboWire server
   */
  broadcastKey?: string;
  /**
   * The signing key for the TurboWire server
   */
  signingKey?: string;
  /**
   * Whether to use secure (https) connections
   * Defaults to true
   */
  secure?: boolean;
};

export class TurboWireHub {
  private domain: string;
  private broadcastKey: string;
  private signingKey: string;

  private broadcastUrl: string;
  private wireUrl: string;

  /**
   * @param domain - The domain of the TurboWire server
   * @param options - The options for the TurboWire server
   */
  constructor(domain: string, options?: TurboWireOptions) {
    if (!domain) {
      throw new Error('domain is required');
    }

    const broadcastKey = options?.broadcastKey ?? process.env.TURBOWIRE_BROADCAST_KEY;
    const signingKey = options?.signingKey ?? process.env.TURBOWIRE_SIGNING_KEY;
    const secure = options?.secure ?? true;

    if (!broadcastKey) {
      throw new Error(
        'TurboWire broadcast key is required, either as an option or as the TURBOWIRE_BROADCAST_KEY environment variable'
      );
    }
    if (!signingKey) {
      throw new Error(
        'TurboWire signing key is required, either as an option or as the TURBOWIRE_SIGNING_KEY environment variable'
      );
    }

    this.domain = domain;
    this.broadcastKey = broadcastKey;
    this.signingKey = signingKey;

    this.broadcastUrl =
      options?.broadcastUrl ?? `${secure ? 'https' : 'http'}://${this.domain}/broadcast`;
    this.wireUrl = options?.wireUrl ?? `${secure ? 'wss' : 'ws'}://${this.domain}`;
  }

  /**
   * Broadcast a message to a room
   * @param room - The room to broadcast the message to
   * @param message - The message to broadcast
   * @returns The response from the TurboWire server
   */
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

  /**
   * Get a signed wire URL for a room
   * @param room - The room to get the wire URL for
   * @returns The signed wire URL
   *
   * This URL can be used to connect to the room via a WebSocket, you can create a signed URL
   * and pass it to the client to connect to the room.
   */
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
