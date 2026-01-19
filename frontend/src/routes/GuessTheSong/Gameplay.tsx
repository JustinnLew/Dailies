import { useEffect, useRef, useState } from "react";
import AudioPlayer from "./AudioPlayer";
import type { Player } from "../../utils/types";
import type { ChatMessage } from "../../utils/types";

export default function Gameplay({
  sendGuess,
  songState,
  players,
  chat,
  leaderboard,
}: {
  sendGuess: (guess: string) => void;
  songState: { previewUrl: string; roundStartTime: number };
  players: Map<string, Player>;
  chat: ChatMessage[];
  leaderboard: Map<string, number>;
}) {
  const [songHint] = useState("🎵 Guess the song");
  const [message, setMessage] = useState("");

  const chatEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  // AutoScroll
  useEffect(() => {
    scrollToBottom();
  }, [chat]);

  const sendMessage = () => {
    if (message === "" || songState.previewUrl === "") return;
    sendGuess(message);
    setMessage("");
  };

  return (
    <div className="h-screen flex flex-col gap-4 p-4 bg-gray-900 text-white">
      <div className="h-1/2 flex gap-4">
        {/* Song Display */}
        <div className="bg-gray-800 rounded-lg p-6 flex items-center justify-center text-2xl font-semibold text-center w-2/3">
          {songHint}
        </div>

        {/* Leaderboard */}
        <div className="bg-gray-800 rounded-lg p-4 w-1/3 flex flex-col">
          <h2 className="text-lg font-bold mb-3">🏆 Leaderboard</h2>
          <ul className="space-y-2 overflow-y-auto">
            {Array.from(players.entries())
              .map(([id, player]) => ({
                id,
                username: player.username,
                // Look up score from leaderboard Map, default to 0
                score: leaderboard.get(id) || 0,
              }))
              .sort((a, b) => b.score - a.score)
              .map((player, i) => (
                <li
                  key={player.id}
                  className="flex justify-between bg-gray-700 px-3 py-1 rounded transition-all duration-300"
                >
                  <div className="flex gap-2">
                    <span className="text-gray-400 w-6">#{i + 1}</span>
                    <span className="font-medium truncate max-w-[100px]">
                      {player.username}
                    </span>
                  </div>
                  <span className="text-emerald-400 font-mono font-bold">
                    {player.score}
                  </span>
                </li>
              ))}
          </ul>
        </div>
      </div>

      {/* Chat Section */}
      <div className="bg-gray-800 rounded-lg p-4 flex flex-col h-1/2">
        <h2 className="text-lg font-bold mb-2">💬</h2>

        {/* Message List: This area will now scroll */}
        <div className="flex-1 overflow-y-auto space-y-1 mb-3 pr-2 custom-scrollbar">
          {chat.map((c, i) => {
            const isSystem = c.user === "";

            return (
              <div
                key={i}
                className={`text-sm break-words p-1 rounded-md transition-all ${
                  isSystem
                    ? "bg-emerald-500/10 border-l-4 border-emerald-500 my-1 shadow-sm"
                    : "hover:bg-gray-700/30"
                }`}
              >
                <span
                  className={`font-bold ${isSystem ? "text-emerald-400" : "text-blue-400"}`}
                >
                  {isSystem ? "🎵" : ""}
                  {c.user}
                </span>
                <span
                  className={`ml-1 ${isSystem ? "text-emerald-50 font-medium" : "text-gray-200"}`}
                >
                  {isSystem ? "" : ": "}
                  {c.message}
                </span>
              </div>
            );
          })}
          {/* Auto-scroll anchor */}
          <div ref={chatEndRef} />
        </div>

        {/* Input Area: Stays at the bottom */}
        <div className="flex gap-2 pt-2 border-t border-gray-700">
          <input
            className="flex-1 rounded bg-gray-700 px-3 py-2 text-white outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="Type a guess..."
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && sendMessage()}
          />
          <button
            onClick={sendMessage}
            className="bg-blue-600 px-4 py-2 rounded hover:bg-blue-500 transition-colors"
          >
            Send
          </button>
        </div>
      </div>
      <AudioPlayer songState={songState} />
    </div>
  );
}
