import { useEffect, useState, useRef, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import WaitingRoom from "./WaitingRoom";
import Gameplay from "./Gameplay";
import { getUserId, getUserName } from "../../utils/util";
import type { Player, GuessTheSongGameSettings, ChatMessage } from "../../utils/types";

export default function Game() {
	const params = useParams();
	const userId = getUserId();
	const username = getUserName();
	const [players, setPlayers] = useState<Map<string, Player>>(new Map());
	const [gameSettings, setGameSettings] = useState<GuessTheSongGameSettings>({
		playlistLink: "",
		numSongs: 10,
		roundLengthSeconds: 30,
		answerDelaySeconds: 0,
		roundDelaySeconds: 3,
	})
	const navigate = useNavigate();
	const socket = useRef<WebSocket>(null!);
	const updateGameSettings = useCallback((settings: GuessTheSongGameSettings) => {
		setGameSettings(settings);
		console.log("Updating game settings: ", settings);

		if (socket.current && socket.current.readyState === WebSocket.OPEN) {
			console.log("Sending updated settings to server", settings);
			socket.current.send(JSON.stringify({
				event: "UpdateGameSettings",
				settings: settings,
			}));
		}
	}, []);
	const [previewUrl, setPreviewUrl] = useState<string>("");
	const [gameState, setGameState] = useState<"waiting" | "loading" | "playing" | "finished">("waiting");
	const [chat, setChat] = useState<ChatMessage[]>([]);
	
	useEffect(() => {
		const s = new WebSocket(`ws://localhost:3000/ws/guess-the-song`);
        socket.current = s;

        s.onopen = () => {
			console.log("Connected to server ", params.lobbyCode);
			socket.current.send(JSON.stringify({
				event: "Join",
				lobby_code: params.lobbyCode,
				user_id: userId,
				username: username,
			}))
        }
        s.onclose = () => {
            console.log("Disconnected from lobby ", params.lobbyCode);
            // navigate('/');
        }
        s.onmessage = (event) => {
            const msg = JSON.parse(event.data);
            switch (msg.event) {
				case "SyncState":
					setPlayers(new Map(msg.data.players.map((p: [string, string, boolean]) => [p[0], { username: p[1], ready: p[2] }])));
					setGameSettings({
						numSongs: msg.data.num_songs,
						playlistLink: msg.data.playlist_link,
						roundLengthSeconds: msg.data.round_length_seconds,
						answerDelaySeconds: msg.data.answer_delay_seconds,
						roundDelaySeconds: msg.data.round_delay_seconds,
					})
					break;
				case "PlayerJoin":
						setPlayers(p => new Map([...p, [msg.data.player_id, { username: msg.data.player_username, ready: false, score: 0 }]]));
					break;
				case "PlayerReady":
					setPlayers(p => new Map([...p].map(pl => pl[0] === msg.data.player_id ? [pl[0], { ...pl[1], ready: true }] : pl)));
					break;
				case "PlayerLeave":
						setPlayers(p => {
							const newPlayers = new Map(p);
							newPlayers.delete(msg.data.player_id);
							return newPlayers;
						});
						break;
				case "GameSettingsUpdated":
					console.log("Received updated game settings from server: ", msg.data.settings);
					setGameSettings(msg.data.settings);
					break;
				case "AllReady":
					setGameState("loading");
					break;
				case "GameStart":
					console.log("Game starting!");
					setGameState("playing");
				break;
				case "RoundStart":
					console.log("Round started with preview URL: ", msg.data.preview_url);
					setPreviewUrl(msg.data.preview_url);
					break;
				case "RoundEnd":
					console.log("Round ended. Correct title: ", msg.data.correct_title, " Correct artists: ", msg.data.correct_artists);
					setPreviewUrl("");
					break;
				case "GameEnd":
					console.log("Game ended");
					break;
				case "PlayerGuess":
					setChat(c => [...c, { user: msg.data.username, message: msg.data.content }]);
					break;
				default:
					console.log("Unknown event received: ", msg);
					break;
			}
        }
        return () => {
            s.close();
            // navigate('/guess-the-song');
        }
	}, [params.lobbyCode, navigate, userId, username]);

	const sendGuess = (guess: string) => {
		socket.current.send(JSON.stringify({ event: "Guess", content: guess }));
	}

	const ready = () => {
		socket.current.send(JSON.stringify({ event: "Ready"}));
	}

	if (gameState === "waiting") {
		return (<WaitingRoom 
			lobbyCode={params.lobbyCode!}
			ready={ready}
			players={players}
			gameSettings={gameSettings}
			updateGameSettings={updateGameSettings} />
		);
	}

	if (gameState === "loading") {
		return (
			<div className="h-screen flex flex-col items-center justify-center bg-gray-900 text-white">
				<div className="animate-spin rounded-full h-16 w-16 border-t-4 border-emerald-500 mb-4"></div>
				<h2 className="text-2xl font-bold">Preparing Playlist...</h2>
				<p className="text-gray-400">The game will start in a moment.</p>
			</div>
    );
}

	return (
		<Gameplay 
			sendGuess={sendGuess}
			previewUrl={previewUrl}
			players={players}
			chat={chat} 
		/>
	);
}