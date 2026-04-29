import { useEffect, useRef, useState } from "react";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
import type { Player } from "../../utils/types";
import { MapContainer, Marker, Popup, TileLayer } from "react-leaflet";
import ResultMap from "./ResultMap";

const PALETTE = [
  "#5b7fff", "#ff9f43", "#26de81",
  "#fd79a8", "#a29bfe", "#00cec9",
];

function haversineKm(lat1: number, lng1: number, lat2: number, lng2: number) {
  const R = 6371;
  const dLat = ((lat2 - lat1) * Math.PI) / 180;
  const dLng = ((lng2 - lng1) * Math.PI) / 180;
  const a =
    Math.sin(dLat / 2) ** 2 +
    Math.cos((lat1 * Math.PI) / 180) *
      Math.cos((lat2 * Math.PI) / 180) *
      Math.sin(dLng / 2) ** 2;
  return R * 2 * Math.asin(Math.sqrt(a));
}

function haversineScore(lat1: number, lng1: number, lat2: number, lng2: number) {
  return Math.round(5000 * Math.exp(-haversineKm(lat1, lng1, lat2, lng2) / 2000));
}

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
  const mapRef = useRef<HTMLDivElement>(null);
  const leafletRef = useRef<L.Map | null>(null);
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