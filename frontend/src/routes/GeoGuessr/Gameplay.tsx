import { useEffect, useRef, useState } from "react";
import { Viewer } from "mapillary-js";
import "mapillary-js/dist/mapillary.css";
import { MapContainer, TileLayer, Marker, useMapEvents } from "react-leaflet";

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

  useEffect(() => {
    if (!containerRef.current || viewerRef.current) return;

    viewerRef.current = new Viewer({
      accessToken: "MLY|27623692320552297|6e04b1092adbc64d5d6eeb79a69570d6",
      container: containerRef.current,
      imageId: imageId,
      component: {
        cover: false,
        keyboard: false,
        attribution: true,
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
    <div className="w-screen h-screen relative">
      <div ref={containerRef} className="w-screen h-screen opacity-0 transition-opacity" />
      <MapContainer className="absolute w-1/4 h-1/3 bottom-0 right-0" center={[0, 0]} zoom={3}>
        <TileLayer
          attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />
        {position && <Marker position={position} />}
        <MapEvents />
      </MapContainer>
    </div>
  );
}
