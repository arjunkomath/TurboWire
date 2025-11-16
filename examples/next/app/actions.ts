"use server";

import { ROOM, type Message } from "@/lib/schema";
import { turboWireHub } from "@/lib/turbo";

export async function userJoined(userId: string) {
  await turboWireHub.broadcast(ROOM).userJoined({
    userId,
    timestamp: Date.now(),
  });
}

export async function userLeft(userId: string) {
  await turboWireHub.broadcast(ROOM).userLeft({
    userId,
    timestamp: Date.now(),
  });
}

export async function broadcastMessage(message: Message) {
  await turboWireHub.broadcast(ROOM).messageSent(message);
}
