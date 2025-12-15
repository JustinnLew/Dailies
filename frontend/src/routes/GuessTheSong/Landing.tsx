import { useState } from "react";
import Navbar from "../../components/NavBar"
import { useNavigate } from "react-router-dom";
import { getUserId, getUserName } from "../../utils/util";

export default function Landing() {
    const navigate = useNavigate();
    const [lobbyInput, setLobbyInput] = useState("");
    const userId = getUserId();
    const userName = getUserName();

    // This will call your server to create a new lobby
    const createLobby = async () => {
        const res = await fetch("http://localhost:3000/guess-the-song/create-lobby", {
            method: "POST",
            body: JSON.stringify({ userId, userName })
        });
        const data = await res.json();
        console.log(`User ${userName} created lobby ${data.lobby_code}`);
        navigate(`/guess-the-song/${data.lobby_code}`);
    }

    const joinLobby = async () => {
        if (lobbyInput.trim() === "") return;
        // Maybe send a http request here to join as well
        const res = await fetch(`http://localhost:3000/guess-the-song/join-lobby/${lobbyInput}`, {
            method: "POST",
            body: JSON.stringify({ userId })
        });
        if (!res.ok) {
            alert("Failed to join lobby. Please check the code and try again.");
            return;
        }

        // navigate to the lobby page
        navigate(`/guess-the-song/${lobbyInput}`);
        setLobbyInput("");
    };

    return (
        <div className="flex flex-col min-h-screen">
            <Navbar />

            <div className="flex flex-col items-center justify-center flex-1 bg-gray-100">
                <h1 className="text-4xl font-bold mb-6">Guess The Song</h1>

                <p className="text-lg text-center max-w-xl px-4 mb-10">
                    Welcome to Guess The Song! Create a lobby, invite your friends,
                    and see who can guess the song the fastest.
                </p>

                {/* Create + Join */}
                <div className="flex gap-4 items-center">
                    <button
                        onClick={createLobby}
                        className="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white text-lg rounded-lg shadow-md"
                    >
                        Create Multiplayer Lobby
                    </button>

                    <div className="flex flex-col gap-2 items-center">
                        <button
                            onClick={joinLobby}
                            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg shadow-md"
                        >
                            Join Lobby
                        </button>
                        <input
                            type="text"
                            value={lobbyInput}
                            onChange={(e) => setLobbyInput(e.target.value)}
                            placeholder="Enter lobby code"
                            className="border border-gray-300 rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        />
                    </div>
                </div>
            </div>
        </div>
    );

}
