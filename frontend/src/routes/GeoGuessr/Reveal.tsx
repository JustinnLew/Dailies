import { useEffect, useState } from "react";
import "leaflet/dist/leaflet.css";
import type { GeoGuessrRoundResult, Player } from "../../utils/types";
import ResultMap from "./ResultMap";
import { AnimatePresence, motion } from "motion/react";

const PALETTE = [
  "#5b7fff",
  "#ff9f43",
  "#26de81",
  "#fd79a8",
  "#a29bfe",
  "#00cec9",
];

export default function Reveal({
  correctLocation,
  results,
  time,
  scores,
  players,
}: {
  correctLocation: [number, number] | null;
  results: Map<string, GeoGuessrRoundResult>;
  time: number;
  scores: Map<string, number>;
  players: Map<string, Player>;
}) {
  const [timeLeft, setTimeLeft] = useState(Math.max(time, 0));
  //   const [nextEnabled, setNextEnabled] = useState(false);

  const sortedPlayers = [...scores.entries()].sort((a, b) => b[1] - a[1]);
  const playerColors: Record<string, string> = {};
  sortedPlayers.forEach(([name], i) => {
    playerColors[name] = PALETTE[i % PALETTE.length];
  });

  useEffect(() => {
    const t = setTimeout(() => setTimeLeft((v) => v - 1), 1000);
    return () => clearTimeout(t);
  }, [timeLeft]);

  const timerPct = (timeLeft / time) * 100;
  const timerColor = timeLeft <= 5 ? "#ff5b5b" : "#5b7fff";

  return (
    <div className="relative w-screen h-screen overflow-hidden bg-black">
      <div className="absolute inset-0">
        <ResultMap
          correctLocation={correctLocation}
          scores={scores}
          results={results}
        />
      </div>

      <div className="absolute top-0 right-0 h-full w-80 flex flex-col bg-black/70 backdrop-blur-sm border-l border-white/10 z-1000 scanlines">
        {/* Header */}
        <div className="px-5 py-5 border-b border-white/10 shrink-0">
          <h2 className="text-2xl font-press-start text-shadow-(--text-shadow-title) text-white tracking-wide">
            Results
          </h2>

          {/* Timer bar */}
          <div className="mt-4 flex items-center gap-3">
            <div className="flex-1 h-1 bg-white/10 rounded-full overflow-hidden">
              <div
                className="h-full rounded-full transition-all duration-1000 ease-linear font-vt323"
                style={{ width: `${timerPct}%`, background: timerColor }}
              />
            </div>
            <span
              className="text-xs tabular-nums"
              style={{ color: timerColor }}
            >
              {timeLeft}s
            </span>
          </div>
        </div>

        {/* Leaderboard */}
        <div className="flex-1 overflow-y-auto py-2 custom-scrollbar">
          <AnimatePresence>
            {sortedPlayers.map(([id, totalScore], i) => {
              const player = players.get(id);
              const result = results.get(id);
              const color = playerColors[id] ?? PALETTE[0];

              return (
                <motion.div
                  layout
                  key={id}
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, scale: 0.95 }}
                  transition={{
                    type: "spring",
                    stiffness: 300,
                    damping: 30,
                    opacity: { duration: 0.2 },
                  }}
                  className="flex items-center gap-3 px-5 py-3 border-b-2 border-gray-700 bg-black/20 hover:bg-white/5 transition-colors"
                >
                  {/* Rank */}
                  <span className="text-neon-yellow font-vt323 text-xl w-8 shrink-0">
                    #{i + 1}
                  </span>

                  {/* Colour */}
                  <div
                    className="w-2.5 h-2.5 shrink-0"
                    style={{ background: color }}
                  />

                  {/* Name */}
                  <span className="flex-1 font-vt323 text-xl text-white/80 truncate">
                    {player?.username ?? "PLAYER"}
                  </span>

                  {/* Right side: score + round info */}
                  <div className="text-right shrink-0">
                    <div className="text-xs font-bold text-white font-press-start">
                      {totalScore.toLocaleString()}
                    </div>
                    {result ? (
                      <>
                        <div className="text-lg text-emerald-400 font-vt323">
                          +{result.pointsGained}
                        </div>
                        <div className="text-xs text-white/60">
                          {Math.round(result.distanceKm)} km
                        </div>
                      </>
                    ) : (
                      <div className="text-xs text-white/20">no guess</div>
                    )}
                  </div>
                </motion.div>
              );
            })}
          </AnimatePresence>
        </div>

        {/* Next button slot */}
        <div className="px-5 py-4 shrink-0 border-t border-white/10">
          <button className="w-full py-3 rounded bg-neon-pink text-white text-sm tracking-widest uppercase cursor-pointer font-press-start">
            Next Round
          </button>
        </div>
      </div>
    </div>
  );
}
