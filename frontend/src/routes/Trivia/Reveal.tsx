import { motion } from "motion/react";
import type { Player } from "../../utils/types";

export default function Reveal({
  correctAnswer,
  scores,
  players,
}: {
  correctAnswer: string;
  scores: Map<string, number>;
  players: Map<string, Player>;
}) {
  const leaderboard = [...scores.entries()]
    .map(([id, score]) => ({
      id,
      username: players.get(id)?.username || "Unknown",
      score,
    }))
    .sort((a, b) => b.score - a.score);

  return (
    <div className="relative w-screen h-screen overflow-hidden bg-black scanlines">
      <div className="flex flex-col items-center justify-center h-full p-4">
        <motion.div
          initial={{ scale: 0.8, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          transition={{ duration: 0.5, delay: 0.2 }}
          className="text-center mb-12"
        >
          <h2 className="font-press-start text-neon-yellow text-shadow-(--text-shadow-title) text-2xl mb-4">
            CORRECT ANSWER
          </h2>
          <p className="font-vt323 text-white text-3xl md:text-4xl max-w-3xl">
            {correctAnswer}
          </p>
        </motion.div>

        <motion.div
          initial={{ y: 50, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ duration: 0.5, delay: 0.4 }}
          className="w-full max-w-2xl"
        >
          <h3 className="font-press-start text-neon-yellow text-shadow-(--text-shadow-icon) text-xl mb-4 text-center">
            &gt; SCOREBOARD &lt;
          </h3>
          <div className="space-y-2">
            {leaderboard.map((player, i) => (
              <motion.div
                key={player.id}
                initial={{ x: -30, opacity: 0 }}
                animate={{ x: 0, opacity: 1 }}
                transition={{ delay: 0.5 + i * 0.1 }}
                className={`font-vt323 text-xl flex items-center gap-4 p-3
                  ${i < 3 ? "bg-neon-pink/20" : "bg-neon-card"}
                  border-2 ${i < 3 ? "border-neon-pink" : "border-neon-blue"}`}
                style={{
                  clipPath:
                    "polygon(0 4px, 4px 4px, 4px 0, calc(100% - 4px) 0, calc(100% - 4px) 4px, 100% 4px, 100% calc(100% - 4px), calc(100% - 4px) calc(100% - 4px), calc(100% - 4px) 100%, 4px 100%, 4px calc(100% - 4px), 0 calc(100% - 4px))",
                }}
              >
                <div className="w-12 text-center text-neon-yellow font-bold">
                  #{i + 1}
                </div>
                <div className="flex-1 text-white min-w-0 truncate">
                  {player.username}
                </div>
                <div className="text-right text-neon-yellow font-bold">
                  {player.score} pts
                </div>
              </motion.div>
            ))}
          </div>
        </motion.div>
      </div>
    </div>
  );
}
