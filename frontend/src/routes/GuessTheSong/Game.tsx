import { useEffect, useState, useRef, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import WaitingRoom from "./WaitingRoom";
import Gameplay from "./Gameplay";
import { getUserId } from "../../utils/util";
import type {
  Player,
  GuessTheSongGameSettings,
  ChatMessage,
  GameState,
} from "../../utils/types";
import Connecting from "../../components/Connecting";
import PreparingPlaylist from "../../components/PreparingPlaylist";
import Ending from "./Ending";

export default function Game() {
  const params = useParams();
  const userId = getUserId();
  const username = localStorage.getItem("username");
  const [players, setPlayers] = useState<Map<string, Player>>(new Map());
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
  };

  useEffect(() => {
    const s = new WebSocket(`ws://localhost:3000/ws/guess-the-song`);
    socket.current = s;

    s.onopen = () => {
      socket.current.send(
        JSON.stringify({
          event: "Join",
          lobby_code: params.lobbyCode,
          user_id: userId,
          username: username,
        }),
      );
    };
    s.onclose = () => {
      // navigate('/');
    };
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
        default:
          break;
      }
    };
    return () => {
      s.close();
      // navigate('/');
    };
  }, [params.lobbyCode, navigate, userId, username]);

  const sendGuess = (guess: string) => {
    socket.current.send(JSON.stringify({ event: "Guess", content: guess }));
  };

  const ready = () => {
    if (players.get(userId)?.ready) {
      socket.current.send(JSON.stringify({ event: "Unready" }));
    } else if (players.get(userId)) {
      socket.current.send(JSON.stringify({ event: "Ready" }));
    }
  };

  if (gameState === "connecting") {
    return <Connecting />;
  }

  if (gameState === "waiting") {
    return (
      <WaitingRoom
        lobbyCode={params.lobbyCode!}
        ready={ready}
        players={players}
        gameSettings={gameSettings}
        updateGameSettings={updateGameSettings}
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
