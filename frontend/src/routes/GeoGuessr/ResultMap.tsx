import { MapContainer, TileLayer, Marker, Popup, Polyline, ZoomControl, useMap } from "react-leaflet";
import L from "leaflet";
import { useEffect } from "react";

const PALETTE = [
  "#5b7fff", "#ff9f43", "#26de81",
  "#fd79a8", "#a29bfe", "#00cec9",
];

// Helper component to handle the fitBounds logic
function MapBoundsHandler({ bounds }: { bounds: L.LatLngBoundsExpression }) {
  const map = useMap();
  useEffect(() => {
    if (bounds && (bounds as any[]).length > 0) {
      map.fitBounds(bounds, { padding: [60, 60] });
    }
  }, []);
  return null;
}

export default function ResultMap({ correctLocation, guesses, scores }: {
  correctLocation: [number, number] | null;
  guesses: Map<string, [number, number]>;
  scores: Map<string, number>;
}) {
  if (!correctLocation) return null;

  const bounds: [number, number][] = [correctLocation, ...Array.from(guesses.values())];
  const playerColors: Record<string, string> = {};
  const sortedPlayers = [...scores.entries()].sort((a, b) => b[1] - a[1]);
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
    >
      <ZoomControl position="bottomright" />

      <TileLayer
        url="https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png"
        attribution="&copy; OpenStreetMap &copy; CartoDB"
        maxZoom={19}
      />

      <Marker position={correctLocation} icon={correctIcon}>
        <Popup>
          <b style={{ fontFamily: "monospace", color: "#ff5b5b" }}>✓ Correct Location</b>
        </Popup>
      </Marker>

      {Array.from(guesses).map(([name, pos]) => {
        const color = playerColors[name] ?? PALETTE[0];

        const guessIcon = L.divIcon({
          html: `<div style="width:14px;height:14px;background:${color};border:3px solid white;border-radius:50%;box-shadow:0 2px 6px rgba(0,0,0,0.5)"></div>`,
          className: "",
          iconSize: [14, 14],
          iconAnchor: [7, 7],
        });

        return (
          <div key={name}>
            <Polyline
              positions={[pos, correctLocation]}
              pathOptions={{
                color,
                weight: 2,
                opacity: 0.7,
                dashArray: "6 6",
              }}
            />

            {/* Player Guess Marker */}
            <Marker position={pos} icon={guessIcon}>
              <Popup>
                <span style={{ fontFamily: "monospace" }}>
                  <b style={{ color }}>{name}</b>
                  <br />
                  {Math.round(1000)} km away
                  <br />
                  {scores.get(name) ?? 0} pts total
                </span>
              </Popup>
            </Marker>
          </div>
        );
      })}

      {/* Handle fitting the map to all markers */}
      <MapBoundsHandler bounds={bounds as L.LatLngBoundsExpression} />
    </MapContainer>
  );
}