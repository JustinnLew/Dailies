type Player = {
	username: string;
	ready: boolean
	score: number
};

type GuessTheSongGameSettings = {
	playlistLink: string;
	numSongs: number;
	roundLengthSeconds: number;
	answerDelaySeconds: number;
	roundDelaySeconds: number;
}

type Song = {
	title: string;
	artists: string[];
	src: string,
}

type ChatMessage = {
  user: string;
  message: string;
}

export type { Player, GuessTheSongGameSettings, Song, ChatMessage }