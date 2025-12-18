import { useEffect, useState, useRef } from "react";
import { useParams, useNavigate } from "react-router-dom";
import WaitingRoom from "./WaitingRoom";
import Gameplay from "./Gameplay";
import { getUserId, getUserName } from "../../utils/util";
import type { Player } from "../../utils/types";

export default function Game() {
	const params = useParams();
	const [waiting, setWaiting] = useState(true);
	const userId = getUserId();
	const username = getUserName();
	const [players, setPlayers] = useState<Map<string, Player>>(new Map());
	
	const navigate = useNavigate();
	const socket = useRef<WebSocket>(null!);
	
	useEffect(() => {
		const s = new WebSocket(`ws://localhost:3000/ws/guess-the-song`);
        socket.current = s;

        s.onopen = () => {
			console.log("Connected to server ", params.lobbyCode);
			socket.current.send(JSON.stringify({
				lobby_code: params.lobbyCode,
				event: "join",
				user_id: userId,
				username: username,
			}))
        }
        s.onclose = () => {
            console.log("Disconnected from lobby ", params.lobbyCode);
            // navigate('/');
        }
        s.onmessage = (event) => {
            console.log("Message from server ", event.data);
            const msg = JSON.parse(event.data);
            switch (msg.action) {
				case "SyncState":
					setPlayers(new Map(msg.data.players.map((p: [string, string, boolean]) => [p[0], { username: p[1], ready: p[2] }])));
					break;
				case "PlayerJoin":
					setPlayers(p => new Map([...p, [msg.data.player_id, { username: msg.data.player_username, ready: false }]]));
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
				case "AllReady":
					setWaiting(false);
					break;
			}
        }
        return () => {
            s.close();
            // navigate('/guess-the-song');
        }
	}, [params.lobbyCode, navigate, userId, username]);

	const sendGuess = (guess: string) => {
		socket.current.send(JSON.stringify({ lobby_code: params.lobbyCode, event: "guess", content: guess }));
	}

	const ready = () => {
		socket.current.send(JSON.stringify({ lobby_code: params.lobbyCode, event: "ready", user_id: userId, username }));
	}

	return (
		waiting ? (<WaitingRoom lobbyCode={params.lobbyCode!} ready={ready} players={players} />) : (<Gameplay sendGuess={sendGuess} />)
);
}
