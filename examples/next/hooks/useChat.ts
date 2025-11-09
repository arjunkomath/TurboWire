import { TurboWire } from "@turbowire/web";
import { useMemo, useState } from "react";
import { v4 } from "uuid";
import {
  broadcastMessage,
  getSignedWireUrl,
  userJoined,
  userLeft,
} from "@/app/actions";
import { type ChatSchema, chatSchema, type Message } from "@/lib/schema";

export function useChat() {
  const [client, setClient] = useState<TurboWire<ChatSchema> | null>(null);
  const [connected, setConnected] = useState(false);
  const [messages, setMessages] = useState<Message[]>([]);
  const [username, setUsername] = useState("");
  const [isJoined, setIsJoined] = useState(false);

  const userId = useMemo(() => v4(), []);

  const joinChat = async () => {
    if (!username.trim()) return;

    try {
      const wireUrl = await getSignedWireUrl();
      const wire = new TurboWire<ChatSchema>(wireUrl, {
        schema: chatSchema,
        debug: true,
      });

      wire.on("user:joined", (data) => {
        setMessages((prev) => [
          ...prev,
          {
            messageId: v4(),
            text: `${data.username} joined the chat`,
            userId: "system",
            username: "System",
            timestamp: data.timestamp,
          },
        ]);
      });

      wire.on("user:left", (data) => {
        setMessages((prev) => [
          ...prev,
          {
            messageId: v4(),
            text: `${data.username} left the chat`,
            userId: "system",
            username: "System",
            timestamp: data.timestamp,
          },
        ]);
      });

      wire.on("message:sent", (data) => {
        setMessages((prev) => [...prev, data]);
      });

      wire.connect(
        () => {
          setConnected(true);
          setIsJoined(true);

          void userJoined(userId, username);
        },
        (error) => {
          console.error("Connection error:", error);
          setConnected(false);
        },
      );

      setClient(wire);
    } catch (error) {
      console.error("Failed to join chat:", error);
    }
  };

  const sendMessage = async (text: string) => {
    if (!text.trim() || !client || !connected) return;

    const message = {
      messageId: v4(),
      text,
      userId: userId,
      username: username,
      timestamp: Date.now(),
    };

    try {
      void broadcastMessage(message);
    } catch (error) {
      console.error("Failed to send message:", error);
    }
  };

  const leaveChat = () => {
    if (client) {
      void userLeft(userId, username);

      client.disconnect();
      setClient(null);
    }
    setConnected(false);
    setIsJoined(false);
    setMessages([]);
  };

  return {
    connected,
    messages,
    username,
    isJoined,
    userId,
    setUsername,
    joinChat,
    sendMessage,
    leaveChat,
  };
}
