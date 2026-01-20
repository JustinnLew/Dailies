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
    <div className="scanlines h-screen flex gap-4 p-6 bg-black text-white font-press-start">
      <div className="flex-2 flex flex-col h-full">
        {/* Chat Section */}
        <div className="border-4 border-neon-blue p-4 flex flex-col tracking-wider flex-1 min-h-0">
          <h2 className="text-lg font-bold mb-2">💬</h2>

          {/* Message List: This area will now scroll */}
          <div className="flex-1 overflow-y-auto space-y-1 mb-3 pr-2 custom-scrollbar min-h-0">
            {chat.map((c, i) => {
              const isSystem = c.user === "";

              return (
                <div
                  key={i}
                  className={`text-xl font-vt323 wrap-break-words p-1 rounded-md transition-all ${
                    isSystem ? "bg-neon-green/50 my-2" : ""
                  }`}
                >
                  <span
                    className={`font-bold ${isSystem ? "" : "text-blue-400"}`}
                  >
                    {isSystem ? "🎵" : ""}
                    {c.user}
                  </span>
                  <span className={"ml-1"}>
                    {isSystem ? "" : ": "}
                    {c.message}
                  </span>
                </div>
              );
            })}
            {/* Auto-scroll anchor */}
            <div ref={chatEndRef} />
          </div>
        </div>

        {/* Input Area: Stays at the bottom */}
        <div className="flex gap-2 pt-2 border-t border-gray-700 shrink-0">
          <input
            className="flex-1 text-xl tracking-widest font-vt323 bg-gray-700 px-3 py-2 text-white focus:outline-none"
            placeholder="Type a guess..."
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && sendMessage()}
          />
          <button
            onClick={sendMessage}
            className="bg-neon-blue px-4 py-2 cursor-pointer"
          >
            Send
          </button>
        </div>
      </div>

      <div className="flex flex-col flex-1 gap-6">
        {/* Leaderboard */}
        <div className="flex-2 p-4 flex flex-col gap-4 border-4 border-neon-pink">
          <h2 className="text-3xl font-bold mb-3 text-shadow-(--text-shadow-title)">
            Leaderboard
          </h2>
          <ul className="space-y-2 overflow-y-auto custom-scrollbar">
            {Array.from(players.entries())
              .map(([id, player]) => ({
                id,
                username: player.username,
                score: leaderboard.get(id) || 0,
              }))
              .sort((a, b) => b.score - a.score)
              .map((player, i) => (
                <li
                  key={player.id}
                  className="text-lg flex justify-between px-3 py-1 border-b-2 border-gray-700"
                >
                  <div className="flex gap-6">
                    <span className="text-neon-yellow w-6">#{i + 1}</span>
                    <span className="truncate max-w-[100px]">
                      {player.username}
                    </span>
                  </div>
                  <span className="text-emerald-400 font-bold">
                    {player.score}
                  </span>
                </li>
              ))}
          </ul>
        </div>

        {/* Song Display */}
        <div className="flex-1 border-neon-pink border-4 p-6 flex items-center justify-center text-2xl font-semibold text-center">
          {"🎵 Guess the song"}
        </div>
      </div>

      <AudioPlayer songState={songState} />
    </div>
  );
}
