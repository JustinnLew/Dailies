import { useEffect, useState, useRef } from "react";
import { useParams, useNavigate } from "react-router-dom";
import WaitingRoom from "./WaitingRoom";
import Gameplay from "./Gameplay";

export default function Game() {
	const params = useParams();
	const [lobbyCode, setLobbyCode] = useState(params.lobbyCode);
	const [waiting, setWaiting] = useState(true);
	const navigate = useNavigate();
	const socket = useRef<WebSocket>(null!);
	
	useEffect(() => {
		const s = new WebSocket(`ws://localhost:3000/ws/guess-the-song`);
        socket.current = s;

        s.onopen = () => {
			console.log("Connected to lobby ", params.lobbyCode);
        }
        s.onclose = () => {
            console.log("Disconnected from lobby ", lobbyCode);
            // navigate('/');
        }
        s.onmessage = (event) => {
            console.log("Message from server ", event.data);
            const msg = JSON.parse(event.data);
            switch (msg.action) {
				case "lobby__created":
					setLobbyCode(msg.lobbyCode);
					break;
				case "game_start":
					setWaiting(false);
					break;
			}
        }
        return () => {
            s.close();
            // navigate('/guess-the-song');
        }
	}, [params.lobbyCode, lobbyCode, navigate]);

	const sendGuess = (guess: string) => {
		console.log("Sending guess: ", guess);
		socket.current.send(JSON.stringify({ action: "guess", guess }));
	}

	const ready = () => {
		console.log("Player is ready");
		setWaiting(false);
		socket.current.send(JSON.stringify({ action: "ready" }));
	}

	return (
		waiting ? (<WaitingRoom lobbyCode={lobbyCode!} ready={ready} />) : (<Gameplay sendGuess={sendGuess} />)
);
}
