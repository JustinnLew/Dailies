import Navbar from "../../components/NavBar"
import { useNavigate } from "react-router-dom"

export default function Landing() {
    const navigate = useNavigate();

    // This will call your server to create a new lobby
    const createLobby = async () => {
        // 1. Call backend to create a lobby
        const res = await fetch("http://localhost:3000/guess-the-song/create-lobby", {
            method: "GET",
        });
        const data = await res.json();

        // backend sends: { lobbyId: "abc123" }
        const lobbyId = data.lobby_code;
        console.log("Lobby created with ID:" , lobbyId);


        // 2. Navigate to the lobby page, where websocket will start
        navigate(`/guess-the-song/lobby/${lobbyId}`);
    }

    return (
        <div className="flex flex-col min-h-screen">
            <Navbar />

            <div className="flex flex-col items-center justify-center flex-1 bg-gray-100">
                <h1 className="text-4xl font-bold mb-6">Guess The Song</h1>

                <p className="text-lg text-center max-w-xl px-4 mb-10">
                    Welcome to Guess The Song! Create a lobby, invite your friends,
                    and see who can guess the song the fastest.
                </p>

                <button
                    onClick={createLobby}
                    className="px-6 py-3 bg-blue-600 hover:bg-blue-700 text-white text-lg rounded-lg shadow-md"
                >
                    Create Multiplayer Lobby
                </button>
            </div>
        </div>
    );
}
