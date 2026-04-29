import { useEffect, useState } from "react";
import "leaflet/dist/leaflet.css";
import type { GeoGuessrRoundResult, Player } from "../../utils/types";
import ResultMap from "./ResultMap";

const PALETTE = [
  "#5b7fff", "#ff9f43", "#26de81",
  "#fd79a8", "#a29bfe", "#00cec9",
];

export default function Reveal({
  correctLocation,
  results,
  time,
  scores,
  players
}: {
  correctLocation: [number, number] | null;
  results: Map<string, GeoGuessrRoundResult>;
  time: number;
  scores: Map<string, number>;
  players: Map<string, Player>;
}) {
  const [timeLeft, setTimeLeft] = useState(time);
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
        <ResultMap correctLocation={correctLocation} scores={scores} results={results} />
      </div>

      <div className="absolute top-0 right-0 h-full w-80 flex flex-col bg-black/70 backdrop-blur-sm border-l border-white/10 z-1000">

        {/* Header */}
        <div className="px-5 py-5 border-b border-white/10 shrink-0">
          <h2 className="text-3xl font-bold text-white tracking-wide">Results</h2>

          {/* Timer bar */}
          <div className="mt-4 flex items-center gap-3">
            <div className="flex-1 h-1 bg-white/10 rounded-full overflow-hidden">
              <div
                className="h-full rounded-full transition-all duration-1000 ease-linear"
                style={{ width: `${timerPct}%`, background: timerColor }}
              />
            </div>
            <span className="text-xs tabular-nums" style={{ color: timerColor }}>
              {timeLeft}s
            </span>
          </div>
        </div>

        {/* Leaderboard */}
        <div className="flex-1 overflow-y-auto py-2">
          {sortedPlayers.map(([id, totalScore], i) => {
            const player = players.get(id);
            const result = results.get(id);
            const color = playerColors[id] ?? PALETTE[0];

            return (
              <div
                key={id}
                className="flex items-center gap-3 px-5 py-3 border-b border-white/5 hover:bg-white/5 transition-colors"
              >
                {/* Rank */}
                <span className="text-xl font-bold tabular-nums w-6 shrink-0"
                  style={{ color: i < 3 ? color : "rgba(255,255,255,0.2)" }}>
                  {i + 1}
                </span>

                {/* Colour dot */}
                <div
                  className="w-2.5 h-2.5 rounded-full shrink-0"
                  style={{ background: color }}
                />

                {/* Name */}
                <span className="flex-1 text-sm text-white/80 truncate">
                  {player?.username ?? id}
                </span>

                {/* Right side: score + round info */}
                <div className="text-right shrink-0">
                  <div className="text-base font-bold text-white tabular-nums">
                    {totalScore.toLocaleString()}
                  </div>
                  {result ? (
                    <>
                      <div className="text-xs text-emerald-400">+{result.pointsGained}</div>
                      <div className="text-xs text-white/30">
                        {Math.round(result.distanceKm)} km
                      </div>
                    </>
                  ) : (
                    <div className="text-xs text-white/20">no guess</div>
                  )}
                </div>
              </div>
            );
          })}
        </div>

        {/* Next button slot */}
        <div className="px-5 py-4 shrink-0 border-t border-white/10">
          <button
            className="w-full py-3 rounded bg-blue-400 text-white text-sm tracking-widest uppercase cursor-pointer"
          >
            Next Round
          </button>
        </div>
      </div>
    </div>
  );
}