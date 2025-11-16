import { broadcastMessage, userJoined, userLeft } from "@/app/actions";
import { chatSchema, type Message } from "@/lib/schema";
import { useWireEvent } from "@turbowire/react";
import { useEffect, useState } from "react";
import { v4 } from "uuid";

export function useChat(userId: string, wireUrl: string) {
  const [messages, setMessages] = useState<Message[]>([]);

  useEffect(() => {
    void userJoined(userId);

    return () => {
      void userLeft(userId);
    };
  }, [userId]);

  useWireEvent(wireUrl, {
    schema: chatSchema,
    userJoined: (data) => {
      setMessages((prev) => [
        ...prev,
        {
          messageId: v4(),
          text: `${data.userId} joined the chat`,
          userId: "system",
          timestamp: data.timestamp,
        },
      ]);
    },
    userLeft: (data) => {
      setMessages((prev) => [
        ...prev,
        {
          messageId: v4(),
          text: `${data.userId} left the chat`,
          userId: "system",
          timestamp: data.timestamp,
        },
      ]);
    },
    messageSent: (data) => {
      setMessages((prev) => [
        ...prev,
        {
          messageId: v4(),
          text: data.text,
          userId: data.userId,
          timestamp: data.timestamp,
        },
      ]);
    },
  });

  const sendMessage = async (text: string) => {
    if (!text.trim()) return;

    const message = {
      messageId: v4(),
      text,
      userId,
      timestamp: Date.now(),
    };

    try {
      void broadcastMessage(message);
    } catch (error) {
      alert("Failed to send message");
      console.error("Failed to send message:", error);
    }
  };

  return {
    messages,
    userId,
    sendMessage,
  };
}
