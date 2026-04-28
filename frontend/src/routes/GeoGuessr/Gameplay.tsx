import { useEffect, useRef } from "react";
import { Viewer } from "mapillary-js";
import "mapillary-js/dist/mapillary.css";

export default function Gameplay({imageId} : {
    imageId: string,
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewerRef = useRef<Viewer>(null);

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
        }
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
    <div ref={containerRef} className="w-screen h-screen opacity-0 transition-opacity" />
  );
}