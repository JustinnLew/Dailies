type Player = {
  username: string;
  ready: boolean;
};

type GuessTheSongGameSettings = {
  playlistLink: string;
  numSongs: number;
  roundLengthSeconds: number;
  answerDelaySeconds: number;
  roundDelaySeconds: number;
};

type Song = {
  title: string;
  artists: string[];
  src: string;
};

type ChatMessage = {
  user: string;
  message: string;
};

type GameState = "playing" | "waiting" | "connecting" | "loading" | "finished";

export const reservedUsernames = ["SYSTEM", "ERROR"];

export type { Player, GameState, GuessTheSongGameSettings, Song, ChatMessage };
