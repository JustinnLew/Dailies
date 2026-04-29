import { useEffect, useRef, useState } from "react";
import "leaflet/dist/leaflet.css";
import type { Player } from "../../utils/types";
import ResultMap from "./ResultMap";

const PALETTE = [
  "#5b7fff", "#ff9f43", "#26de81",
  "#fd79a8", "#a29bfe", "#00cec9",
];

export default function Reveal({
  correctLocation,
  guesses,
  time,
  scores,
  players
}: {
  correctLocation: [number, number] | null;
  guesses: Map<string, [number, number]>;
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
    <div className="w-screen h-screen overflow-hidden bg-black">

      {/* Map */}
      <ResultMap correctLocation={correctLocation} scores={scores} guesses={guesses} />

      {/* Sidebar */}

        {/* Header */}

        {/* Leaderboard */}


        {/* Next button
        <button
          disabled={!nextEnabled}
          onClick={onNext}
          style={{
            margin: "16px 20px", padding: 12,
            background: nextEnabled ? "#5b7fff" : "#2a2a3a",
            border: "none", borderRadius: 4,
            fontFamily: "'DM Mono', monospace", fontSize: 12,
            letterSpacing: "0.15em", textTransform: "uppercase",
            color: nextEnabled ? "white" : "#555570",
            cursor: nextEnabled ? "pointer" : "not-allowed",
            transition: "background 0.15s",
          }}
        >
          Next Round
        </button> */}
    </div>
  );
}