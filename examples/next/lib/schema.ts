import type { EventNames, EventPayload } from "@turbowire/serverless";
import z from "zod/v4";

export const chatSchema = {
  "user:joined": z.object({
    userId: z.string(),
    username: z.string(),
    timestamp: z.number(),
  }),
  "user:left": z.object({
    userId: z.string(),
    username: z.string(),
    timestamp: z.number(),
  }),
  "message:sent": z.object({
    messageId: z.string(),
    text: z.string(),
    userId: z.string(),
    username: z.string(),
    timestamp: z.number(),
  }),
} as const;

export type ChatSchema = typeof chatSchema;
export type ChatEvents = EventNames<typeof chatSchema>;
export type Message = EventPayload<typeof chatSchema, "message:sent">;
