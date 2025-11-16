"use client";

import { useState } from "react";
import { useChat } from "@/hooks/useChat";

export default function ChatRoom({
  userId,
  wireUrl,
}: {
  userId: string;
  wireUrl: string;
}) {
  const [inputValue, setInputValue] = useState("");

  const { messages, sendMessage } = useChat(userId, wireUrl);

  return (
    <div className="max-w-6xl mx-auto h-[calc(100vh-2rem)] flex gap-4">
      <div className="flex-1 bg-white rounded-lg shadow-lg flex flex-col">
        <div className="p-4 border-b border-gray-200 flex justify-between items-center">
          <div>
            <h1 className="text-2xl font-bold text-gray-800">TurboWire Chat</h1>
            <p className="text-sm text-gray-500">
              Connected as <span className="font-medium">{userId}</span>
            </p>
          </div>
        </div>

        <div className="flex-1 overflow-y-auto p-4 space-y-3">
          {messages.map((msg) => (
            <div
              key={msg.messageId}
              className={`flex ${msg.userId === userId ? "justify-end" : "justify-start"}`}
            >
              <div
                className={`max-w-xs px-4 py-2 rounded-lg ${
                  msg.userId === "system"
                    ? "bg-gray-200 text-gray-600 text-sm italic text-center"
                    : msg.userId === userId
                      ? "bg-blue-600 text-white"
                      : "bg-gray-200 text-gray-800"
                }`}
              >
                {msg.userId !== "system" && msg.userId !== userId && (
                  <p className="text-xs font-semibold mb-1">{msg.userId}</p>
                )}
                <p>{msg.text}</p>
                <p className="text-xs opacity-75 mt-1">
                  {new Date(msg.timestamp).toLocaleTimeString()}
                </p>
              </div>
            </div>
          ))}
        </div>

        <div className="p-4 border-t border-gray-200">
          <div className="flex gap-2">
            <input
              type="text"
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && inputValue.trim()) {
                  sendMessage(inputValue);
                  setInputValue("");
                }
              }}
              placeholder="Type a message..."
              className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <button
              type="button"
              onClick={() => {
                if (inputValue.trim()) {
                  sendMessage(inputValue);
                  setInputValue("");
                }
              }}
              disabled={!inputValue.trim()}
              className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
            >
              Send
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
