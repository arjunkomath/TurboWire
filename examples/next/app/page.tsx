"use client";

import { useState } from "react";
import { useChat } from "@/hooks/useChat";

export default function Home() {
  const [inputValue, setInputValue] = useState("");

  const {
    connected,
    messages,
    username,
    isJoined,
    userId,
    setUsername,
    joinChat,
    sendMessage,
    leaveChat,
  } = useChat();

  if (!isJoined) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-linear-to-br from-blue-50 to-indigo-100">
        <div className="bg-white p-8 rounded-lg shadow-lg w-96">
          <h1 className="text-3xl font-bold mb-6 text-gray-800">
            TurboWire Chat Demo
          </h1>
          <div className="space-y-4">
            <div>
              <label
                htmlFor="username"
                className="block text-sm font-medium text-gray-700 mb-2"
              >
                Enter your username
              </label>
              <input
                id="username"
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && joinChat()}
                placeholder="Your name..."
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              />
            </div>
            <button
              type="button"
              onClick={joinChat}
              disabled={!username.trim()}
              className="w-full bg-blue-600 text-white py-2 px-4 rounded-lg hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
            >
              Join Chat
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-linear-to-br from-blue-50 to-indigo-100 p-4">
      <div className="max-w-6xl mx-auto h-[calc(100vh-2rem)] flex gap-4">
        <div className="flex-1 bg-white rounded-lg shadow-lg flex flex-col">
          <div className="p-4 border-b border-gray-200 flex justify-between items-center">
            <div>
              <h1 className="text-2xl font-bold text-gray-800">
                TurboWire Chat
              </h1>
              <p className="text-sm text-gray-500">
                Connected as <span className="font-medium">{username}</span>
              </p>
            </div>
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2">
                <div
                  className={`w-2 h-2 rounded-full ${connected ? "bg-green-500" : "bg-red-500"}`}
                />
                <span className="text-sm text-gray-600">
                  {connected ? "Connected" : "Disconnected"}
                </span>
              </div>
              <button
                type="button"
                onClick={leaveChat}
                className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors text-sm"
              >
                Leave
              </button>
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
                    <p className="text-xs font-semibold mb-1">{msg.username}</p>
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
    </div>
  );
}
