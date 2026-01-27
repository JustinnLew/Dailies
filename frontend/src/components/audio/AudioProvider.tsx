import { createContext, useContext, useEffect, useState, type ReactNode } from "react";

interface AudioContextType {
  audio: HTMLAudioElement,
  audioCtx: AudioContext,
  source: MediaElementAudioSourceNode,
}

const AudioContextInstance = createContext<AudioContextType | null>(null);

export default function AudioProvider({ children }: { children: ReactNode }) {
  const [audioState, setAudioState] = useState<AudioContextType | null>(null);

  useEffect(() => {
    const audio = new Audio();
    audio.crossOrigin = "anonymous";
    const audioCtx = new AudioContext()
    const source = audioCtx.createMediaElementSource(audio);
    source.connect(audioCtx.destination);

    setAudioState({
      audio,
      audioCtx,
      source,
    })

    return () => {
      audioCtx.close();
      source.disconnect();
    }
  }, [])
  if (!audioState) return null;

  return (
    <AudioContextInstance.Provider value={audioState}>{children}</AudioContextInstance.Provider>
  );
}

export const useAudio = () => {
  const context = useContext(AudioContextInstance);
  if (!context) {
    throw new Error("useAudio must be used within an AudioProvider");
  }
  return context;
};
