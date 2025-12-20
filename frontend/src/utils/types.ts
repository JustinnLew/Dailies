type Player = {
  username: string;
  ready: boolean
};

type GuessTheSongGameSettings = {
  playlistLink: string;
  numSongs: number;
}

export type { Player, GuessTheSongGameSettings }