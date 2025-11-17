import { useWireEvent } from "@turbowire/react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { v4 } from "uuid";
import { broadcastMessage, userJoined, userLeft } from "@/app/actions";
import { chatSchema, type Message, type UserJoinedPayload } from "@/lib/schema";

export function useChat(userId: string, wireUrl: string) {
  const [messages, setMessages] = useState<Message[]>([]);

  useEffect(() => {
    userJoined(userId).catch((error) => {
      alert("Failed to join chat");
      console.error("Failed to join chat:", error);
    });

    return () => {
      userLeft(userId).catch((error) => {
        alert("Failed to leave chat");
        console.error("Failed to leave chat:", error);
      });
    };
  }, [userId]);

  const handleUserJoined = useCallback((data: UserJoinedPayload) => {
    setMessages((prev) => [
      ...prev,
      {
        messageId: v4(),
        text: `${data.userId} joined the chat`,
        userId: "system",
        timestamp: data.timestamp,
      },
    ]);
  }, []);

  const handleUserLeft = useCallback((data: UserJoinedPayload) => {
    setMessages((prev) => [
      ...prev,
      {
        messageId: v4(),
        text: `${data.userId} left the chat`,
        userId: "system",
        timestamp: data.timestamp,
      },
    ]);
  }, []);

  const handleMessageSent = useCallback((data: Message) => {
    setMessages((prev) => [
      ...prev,
      {
        messageId: v4(),
        text: data.text,
        userId: data.userId,
        timestamp: data.timestamp,
      },
    ]);
  }, []);

  const sendMessage = useCallback(
    async (text: string) => {
      if (!text.trim()) return;

      const message = {
        messageId: v4(),
        text,
        userId,
        timestamp: Date.now(),
      };

      try {
        await broadcastMessage(message);
      } catch (error) {
        alert("Failed to send message");
        console.error("Failed to send message:", error);
      }
    },
    [userId],
  );

  const wireOptions = useMemo(
    () => ({
      schema: chatSchema,
      userJoined: handleUserJoined,
      userLeft: handleUserLeft,
      messageSent: handleMessageSent,
    }),
    [handleUserJoined, handleUserLeft, handleMessageSent],
  );

  useWireEvent(wireUrl, wireOptions);

  return {
    messages,
    userId,
    sendMessage,
  };
}
