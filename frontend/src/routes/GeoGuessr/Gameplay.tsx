import { useEffect, useRef, useState } from "react";
import { Viewer } from "mapillary-js";
import L from "leaflet";
import "mapillary-js/dist/mapillary.css";
import {
  MapContainer,
  TileLayer,
  Marker,
  useMapEvents,
  useMap,
} from "react-leaflet";
import { motion, AnimatePresence } from "motion/react";

export default function Gameplay({
  imageId,
  sendGuess,
  time,
  center,
  zoom,
}: {
  imageId: string;
  sendGuess: (guess: [number, number]) => void;
  time: number;
  center: [number, number];
  zoom: number;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewerRef = useRef<Viewer>(null);
  const [position, setPosition] = useState<[number, number] | null>(null);
  const [mapExpanded, setMapExpanded] = useState(false);
  const [guessed, setGuessed] = useState(false);
  const [showCredits, setShowCredits] = useState(false);
  const [timeLeft, setTimeLeft] = useState(Math.max(time, 0));

  useEffect(() => {
    if (timeLeft <= 0) return;
    const t = setTimeout(() => setTimeLeft((v) => v - 1), 1000);
    return () => clearTimeout(t);
  }, [timeLeft]);

  const timerPct = Math.max(0, (timeLeft / time) * 100);
  const timerColor =
    timeLeft <= 10 ? "#ff5b5b" : timeLeft <= 20 ? "#ff9f43" : "#26de81";

  const MapEvents = () => {
    useMapEvents({
      click(e) {
        setPosition([e.latlng.lat, e.latlng.lng]);
      },
    });
    return null;
  };

  const MapResizer = ({ expanded }: { expanded: boolean }) => {
    const map = useMap();
    useEffect(() => {
      setTimeout(() => map.invalidateSize(), 210);
    }, [expanded]);
    return null;
  };

  useEffect(() => {
    if (viewerRef.current && imageId) {
      viewerRef.current.moveTo(imageId).catch((error) => {
        console.warn("Failed to move to imageId:", error);
      });
    }
  }, [imageId]);

  useEffect(() => {
    if (!containerRef.current || viewerRef.current) return;

    viewerRef.current = new Viewer({
      accessToken: import.meta.env.VITE_MAPILLARY_API_KEY,
      container: containerRef.current,
      imageId: imageId || undefined,
      component: {
        cover: false,
        cache: false,
        keyboard: false,
      },
    });

    viewerRef.current?.on("image", () => {
      containerRef.current?.classList.replace("opacity-0", "opacity-100");
    });

    return () => {
      viewerRef.current?.remove();
      viewerRef.current = null;
    };
  }, []);

  return (
    <div className="relative w-full h-screen overflow-hidden bg-black">
      <div
        ref={containerRef}
        className="absolute inset-0 w-full h-full opacity-0 transition-opacity duration-1000"
      />
      {/* Timer bar */}
      <div className="absolute top-0 left-0 right-0 z-1000 h-1">
        <div
          className="h-full transition-all duration-1000 ease-linear"
          style={{ width: `${timerPct}%`, background: timerColor }}
        />
      </div>

      {/* Timer label */}
      <div
        className="absolute top-3 left-3 z-1000 font-press-start text-md px-2 py-0.5 rounded bg-black/40"
        style={{ color: timerColor }}
      >
        {timeLeft}s
      </div>
      {!guessed && (
        <motion.div
          animate={
            mapExpanded
              ? { width: "70%", height: "80%" }
              : { width: "25%", height: "30%" }
          }
          transition={{ duration: 0.2, ease: "easeInOut" }}
          className="fixed bottom-4 left-4 border-2 border-white/20 rounded-lg shadow-2xl overflow-hidden"
        >
          <button
            onClick={() => setMapExpanded(!mapExpanded)}
            className="absolute top-2 right-2 z-1000 bg-black/50 hover:bg-black/70 text-white rounded p-1 transition-colors"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              {mapExpanded ? (
                <path d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3" />
              ) : (
                <path d="M15 3h6m0 0v6m0-6-7 7M9 21H3m0 0v-6m0 6 7-7" />
              )}
            </svg>
          </button>

          <button
            onClick={(e) => {
              e.stopPropagation();
              setShowCredits(true);
            }}
            className="absolute bottom-2 right-2 z-1000 bg-black/40 hover:bg-black/60 text-white/80 rounded-full w-fit h-fit px-2 text-md
            text-center"
          >
            i
          </button>

          <AnimatePresence>
            {showCredits && (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                onClick={() => setShowCredits(false)}
                className="absolute inset-0 z-1000 bg-black/80 flex items-center justify-center p-4 text-center cursor-pointer"
              >
                <div>
                  <p className="font-bold mb-1 text-white text-md">
                    Map Credits
                  </p>
                  <p className="text-sm text-gray-300">
                    Tiles &copy; Esri &mdash; Source: Esri, DeLorme, NAVTEQ,
                    USGS, Intermap, iPC, NRCAN, Esri Japan, METI, Esri China
                    (Hong Kong), Esri (Thailand), TomTom, 2012
                  </p>
                  <p className="mt-2 text-blue-400">Click to close</p>
                </div>
              </motion.div>
            )}
          </AnimatePresence>

          <MapContainer
            className="w-full h-full"
            center={center}
            zoom={zoom}
            attributionControl={false}
          >
            <TileLayer
              url="https://server.arcgisonline.com/ArcGIS/rest/services/World_Street_Map/MapServer/tile/{z}/{y}/{x}"
              attribution="Tiles &copy; Esri &mdash; Source: Esri, DeLorme, NAVTEQ, USGS, Intermap, iPC, NRCAN, Esri Japan, METI, Esri China (Hong Kong), Esri (Thailand), TomTom, 2012"
            />
            {position && (
              <Marker
                position={position}
                icon={L.divIcon({
                  html: `
                    <div style="position:relative;width:22px;height:22px">
                      <div style="position:absolute;inset:0;border-radius:50%;background:rgba(255,91,91,0.4);animation:ping 1.2s ease-out infinite"></div>
                      <div style="position:absolute;inset:3px;border-radius:50%;background:#ff5b5b;border:2px solid white"></div>
                    </div>
                    <style>@keyframes ping{0%{transform:scale(1);opacity:0.8}100%{transform:scale(2.2);opacity:0}}</style>`,
                  className: "",
                  iconSize: [24, 24],
                  iconAnchor: [8, 8],
                })}
              />
            )}
            <MapEvents />
            <MapResizer expanded={mapExpanded} />
          </MapContainer>
          <button
            disabled={!position}
            onClick={() => {
              if (position) {
                sendGuess(position);
                setGuessed(true);
              }
            }}
            className={`font-semibold font-mono text-lg absolute bottom-3 left-3 z-1000 w-1/4
            ${position ? "bg-neon-green/60 cursor-pointer hover:bg-neon-green" : "bg-red-500 cursor-not-allowed"}
            text-white rounded transition-colors px-2 py-1 font-press-start}`}
          >
            Guess
          </button>
        </motion.div>
      )}
    </div>
  );
}
