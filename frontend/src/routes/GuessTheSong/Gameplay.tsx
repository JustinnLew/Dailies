import { useEffect, useRef, useState } from "react";
import AudioPlayer from "./AudioPlayer";
import type { Player } from "../../utils/types";
import type { ChatMessage } from "../../utils/types";

export default function Gameplay({ 
    sendGuess,
  	previewUrl,
	players,
	chat,} 
  : { 
    sendGuess: (guess: string) => void,
    previewUrl: string,
	players: Map<string, Player>,
	chat: ChatMessage[],}) {
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
		if (message === ""|| previewUrl === "") return;
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
				<div className="bg-gray-800 rounded-lg p-4 w-1/3">
					<h2 className="text-lg font-bold mb-3">🏆 Leaderboard</h2>
					<ul className="space-y-2">
					{Array.from(players.values())
						.sort((a, b) => (b.score || 0) - (a.score || 0))
						.map((p, i) => (
							<li
								key={p.username}
								className="flex justify-between bg-gray-700 px-3 py-1 rounded"
							>
								<div className="flex gap-2">
									<span className="text-gray-400">#{i + 1}</span>
									<span className="font-medium">{p.username}</span>
								</div>
								<span className="text-emerald-400 font-mono">{p.score || 0}</span>
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
					{chat.map((c, i) => (
						<div key={i} className="text-sm break-words">
						<span className="font-semibold text-blue-400">{c.user}: </span>
						<span className="text-gray-200">{c.message}</span>
						</div>
					))}
					{/* Auto-scroll anchor (optional but recommended) */}
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
			<AudioPlayer previewUrl={previewUrl} />
		</div>
  	);
}
