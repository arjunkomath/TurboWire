import type { EventNames, EventPayload } from "@turbowire/serverless";
import z from "zod/v4";

export const chatSchema = {
  userJoined: z.object({
    userId: z.string(),
    username: z.string(),
    timestamp: z.number(),
  }),
  userLeft: z.object({
    userId: z.string(),
    username: z.string(),
    timestamp: z.number(),
  }),
  messageSent: z.object({
    messageId: z.string(),
    text: z.string(),
    userId: z.string(),
    username: z.string(),
    timestamp: z.number(),
  }),
} as const;

export type ChatSchema = typeof chatSchema;
export type ChatEvents = EventNames<typeof chatSchema>;
export type Message = EventPayload<typeof chatSchema, "messageSent">;
