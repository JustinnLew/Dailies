import { useEffect, useRef } from "react";

export default function AudioPlayer({ previewUrl }: { previewUrl: string }) {
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

    // Handle Song changes
    useEffect(() => {
        const audio = audioRef.current;
        if (!audio) return;

        if (previewUrl) {
            audio.src = previewUrl;
            audio.load(); 
            
            audio.play().catch((err) => {
                console.warn("Playback error", err);
            });
        } else {
            audio.pause();
            audio.currentTime = 0;
        }
    }, [previewUrl]);

    return null;
}