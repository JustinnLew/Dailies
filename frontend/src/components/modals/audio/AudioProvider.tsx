import { createContext, useContext, type ReactNode } from "react";

const audio = new Audio();
const AudioContext = createContext<HTMLAudioElement>(audio);

export default function AudioProvider({ children }: {
  children: ReactNode;
}) {
  return <AudioContext.Provider value={audio}>{children}</AudioContext.Provider>;
}

export const useAudio = () => {
  const context = useContext(AudioContext);
  if (!context) {
    throw new Error("useAudio must be used within an AudioProvider");
  }
  return context;
};