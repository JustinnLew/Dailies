import type { Player, GuessTheSongGameSettings } from "../../utils/types";

export default function Waiting({ 
    lobbyCode,
    ready,
    players,
    gameSettings,
    updateGameSettings, 
}: { 
    lobbyCode: string, 
    ready: () => void, 
    players: Map<string, Player>, 
    gameSettings: GuessTheSongGameSettings, 
    updateGameSettings: (settings: GuessTheSongGameSettings) => void 
}) {

    const isValidSpotifyLink = (link: string) => /https:\/\/open\.spotify\.com\/playlist\/[a-zA-Z0-9]+/.test(link);

    return (
        <div className="h-screen flex p-4 bg-gray-100">
            {/* Left panel: Game ID + Player List */}
            <div className="flex flex-col w-1/3 bg-white p-4 rounded shadow-md">
                <div className="mb-4">
                    <h2 className="text-lg font-bold">Game Code</h2>
                    <p className="text-gray-700 font-mono tracking-widest">{lobbyCode}</p>
                </div>

                <div>
                    <h2 className="text-lg font-bold mb-2">Players</h2>
                    <ul className="space-y-2">
                        {[...players.entries()].map(([id, p]) => (
                            <li key={id} className={`p-2 rounded transition-colors ${p.ready ? "bg-green-200 text-green-900" : "bg-gray-200 text-gray-900"}`}>
                                {p.username} {p.ready && "✓"}
                            </li>
                        ))}
                    </ul>
                </div>
            </div>

            {/* Right panel: Settings */}
            <div className="flex-1 ml-4 flex flex-col justify-between bg-white p-4 rounded shadow-md">
                <div>
                    <h2 className="text-lg font-bold mb-4">Settings</h2>
                    
                    <div className="grid grid-cols-2 gap-4">
                        {/* Number of Songs */}
                        <div className="mb-2">
                            <label className="block mb-1 font-medium text-sm">Number of Songs:</label>
                            <select 
                                className="w-full border rounded p-1 bg-white cursor-pointer"
                                value={gameSettings.numSongs}
                                onChange={(e) => updateGameSettings({
                                    ...gameSettings, 
                                    numSongs: parseInt(e.target.value)
                                })}
                            >
                                {[5, 10, 20].map(val => <option key={val} value={val}>{val}</option>)}
                            </select>
                        </div>

                        {/* Round Length */}
                        <div className="mb-2">
                            <label className="block mb-1 font-medium text-sm">Round Length (s):</label>
                            <select 
                                className="w-full border rounded p-1 bg-white cursor-pointer"
                                value={gameSettings.roundLengthSeconds}
                                onChange={(e) => updateGameSettings({
                                    ...gameSettings, 
                                    roundLengthSeconds: parseInt(e.target.value)
                                })}
                            >
                                {[5, 10, 20, 30].map(val => <option key={val} value={val}>{val}s</option>)}
                            </select>
                        </div>

                        {/* Answer Delay */}
                        <div className="mb-2">
                            <label className="block mb-1 font-medium text-sm">Answer Delay (s):</label>
                            <select 
                                className="w-full border rounded p-1 bg-white cursor-pointer"
                                value={gameSettings.answerDelaySeconds}
                                onChange={(e) => updateGameSettings({
                                    ...gameSettings, 
                                    answerDelaySeconds: parseInt(e.target.value)
                                })}
                            >
                                {[0, 1, 2, 3].map(val => <option key={val} value={val}>{val}s</option>)}
                            </select>
                        </div>

                        {/* Round Delay */}
                        <div className="mb-2">
                            <label className="block mb-1 font-medium text-sm">Round Delay (s):</label>
                            <select 
                                className="w-full border rounded p-1 bg-white cursor-pointer"
                                value={gameSettings.roundDelaySeconds}
                                onChange={(e) => updateGameSettings({
                                    ...gameSettings, 
                                    roundDelaySeconds: parseInt(e.target.value)
                                })}
                            >
                                {[0, 1, 2, 3].map(val => <option key={val} value={val}>{val}s</option>)}
                            </select>
                        </div>
                    </div>

                    <div className="mt-4">
                        <label className="block mb-1 font-medium text-sm">Spotify Playlist Link:</label>
                        <input
                            type="text"
                            placeholder="https://open.spotify.com/playlist/..."
                            className="w-full border rounded p-2"
                            value={gameSettings.playlistLink}
                            onChange={(e) => updateGameSettings({...gameSettings, playlistLink: e.target.value})}
                        />
                    </div>
                </div>

                {/* Start Game button at bottom-right */}
                <div className="flex justify-end">
                    <button
                        disabled={!isValidSpotifyLink(gameSettings.playlistLink)}
                        className={`px-6 py-2 rounded text-white font-bold transition-all
                            ${isValidSpotifyLink(gameSettings.playlistLink)
                            ? "bg-emerald-500 hover:bg-emerald-600 shadow-lg active:scale-95"
                            : "bg-gray-400 cursor-not-allowed"
                            }
                        `}
                        onClick={ready}>
                        Ready
                    </button>
                </div>
            </div>
        </div>
    )
}