type Player = {
	username: string;
	ready: boolean
};

type GuessTheSongGameSettings = {
	playlistLink: string;
	numSongs: number;
	roundLengthSeconds: number;
	answerDelaySeconds: number;
}

type Song = {
	title: string;
	artists: string[];
	src: string,
}

export type { Player, GuessTheSongGameSettings, Song }