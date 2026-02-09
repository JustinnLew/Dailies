import { useEffect, useState, useRef, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import WaitingRoom from "./WaitingRoom";
import Gameplay from "./Gameplay";
import.meta.env;
import type {
  Player,
  GuessTheSongGameSettings,
  ChatMessage,
  GameState,
} from "../../utils/types";
import Connecting from "../../components/loading/Connecting";
import PreparingPlaylist from "../../components/loading/PreparingPlaylist";
import Ending from "./Ending";
import { WS_URL } from "../../apiConfig";

export default function Game() {
  const params = useParams();
  const [playerReady, setPlayerReady] = useState(false);
  const username = localStorage.getItem("username") || "PLAYER";
  const [players, setPlayers] = useState<Map<string, Player>>(new Map());
  const [error, setError] = useState<string>("");
  const [gameSettings, setGameSettings] = useState<GuessTheSongGameSettings>({
    playlistLink: "",
    numSongs: 10,
    roundLengthSeconds: 30,
    answerDelaySeconds: 0,
    roundDelaySeconds: 3,
  });
  const navigate = useNavigate();
  const socket = useRef<WebSocket>(null!);
  const updateGameSettings = useCallback(
    (settings: GuessTheSongGameSettings) => {
      setGameSettings(settings);

      if (socket.current && socket.current.readyState === WebSocket.OPEN) {
        socket.current.send(
          JSON.stringify({
            event: "UpdateGameSettings",
            settings: settings,
          }),
        );
      }
    },
    [],
  );
  const [songState, setSongState] = useState<{
    previewUrl: string;
    roundStartTime: number;
  }>({ previewUrl: "", roundStartTime: 0 });
  const [gameState, setGameState] = useState<GameState>("connecting");
  const [chat, setChat] = useState<ChatMessage[]>([]);
  const [scores, setScores] = useState<Map<string, number>>(new Map());

  const resetGame = () => {
    setGameState("waiting");
    setChat([]);
    setError("");
    setScores(new Map([...scores.keys()].map((key) => [key, 0])));
    setPlayerReady(false);
  };

  useEffect(() => {
    const s = new WebSocket(`${WS_URL}/guess-the-song`);
    socket.current = s;

    s.onopen = () => {
      socket.current.send(
        JSON.stringify({
          event: "Join",
          lobby_code: params.lobbyCode,
          username: username,
        }),
      );
    };
    s.onclose = () => {};
    s.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      switch (msg.event) {
        case "SyncState":
          setPlayers(
            new Map(
              msg.data.players.map((p: [string, string, boolean]) => [
                p[0],
                { username: p[1], ready: p[2] },
              ]),
            ),
          );
          setGameSettings(msg.data.settings);
          setScores(new Map(Object.entries(msg.data.leaderboard)));
          setSongState({
            previewUrl: msg.data.preview_url || "",
            roundStartTime: msg.data.round_start_time,
          });
          setGameState(msg.data.status);
          break;
        case "PlayerJoin":
          setPlayers(
            (p) =>
              new Map([
                ...p,
                [
                  msg.data.player_id,
                  { username: msg.data.player_username, ready: false },
                ],
              ]),
          );
          break;
        case "PlayerReady":
          setPlayers(
            (p) =>
              new Map(
                [...p].map((pl) =>
                  pl[0] === msg.data.player_id
                    ? [pl[0], { ...pl[1], ready: true }]
                    : pl,
                ),
              ),
          );
          break;
        case "PlayerUnready":
          setPlayers(
            (p) =>
              new Map(
                [...p].map((pl) =>
                  pl[0] === msg.data.player_id
                    ? [pl[0], { ...pl[1], ready: false }]
                    : pl,
                ),
              ),
          );
          break;
        case "PlayerLeave":
          setPlayers((p) => {
            const newPlayers = new Map(p);
            newPlayers.delete(msg.data.player_id);
            return newPlayers;
          });
          break;
        case "GameSettingsUpdated":
          setGameSettings(msg.data.settings);
          break;
        case "AllReady":
          setGameState("loading");
          break;
        case "GameStart":
          setGameState("playing");
          break;
        case "RoundStart":
          setSongState({
            previewUrl: msg.data.preview_url,
            roundStartTime: msg.data.round_start_time,
          });
          break;
        case "RoundEnd":
          setScores(new Map(Object.entries(msg.data.leaderboard)));
          setSongState({ previewUrl: "", roundStartTime: 0 });
          setChat((c) => [
            ...c,
            {
              user: "",
              message: `The correct song was '${msg.data.correct_title}' by ${msg.data.correct_artists.join(", ")}`,
            },
          ]);
          break;
        case "GameEnd":
          socket.current.send(JSON.stringify({ event: "Unready" }));
          setGameState("finished");
          break;
        case "PlayerGuess":
          setChat((c) => [
            ...c,
            { user: msg.data.username, message: msg.data.content },
          ]);
          break;
        case "CorrectGuess":
          setChat((c) => [...c, { user: "", message: msg.data.msg }]);
          break;
        case "JoinError":
          navigate("/", { state: { error: msg.data.message } });
          break;
        case "PlaylistError":
          setGameState("waiting");
          setError(msg.data.message);
          setPlayerReady(false);
          break;
        default:
          break;
      }
    };
    s.onerror = () => {
      navigate("/", { state: { error: "Failed to connect to server" } });
    };
    return () => {
      s.close();
    };
  }, [params.lobbyCode, navigate, username]);

  const sendGuess = (guess: string) => {
    socket.current.send(JSON.stringify({ event: "Guess", content: guess }));
  };

  const onReady = () => {
    if (playerReady) {
      socket.current.send(JSON.stringify({ event: "Unready" }));
      setPlayerReady(false);
    } else {
      socket.current.send(JSON.stringify({ event: "Ready" }));
      setPlayerReady(true);
    }
  };

  if (gameState === "connecting") {
    return <Connecting />;
  }

  if (gameState === "waiting") {
    return (
      <WaitingRoom
        lobbyCode={params.lobbyCode!}
        onReady={onReady}
        players={players}
        gameSettings={gameSettings}
        updateGameSettings={updateGameSettings}
        error={error}
        setError={setError}
        ready={playerReady}
      />
    );
  }

  if (gameState === "loading") {
    return <PreparingPlaylist />;
  }

  if (gameState === "playing") {
    return (
      <Gameplay
        sendGuess={sendGuess}
        songState={songState}
        players={players}
        chat={chat}
        scores={scores}
      />
    );
  }

  return <Ending resetGame={resetGame} players={players} scores={scores} />;
}
