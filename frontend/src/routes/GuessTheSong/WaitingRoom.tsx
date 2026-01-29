import { useNavigate } from "react-router-dom";
import CopyIcon from "../../icons/CopyIcon";
import type { Player, GuessTheSongGameSettings } from "../../utils/types";
import { type Dispatch, type SetStateAction } from "react";
import ErrorSnackbar from "../../components/ErrorSnackbar";

export default function Waiting({
  lobbyCode,
  onReady,
  players,
  gameSettings,
  updateGameSettings,
  error,
  setError,
  ready,
}: {
  lobbyCode: string;
  onReady: () => void;
  players: Map<string, Player>;
  gameSettings: GuessTheSongGameSettings;
  updateGameSettings: (settings: GuessTheSongGameSettings) => void;
  error: string;
  setError: Dispatch<SetStateAction<string>>;
  ready: boolean;
}) {
  const isValidSpotifyLink = (link: string) =>
    /^https:\/\/open\.spotify\.com\/playlist\/[a-zA-Z0-9]+$/.test(link);
  const navigate = useNavigate();

  return (
    <div className="flex-col h-fit sm:h-screen flex sm:flex-row p-4 gap-4 bg-black font-press-start scanlines">
      {/* Left panel: Game ID + Player List */}
      <div className="flex w-full flex-col gap-6 sm:w-1/3 p-4 rounded shadow-md border-4 border-neon-pink">
        <h2 className="text-2xl font-bold text-white text-shadow-(--text-shadow-title)">
          Game Code
        </h2>
        <div className="flex">
          <p className="flex-1 text-neon-yellow font-vt323 text-2xl tracking-widest">
            {lobbyCode}
          </p>
          <button
            onClick={() => {
              navigator.clipboard.writeText(lobbyCode);
            }}
            className="p-2 transition-all hover:scale-110 active:scale-95 bg-neon-bg"
            title="Copy Code"
          >
            {/* ICON PLACEHOLDER */}
            <div className="w-5 h-5 flex items-center justify-center cursor-pointer">
              <CopyIcon size={28} color={"white"} />
            </div>
          </button>
        </div>

        <div>
          <h2 className="font-bold mb-2 text-white text-shadow-(--text-shadow-icon)">
            Players
          </h2>
          <ul className="space-y-2">
            {[...players.entries()].sort().map(([id, p]) => (
              <li
                key={id}
                className={`flex text-sm p-2 rounded transition-colors border-2 border-neon-yellow text-white ${p.ready ? "bg-neon-green" : "bg-black"}`}
                style={{
                  clipPath:
                    "polygon(0 4px, 4px 4px, 4px 0, calc(100% - 4px) 0, calc(100% - 4px) 4px, 100% 4px, 100% calc(100% - 4px), calc(100% - 4px) calc(100% - 4px), calc(100% - 4px) 100%, 4px 100%, 4px calc(100% - 4px), 0 calc(100% - 4px))",
                }}
              >
                <p className="flex-1 truncate min-w-0">{p.username}</p>
                {p.ready && "✓"}
              </li>
            ))}
          </ul>
        </div>
      </div>

      {/* Right panel: Settings */}
      <div
        className="relative flex-1 flex flex-col justify-between text-white p-4 rounded shadow-md
                      border-4 border-neon-blue overflow-auto custom-scrollbar"
      >
        <div>
          <h2 className="text-2xl font-bold mb-4 text-shadow-(--text-shadow-icon)">
            Settings
          </h2>
          <div className="grid grid-cols-2 gap-4">
            {/* Number of Songs */}
            <div className="mb-2">
              <label className="block mb-1 font-medium text-sm">
                Number of Songs:
              </label>
              <select
                className="w-full border rounded mt-2 p-2 cursor-pointer bg-black text-sm"
                value={gameSettings.numSongs}
                onChange={(e) =>
                  updateGameSettings({
                    ...gameSettings,
                    numSongs: parseInt(e.target.value),
                  })
                }
              >
                {[5, 10, 20].map((val) => (
                  <option key={val} value={val}>
                    {val}
                  </option>
                ))}
              </select>
            </div>

            {/* Round Length */}
            <div className="mb-2">
              <label className="block mb-1 font-medium text-sm">
                Round Length (s):
              </label>
              <select
                className="w-full border rounded mt-2 p-2 cursor-pointer bg-black text-sm"
                value={gameSettings.roundLengthSeconds}
                onChange={(e) =>
                  updateGameSettings({
                    ...gameSettings,
                    roundLengthSeconds: parseInt(e.target.value),
                  })
                }
              >
                {[5, 10, 20, 30].map((val) => (
                  <option key={val} value={val}>
                    {val}s
                  </option>
                ))}
              </select>
            </div>

            {/* Answer Delay */}
            <div className="mb-2">
              <label className="text-gray-700 block mb-1 font-medium text-sm">
                Answer Delay (s):
              </label>
              <select
                disabled={true}
                className="w-full border text-gray-700 border-gray-700 rounded mt-2 p-2 cursor-pointer bg-black text-sm"
                value={gameSettings.answerDelaySeconds}
                onChange={(e) =>
                  updateGameSettings({
                    ...gameSettings,
                    answerDelaySeconds: parseInt(e.target.value),
                  })
                }
              >
                {[0, 1, 2, 3].map((val) => (
                  <option key={val} value={val}>
                    {val}s
                  </option>
                ))}
              </select>
            </div>

            {/* Round Delay */}
            <div className="mb-2">
              <label className="block mb-1 font-medium text-sm">
                Round Delay (s):
              </label>
              <select
                className="w-full border rounded mt-2 p-2 cursor-pointer bg-black text-sm"
                value={gameSettings.roundDelaySeconds}
                onChange={(e) =>
                  updateGameSettings({
                    ...gameSettings,
                    roundDelaySeconds: parseInt(e.target.value),
                  })
                }
              >
                {[0, 1, 2, 3].map((val) => (
                  <option key={val} value={val}>
                    {val}s
                  </option>
                ))}
              </select>
            </div>
          </div>

          {/* Playlist Link */}
          <div className="mt-4">
            <label className="block mb-1 text-sm">Spotify Playlist Link:</label>
            <input
              type="text"
              placeholder="https://open.spotify.com/playlist/..."
              className="w-full border rounded p-2 font-vt323 text-xl"
              value={gameSettings.playlistLink}
              onChange={(e) =>
                updateGameSettings({
                  ...gameSettings,
                  playlistLink: e.target.value,
                })
              }
            />
          </div>
          <ErrorSnackbar
            style={{ position: "absolute", bottom: "13.5%" }}
            error={error}
            setError={setError}
          />
        </div>

        {/* Start Game button at bottom-right */}
        <div className="flex mt-4 pt-4 border-t-2">
          <button
            onClick={() => navigate("/")}
            className="px-6 py-2 text-white font-bold mr-auto border-red-500 border-4 cursor-pointer"
          >
            Exit
          </button>
          <button
            disabled={!isValidSpotifyLink(gameSettings.playlistLink)}
            className={`px-6 py-2 text-white font-bold transition-all cursor-pointer
                            ${
                              isValidSpotifyLink(gameSettings.playlistLink) && !ready
                                ? "border-green-500 border-4"
                                : "border-red-500 border-4"
                            }
                        `}
            onClick={onReady}
          >
            {ready ? "Unready" : "Ready"}
          </button>
        </div>
      </div>
    </div>
  );
}
