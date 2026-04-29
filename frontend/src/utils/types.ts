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

type GeoGuessrGameSettings = {
  numRounds: number;
  roundLengthSeconds: number;
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

type GeoGuessrRoundResult = {
  distanceKm : number,
  pointsGained: number,
  guess: [number, number]
}

type GeoGuesserGameState =
  | "answer_reveal"
  | "playing"
  | "waiting"
  | "connecting"
  | "loading"
  | "finished";
type GameState = "playing" | "waiting" | "connecting" | "loading" | "finished";

export const reservedUsernames = ["SYSTEM", "ERROR"];

export type {
  Player,
  GameState,
  GuessTheSongGameSettings,
  Song,
  ChatMessage,
  GeoGuessrGameSettings,
  GeoGuesserGameState,
  GeoGuessrRoundResult
};
