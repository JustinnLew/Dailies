import Modal from "@mui/material/Modal";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { getUserId, getUserName } from "../../utils/util";

export default function GTSHomeModal({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const navigate = useNavigate();
  const [lobbyCode, setLobbyCode] = useState("");
  const userId = getUserId();
  const userName = getUserName();
  const validLobbyCode = () => {
    return lobbyCode.length == 6;
  };
  const createLobby = async () => {
    const res = await fetch(
      "http://localhost:3000/guess-the-song/create-lobby",
      {
        method: "POST",
        body: JSON.stringify({ userId, userName }),
      },
    );
    const data = await res.json();
    console.log(`User ${userName} created lobby ${data.lobby_code}`);
    navigate(`/guess-the-song/${data.lobby_code}`);
  };

  return (
    <Modal open={open} onClose={onClose}>
      <div
        className="fixed inset-0 flex items-center justify-center p-4 scanlines backdrop-blur-md"
        onClick={onClose}
      >
        <div
          className="flex flex-col gap-6 items-center relative p-8 max-w-2xl w-full border-6 border-neon-pink"
          style={{
            clipPath:
              "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
          }}
          onClick={(e) => e.stopPropagation()}
        >
          {/* Close button */}
          <button
            onClick={onClose}
            className="text-red-500 absolute top-4 right-4 text-2xl font-bold transition-colors hover:scale-110 font-press-start"
          >
            ✕
          </button>

          <h2
            className="text-2xl font-bold text-center font-press-start text-white text-shadow-(--text-shadow-icon)
                                   pb-4 border-neon-yellow/60 border-b-4 w-full"
          >
            GUESS THE SONG
          </h2>

          {/* START GAME */}
          <h2 className="text-sm font-press-start text-neon-yellow">
            &gt; START A NEW GAME
          </h2>
          <button
            onClick={createLobby}
            className="px-8 py-4 transition-all hover:scale-105 cursor-pointer bg-neon-pink
              border-4 border-neon-blue w-full"
            style={{
              clipPath:
                "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
            }}
          >
            <span className="text-white text-sm font-press-start">
              CREATE LOBBY
            </span>
          </button>

          {/* Divider */}
          <div className="w-full relative flex items-center justify-center">
            {/* Left Line */}
            <div className="flex-1 border-t-4 border-neon-yellow/60" />
            <div className="relative px-4">
              <span className="font-press-start text-neon-yellow text-xs relative z-10">
                OR
              </span>
            </div>
            {/* Right Line */}
            <div className="flex-1 border-t-4 border-neon-yellow/60" />
          </div>

          <h2 className="text-sm font-press-start text-neon-yellow">
            &gt; JOIN EXISTING GAME
          </h2>
          <div className="flex flex-col sm:flex-row gap-4 w-full">
            <input
              type="text"
              value={lobbyCode}
              onChange={(e) => setLobbyCode(e.target.value)}
              placeholder="LOBBY CODE"
              maxLength={6}
              className="flex-1 text-white font-press-start border-4 border-neon-blue px-4 py-4 focus:outline-none text-center"
              style={{
                clipPath:
                  "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
              }}
            />
            <button
              disabled={!validLobbyCode()}
              onClick={() => navigate(`/guess-the-song/${lobbyCode}`)}
              className={`px-8 py-4 transition-all hover:scale-105 
                          ${
                            validLobbyCode()
                              ? "border-4 border-green-500 cursor-pointer"
                              : "border-4 border-red-500 cursor-not-allowed"
                          }`}
              style={{
                clipPath:
                  "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
              }}
            >
              <span className="text-sm font-press-start text-neon-yellow">
                JOIN
              </span>
            </button>
          </div>
        </div>
      </div>
    </Modal>
  );
}
