import { useEffect, useRef } from "react";

export default function AudioPlayer({ songState }: { songState: { previewUrl: string, roundStartTime: number } }) {
    const audioRef = useRef<HTMLAudioElement | null>(null);

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
        const audio = audioRef.current;
        if (!audio) return;

        if (songState.previewUrl) {
            const onLoadedMetadata = () => {
                const nowInSecs = Date.now() / 1000;
                const offset = Math.max(0, nowInSecs - songState.roundStartTime);
                
                audio.currentTime = offset;
                audio.play().catch((err) => console.warn("Playback blocked:", err));
            };
            audio.addEventListener('loadedmetadata', onLoadedMetadata, { once: true });
            audio.src = songState.previewUrl;
            audio.load(); 

        } else {
            audio.pause();
            audio.currentTime = 0;
            audio.src = "";
        }

        return () => {
            audio.removeEventListener('loadedmetadata', () => {});
        };
    }, [songState.previewUrl, songState.roundStartTime]);

    return null;
}