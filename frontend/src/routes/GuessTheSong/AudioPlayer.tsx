import { useEffect, useRef, useState, type ChangeEvent } from "react";

const STORAGE_KEY = "audio-player-volume";

export default function AudioPlayer({
  songState,
}: {
  songState: { previewUrl: string; roundStartTime: number };
}) {
  const audioRef = useRef<HTMLAudioElement | null>(null);

  const [volume, setVolume] = useState(() => {
    if (typeof window !== "undefined") {
      const saved = localStorage.getItem(STORAGE_KEY);
      return saved !== null ? parseFloat(saved) : 0.5;
    }
    return 0.5;
  });

  // Initialize Audio object
  useEffect(() => {
    if (!audioRef.current) {
      audioRef.current = new Audio();
    }
    return () => {
      if (audioRef.current) {
        audioRef.current.pause();
        audioRef.current.src = "";
        audioRef.current = null;
      }
    };
  }, []);

  useEffect(() => {
    if (audioRef.current) {
      audioRef.current.volume = volume;
    }
    localStorage.setItem(STORAGE_KEY, volume.toString());
  }, [volume]);

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    const onLoadedMetadata = () => {
      const nowInSecs = Date.now() / 1000;
      const offset = Math.max(0, nowInSecs - songState.roundStartTime);

      audio.currentTime = offset;
      audio.volume = volume; 
      audio.play().catch((err) => console.warn("Playback blocked:", err));
    };

    if (songState.previewUrl) {
      audio.addEventListener("loadedmetadata", onLoadedMetadata);
      audio.src = songState.previewUrl;
    } else {
      audio.pause();
      audio.currentTime = 0;
      audio.src = "";
    }

    return () => {
      audio.removeEventListener("loadedmetadata", onLoadedMetadata);
    };
  }, [songState.previewUrl, songState.roundStartTime]);

  const handleVolumeChange = (e: ChangeEvent<HTMLInputElement>) => {
    setVolume(parseFloat(e.target.value));
  };

  return (
    <div style={{ display: "flex", alignItems: "center", gap: "10px", padding: "10px" }}>
      <label htmlFor="volume-slider" style={{ fontSize: "14px", fontWeight: "bold" }}>
        Volume
      </label>
      <input
        id="volume-slider"
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={volume}
        onChange={handleVolumeChange}
      />
      <span>{Math.round(volume * 100)}%</span>
    </div>
  );
}