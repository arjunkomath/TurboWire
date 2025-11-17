import type { EventNames, EventPayload } from "@turbowire/serverless";
import z from "zod/v4";

export const ROOM = "demo-chat-room";

export const chatSchema = {
  userJoined: z.object({
    userId: z.string(),
    timestamp: z.number(),
  }),
  userLeft: z.object({
    userId: z.string(),
    timestamp: z.number(),
  }),
  messageSent: z.object({
    messageId: z.string(),
    text: z.string(),
    userId: z.string(),
    timestamp: z.number(),
  }),
} as const;

export type ChatSchema = typeof chatSchema;
export type ChatEvents = EventNames<typeof chatSchema>;
export type Message = EventPayload<typeof chatSchema, "messageSent">;
export type UserJoinedPayload = EventPayload<typeof chatSchema, "userJoined">;
export type UserLeftPayload = EventPayload<typeof chatSchema, "userLeft">;
