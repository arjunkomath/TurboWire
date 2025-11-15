"use server";

import type { Message } from "@/lib/schema";
import { turboWireHub } from "@/lib/turbo";

const room = "demo-chat-room";

export async function getSignedWireUrl() {
  const signedUrl = turboWireHub.getSignedWire(room);
  return signedUrl;
}

export async function userJoined(userId: string, username: string) {
  await turboWireHub.broadcast(room, "user:joined", {
    userId,
    username,
    timestamp: Date.now(),
  });
}

export async function userLeft(userId: string, username: string) {
  await turboWireHub.broadcast(room, "user:left", {
    userId,
    username,
    timestamp: Date.now(),
  });
}

export async function broadcastMessage(message: Message) {
  await turboWireHub.broadcast(room, "message:sent", message);
}
