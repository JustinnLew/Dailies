import { MapContainer, TileLayer, Marker, Popup, Polyline, ZoomControl, useMap, AttributionControl } from "react-leaflet";
import L from "leaflet";
import { useEffect } from "react";
import type { GeoGuessrRoundResult } from "../../utils/types";

const PALETTE = [
  "#5b7fff", "#ff9f43", "#26de81",
  "#fd79a8", "#a29bfe", "#00cec9",
];

function MapBoundsHandler({ bounds }: { bounds: [number, number][] }) {
  const map = useMap();
  useEffect(() => {
    if (bounds.length > 0) {
      map.fitBounds(bounds as L.LatLngBoundsExpression, { padding: [60, 60] });
    }
  }, []);
  return null;
}

export default function ResultMap({ correctLocation, results, scores }: {
  correctLocation: [number, number] | null;
  results: Map<string, GeoGuessrRoundResult>;
  scores: Map<string, number>;
}) {
  if (!correctLocation) return null;

  const bounds: [number, number][] = [
    correctLocation,
    ...Array.from(results.values())
      .filter(r => r.guess != null)
      .map(r => r.guess),
  ];

  const sortedPlayers = [...scores.entries()].sort((a, b) => b[1] - a[1]);
  const playerColors: Record<string, string> = {};
  sortedPlayers.forEach(([name], i) => {
    playerColors[name] = PALETTE[i % PALETTE.length];
  });

  const correctIcon = L.divIcon({
    html: `<div style="width:18px;height:18px;background:#ff5b5b;border:3px solid white;border-radius:50%;box-shadow:0 0 0 3px rgba(255,91,91,0.3)"></div>`,
    className: "",
    iconSize: [18, 18],
    iconAnchor: [9, 9],
  });

  return (
    <MapContainer
      center={[30, 15]}
      zoom={2}
      maxZoom={19}
      minZoom={3}
      zoomControl={false}
      className="w-screen h-screen overflow-hidden"
      attributionControl={false}
    >
      <ZoomControl position="topleft" />
      <TileLayer
        url="https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png"
        attribution="&copy; OpenStreetMap &copy; CartoDB"
        maxZoom={19}
      />
      <AttributionControl position="bottomleft" />

      <Marker position={correctLocation} icon={correctIcon}>
        <Popup>
          <b style={{ fontFamily: "monospace", color: "#ff5b5b" }}>✓ Correct Location</b>
        </Popup>
      </Marker>

      {Array.from(results.entries()).map(([name, result]) => {
        if (!result.guess) return null;
        const color = playerColors[name] ?? PALETTE[0];

        const guessIcon = L.divIcon({
          html: `<div style="width:14px;height:14px;background:${color};border:3px solid white;border-radius:50%;box-shadow:0 2px 6px rgba(0,0,0,0.5)"></div>`,
          className: "",
          iconSize: [14, 14],
          iconAnchor: [7, 7],
        });

        return (
          <div key={`line-${name}`}>
            <Polyline
              positions={[result.guess, correctLocation]}
              pathOptions={{ color, weight: 2, opacity: 0.7, dashArray: "6 6" }}
            />
            <Marker key={`marker-${name}`} position={result.guess} icon={guessIcon}/>
          </div>
        );
      })}

      <MapBoundsHandler bounds={bounds} />
    </MapContainer>
  );
}