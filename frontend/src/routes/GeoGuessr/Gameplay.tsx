import { useEffect, useRef, useState } from "react";
import { Viewer } from "mapillary-js";
import "mapillary-js/dist/mapillary.css";
import { MapContainer, TileLayer, Marker, useMapEvents, useMap } from "react-leaflet";
import { motion } from "motion/react";

export default function Gameplay({ imageId }: { imageId: string }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewerRef = useRef<Viewer>(null);
  const [position, setPosition] = useState<[number, number] | null>(null);
  const [mapExpanded, setMapExpanded] = useState(false);

  const MapEvents = () => {
    useMapEvents({
      click(e) {
        setPosition([e.latlng.lat, e.latlng.lng])
      },
    });
    return false;
  };

  const MapResizer = ({ expanded }: { expanded: boolean }) => {
    const map = useMap();
    useEffect(() => {
      setTimeout(() => map.invalidateSize(), 210);
    }, [expanded]);
    return null;
  };

  useEffect(() => {
    if (!containerRef.current || viewerRef.current) return;

    viewerRef.current = new Viewer({
      accessToken: "MLY|27623692320552297|6e04b1092adbc64d5d6eeb79a69570d6",
      container: containerRef.current,
      imageId: imageId,
      component: {
        cover: false,
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
    <div className="relative w-full h-screen overflow-hidden">
      <div
        ref={containerRef}
        className="absolute inset-0 w-full h-full opacity-0 transition-opacity duration-1000"
      />
      <motion.div
        animate={mapExpanded ? { width: "75%", height: "83.333%" } : { width: "25%", height: "33.333%" }}
        transition={{ duration: 0.2, ease: "easeInOut" }}
        className="fixed bottom-4 left-4 border-2 border-white/20 rounded-lg shadow-2xl overflow-hidden"
      >
        <button
          onClick={() => setMapExpanded(!mapExpanded)}
          className="absolute top-2 right-2 z-1000 bg-black/50 hover:bg-black/70 text-white rounded p-1 transition-colors"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            {mapExpanded
              ? <path d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3" />
              : <path d="M15 3h6m0 0v6m0-6-7 7M9 21H3m0 0v-6m0 6 7-7" />
            }
          </svg>
        </button>
        <MapContainer
          className="w-full h-full"
          center={[0, 0]}
          zoom={3}
        >
          <TileLayer
            attribution='&copy; <a href="https://www.openstreetmap.org/copyright" target="_blank">OpenStreetMap</a>'
            url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
          />
          {position && <Marker position={position} />}
          <MapEvents />
          <MapResizer expanded={mapExpanded} />
        </MapContainer>
      </motion.div>
    </div>
  );
}
