import {
  MapContainer,
  TileLayer,
  Marker,
  Popup,
  Polyline,
  ZoomControl,
  useMap,
  AttributionControl,
} from "react-leaflet";
import L from "leaflet";
import { useEffect } from "react";
import type { GeoGuessrRoundResult } from "../../utils/types";

const PALETTE = [
  "#1a3aff", // deep blue
  "#e65c00", // burnt orange
  "#00843d", // forest green
  "#c2006b", // deep pink
  "#5c00d4", // deep purple
  "#007a7a", // dark teal
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

export default function ResultMap({
  correctLocation,
  results,
  scores,
}: {
  correctLocation: [number, number] | null;
  results: Map<string, GeoGuessrRoundResult>;
  scores: Map<string, number>;
}) {
  if (!correctLocation) return null;

  const bounds: [number, number][] = [
    correctLocation,
    ...Array.from(results.values())
      .filter((r) => r.guess != null)
      .map((r) => r.guess),
  ];

  const sortedPlayers = [...scores.entries()].sort((a, b) => b[1] - a[1]);
  const playerColors: Record<string, string> = {};
  sortedPlayers.forEach(([name], i) => {
    playerColors[name] = PALETTE[i % PALETTE.length];
  });

  // Pin-shaped correct location icon (circle + pointer tip)
  const correctIcon = L.divIcon({
    html: `
      <style>
        @keyframes correctPing {
          0%   { transform: scale(1); opacity: 0.9 }
          100% { transform: scale(2.8); opacity: 0 }
        }
      </style>
      <div style="position:relative;width:32px;height:40px">
        <!-- pulsing ring -->
        <div style="
          position:absolute;top:0;left:0;
          width:32px;height:32px;
          border-radius:50%;
          background:rgba(255,60,60,0.35);
          animation:correctPing 1.4s ease-out infinite;
        "></div>
        <!-- outer circle -->
        <div style="
          position:absolute;top:0;left:0;
          width:32px;height:32px;
          border-radius:50%;
          background:#ff3c3c;
          border:3px solid #fff;
          box-shadow:0 0 0 2px #ff3c3c;
          display:flex;align-items:center;justify-content:center;
        ">
          <!-- checkmark -->
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <polyline points="2.5,7 5.5,10 11.5,4" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
        <!-- downward pin tip -->
        <div style="
          position:absolute;bottom:0;left:50%;
          transform:translateX(-50%);
          width:0;height:0;
          border-left:7px solid transparent;
          border-right:7px solid transparent;
          border-top:10px solid #ff3c3c;
        "></div>
      </div>`,
    className: "",
    iconSize: [32, 40],
    iconAnchor: [16, 40],
  });

  return (
    <MapContainer
      zoom={4}
      maxZoom={19}
      minZoom={2}
      zoomControl={false}
      className="w-screen h-screen overflow-hidden"
      attributionControl={false}
    >
      <ZoomControl position="topleft" />
      <TileLayer
        url="https://server.arcgisonline.com/ArcGIS/rest/services/World_Street_Map/MapServer/tile/{z}/{y}/{x}"
        attribution="Tiles &copy; Esri &mdash; Source: Esri, DeLorme, NAVTEQ, USGS, Intermap, iPC, NRCAN, Esri Japan, METI, Esri China (Hong Kong), Esri (Thailand), TomTom, 2012"
        maxZoom={19}
        minZoom={2}
      />
      <AttributionControl position="bottomleft" />

      <Marker position={correctLocation} icon={correctIcon}>
        <Popup>
          <b style={{ fontFamily: "monospace", color: "#ff3c3c" }}>
            ✓ Correct Location
          </b>
        </Popup>
      </Marker>

      {sortedPlayers.map(([name], rank) => {
        const result = results.get(name);
        if (!result?.guess) return null;
        const color = playerColors[name] ?? PALETTE[0];

        const guessIcon = L.divIcon({
          html: `
            <div style="
              position:relative;
              width:28px;height:36px;
            ">
              <!-- outer circle -->
              <div style="
                position:absolute;top:0;left:0;
                width:28px;height:28px;
                border-radius:50%;
                background:${color};
                border:3px solid #fff;
                box-shadow:0 0 0 2px ${color}, 0 3px 8px rgba(0,0,0,0.55);
                display:flex;align-items:center;justify-content:center;
              ">
                <span style="
                  color:#fff;
                  font-size:11px;
                  font-weight:700;
                  font-family:monospace;
                  line-height:1;
                  text-shadow:0 1px 2px rgba(0,0,0,0.5);
                ">${rank + 1}</span>
              </div>
              <!-- pin tip -->
              <div style="
                position:absolute;bottom:0;left:50%;
                transform:translateX(-50%);
                width:0;height:0;
                border-left:6px solid transparent;
                border-right:6px solid transparent;
                border-top:9px solid ${color};
              "></div>
            </div>`,
          className: "",
          iconSize: [28, 36],
          iconAnchor: [14, 36],
        });

        return (
          <div key={`group-${name}`}>
            <Polyline
              positions={[result.guess, correctLocation]}
              pathOptions={{
                color,
                weight: 3,
                opacity: 0.85,
                dashArray: "8 6",
              }}
            />
            <Marker position={result.guess} icon={guessIcon} />
          </div>
        );
      })}

      <MapBoundsHandler bounds={bounds} />
    </MapContainer>
  );
}