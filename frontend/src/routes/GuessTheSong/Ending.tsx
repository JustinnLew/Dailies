import { useNavigate } from "react-router-dom";
import type { Player } from "../../utils/types";
import Crown from "../../icons/CrownIcon";
import Trophy from "../../icons/Trophy";
import { useState } from "react";
import { motion } from "motion/react";

export default function Ending({
  resetGame,
  players,
  scores,
}: {
  resetGame: () => void;
  players: Map<string, Player>;
  scores: Map<string, number>;
}) {
  let [leaderboard, _] = useState(
    Array.from(players.entries())
      .map(([id, player]) => ({
        id,
        username: player.username,
        score: scores.get(id) || 0,
      }))
      .sort((a, b) => b.score - a.score),
  );

  const navigate = useNavigate();

  return (
    <div className="scanlines h-screen overflow-y-auto custom-scrollbar flex pt-12 justify-center bg-black">
      <div className="max-w-4xl p-4 w-full flex flex-col gap-8">
        {/* Title */}
        <div className="text-center mb-6">
          <h1 className="text-4xl font-bold mb-4 font-press-start text-neon-yellow text-shadow-(--text-shadow-title)">
            GAME OVER
          </h1>
          <p className="font-vt323 text-neon-yellow text-3xl">Final Results</p>
        </div>

        {/* Top 3 */}
        <div className="hidden sm:flex items-end justify-center gap-4 mb-2 min-h-52">
          {/* 2nd Place */}
          <div className="flex flex-col items-center w-1/5 h-4/5 gap-2">
            {leaderboard[1] && <Trophy color="white" />}
            {leaderboard[1] && (
              <div
                className="flex flex-col gap-2 h-full w-full text-center justify-center border-4 border-gray-400"
                style={{
                  clipPath:
                    "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
                }}
              >
                <div className="font-press-start text-gray-400">2ND</div>
                <div className="font-press-start text-white text-sm  px-4 min-w-0 truncate">
                  {leaderboard[1].username}
                </div>
                <div className="font-vt323 text-2xl text-neon-yellow px-4 min-w-0 truncate">
                  {leaderboard[1].score}
                </div>
              </div>
            )}
          </div>

          {/* 1st Place */}
          <div className="flex flex-col items-center w-1/4 h-full gap-2">
            <Crown size={48} color="#FFD700" />
            <div
              className="flex flex-col gap-4 h-full w-full text-center justify-center border-4 border-yellow-500"
              style={{
                clipPath:
                  "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
              }}
            >
              <div className="font-press-start text-neon-yellow">WINNER</div>
              <div className="font-press-start text-sm text-white px-4 min-w-0 truncate">
                {leaderboard[0].username}
              </div>
              <div className="font-vt323 text-2xl text-neon-yellow px-4 min-w-0 truncate">
                {leaderboard[0].score}
              </div>
            </div>
          </div>

          {/* 3rd Place */}
          <div className="w-1/5 h-4/6 flex flex-col items-center gap-2">
            {leaderboard[2] && <Trophy color="#CD7F32" />}
            {leaderboard[2] && (
              <div
                className="flex flex-col gap-1 h-full w-full text-center justify-center border-4 border-yellow-700"
                style={{
                  clipPath:
                    "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
                }}
              >
                <div className="font-press-start text-yellow-700 text-xs">
                  3RD
                </div>
                <div className="font-press-start text-xs text-white px-4 min-w-0 truncate">
                  {leaderboard[2].username}
                </div>
                <div className="font-vt323 text-2xl text-neon-yellow px-4 min-w-0 truncate">
                  {leaderboard[2].score}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Full Leaderboard */}
        <div
          className="mb-8 p-6 bg-neon-blue/20 border-4 border-neon-blue"
          style={{
            clipPath:
              "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
          }}
        >
          <h3 className="text-center mb-4 font-press-start text-neon-yellow text-shadow-(--text-shadow-icon)">
            &gt; FULL STANDINGS &lt;
          </h3>
          <div className="space-y-2">
            {leaderboard.map((player, i) => (
              <motion.div
                key={player.id}
                initial={{ x: -50, opacity: 0 }}
                whileInView={{ x: 0, opacity: 1 }}
                viewport={{ once: true }}
                transition={{ delay: i * 0.1 }}
                className={`font-vt323 text-2xl flex items-center gap-4 p-3 ${i < 3 ? "bg-neon-pink/20" : "bg-neon-card"}
                            border-2 ${i < 3 ? "border-neon-pink" : "border-neon-blue"}`}
              >
                <div className="w-12 text-center text-neon-yellow">
                  #{i + 1}
                </div>
                <div className="flex-1 text-white min-w-0 truncate">
                  {player.username}
                </div>
                <div className="text-right text-neon-yellow">
                  {player.score} pts
                </div>
              </motion.div>
            ))}
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex flex-col sm:flex-row gap-4 justify-center pb-10">
          <button
            onClick={resetGame}
            className="font-press-start text-white px-8 py-4 transition-all hover:scale-105 bg-neon-pink border-4 border-neon-blue"
            style={{
              clipPath:
                "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
            }}
          >
            PLAY AGAIN
          </button>
          <button
            className="font-press-start px-8 py-4 transition-all hover:scale-105 border-4 border-neon-blue text-neon-yellow"
            onClick={() => navigate("/")}
            style={{
              clipPath:
                "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
            }}
          >
            EXIT
          </button>
        </div>
      </div>
    </div>
  );
}
