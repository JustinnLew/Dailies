import { useEffect} from "react";
import { useAudio } from "./AudioProvider";

export default function AudioPlayer({
  songState,
}: {
  songState: { previewUrl: string; roundStartTime: number };
}) {
  const audio = useAudio();

  useEffect(() => {
    if (!audio) return;
    const onLoadedMetadata = () => {
      const nowInSecs = Date.now() / 1000;
      const offset = Math.max(0, nowInSecs - songState.roundStartTime);

      audio.currentTime = offset;
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
      audio.pause();
      audio.src= "";
    };
  }, [songState.previewUrl, songState.roundStartTime]);


  return <></>;
}
