import { useEffect, useState, type ChangeEvent } from "react";
import { useAudio } from "./AudioProvider";

const STORAGE_KEY = "audio-player-volume";

export default function AudioVolumeSlider() {
    const audio = useAudio();
    const [volume, setVolume] = useState(() => {
        if (typeof window !== "undefined") {
          const saved = localStorage.getItem(STORAGE_KEY);
          return saved !== null ? parseFloat(saved) : 0.5;
        }
        return 0.5;
      });
    
      const handleVolumeChange = (e: ChangeEvent<HTMLInputElement>) => {
        setVolume(parseFloat(e.target.value));
      };

    useEffect(() => {
        audio.volume = volume;
        localStorage.setItem(STORAGE_KEY, volume.toString());
    }, [volume]);

    
    return (
    <div className="flex flex-col gap-2 p-2">
      <input
        id="volume-slider"
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={volume}
        onChange={handleVolumeChange}
        style={{
          writingMode: "vertical-rl",
          transform: "rotate(180deg)",
        }}
      />
      <span className="text-xs w-12 text-center">
        {Math.round(volume * 100)}%
      </span>
    </div>
  );
}