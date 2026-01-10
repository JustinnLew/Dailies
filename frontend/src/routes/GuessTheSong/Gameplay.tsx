import { useState } from "react";
import AudioPlayer from "./AudioPlayer";

type Player = {
  name: string;
  score: number;
};

type ChatMessage = {
  user: string;
  message: string;
};

export default function Gameplay({ 
    sendGuess,
  	previewUrl } 
  : { 
    sendGuess: (guess: string) => void,
    previewUrl: string }) {
  const [songHint] = useState("🎵 Guess the song");
  const [leaderboard] = useState<Player[]>([
    { name: "Justin", score: 12 },
    { name: "Alice", score: 9 },
    { name: "Bob", score: 5 },
  ]);
  const [chat, setChat] = useState<ChatMessage[]>([
    { user: "Alice", message: "Sounds like 80s pop 🤔" },
    { user: "Bob", message: "Is it Queen?" },
    { user: "Justin", message: "Never Gonna Give You Up!" },
  ]);
  const [message, setMessage] = useState("");

  const sendMessage = () => {
    sendGuess(message);
    if (!message.trim()) return;

    setChat((prev) => [
      ...prev,
      { user: "You", message },
    ]);
    setMessage("");
  };

  return (
    <div className="h-screen grid grid-cols-4 gap-4 p-4 bg-gray-900 text-white">
      {/* Song Display */}
      <div className="col-span-3 bg-gray-800 rounded-lg p-6 flex items-center justify-center text-2xl font-semibold text-center">
        {songHint}
      </div>

      {/* Leaderboard */}
      <div className="bg-gray-800 rounded-lg p-4">
        <h2 className="text-lg font-bold mb-3">🏆 Leaderboard</h2>
        <ul className="space-y-2">
          {leaderboard.map((p, i) => (
            <li
              key={i}
              className="flex justify-between bg-gray-700 px-3 py-1 rounded"
            >
              <span>{p.name}</span>
              <span>{p.score}</span>
            </li>
          ))}
        </ul>
      </div>

      {/* Chat */}
      <div className="col-span-4 bg-gray-800 rounded-lg p-4 flex flex-col">
        <h2 className="text-lg font-bold mb-2">💬 Chat</h2>

        <div className="flex-1 overflow-y-auto space-y-1 mb-3">
          {chat.map((c, i) => (
            <div key={i} className="text-sm">
              <span className="font-semibold">{c.user}: </span>
              {c.message}
            </div>
          ))}
        </div>

        <div className="flex gap-2">
          <input
            className="flex-1 rounded bg-gray-700 px-3 py-2 outline-none"
            placeholder="Type a guess..."
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && sendMessage()}
          />
          <button
            onClick={sendMessage}
            className="bg-blue-600 px-4 py-2 rounded hover:bg-blue-500"
          >
            Send
          </button>
        </div>
      </div>
	  <AudioPlayer previewUrl={previewUrl} />
    </div>
  );
}
