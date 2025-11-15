"use server";

import type { Message } from "@/lib/schema";
import { turboWireHub } from "@/lib/turbo";

const room = "demo-chat-room";

export async function getSignedWireUrl() {
  return turboWireHub.getSignedWire(room);
}

export async function userJoined(userId: string, username: string) {
  await turboWireHub.broadcast(room).userJoined({
    userId,
    username,
    timestamp: Date.now(),
  });
}

export async function userLeft(userId: string, username: string) {
  await turboWireHub.broadcast(room).userLeft({
    userId,
    username,
    timestamp: Date.now(),
  });
}

export async function broadcastMessage(message: Message) {
  await turboWireHub.broadcast(room).messageSent(message);
}
