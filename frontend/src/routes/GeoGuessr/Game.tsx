import { useEffect, useState, useRef, useCallback } from "react";
import { useParams, useNavigate } from "react-router-dom";
import.meta.env;
import type {
  Player,
  GameState,
  GeoGuessrGameSettings,
} from "../../utils/types";
import Connecting from "../../components/loading/Connecting";
import { WS_URL } from "../../apiConfig";
import Waiting from "./WaitingRoom";
import GameLoading from "../../components/loading/GameLoading";
import Ending from "../../components/Ending";
import Gameplay from "./Gameplay";

export default function Game() {
  const params = useParams();
  const [playerReady, setPlayerReady] = useState(false);
  const username = localStorage.getItem("username") || "PLAYER";
  const [players, setPlayers] = useState<Map<string, Player>>(new Map());
  const [error, setError] = useState<string>("");
  const [gameSettings, setGameSettings] = useState<GeoGuessrGameSettings>({
    numRounds: 10,
    roundLengthSeconds: 30,
    roundDelaySeconds: 3,
  });
  const navigate = useNavigate();
  const socket = useRef<WebSocket>(null!);
  const updateGameSettings = useCallback((settings: GeoGuessrGameSettings) => {
    setGameSettings(settings);

    if (socket.current && socket.current.readyState === WebSocket.OPEN) {
      socket.current.send(
        JSON.stringify({
          type: "GameEvent",
          data: {
            event: "UpdateGameSettings",
            settings: settings,
          },
        }),
      );
    }
  }, []);
  const [gameState, setGameState] = useState<GameState>("connecting");
  const [scores, setScores] = useState<Map<string, number>>(new Map());
  const [guesses, setGuesses] = useState<Map<string, [number, number]>>(new Map());
  const [imageId, setImageId] = useState<string>("");

  const resetGame = () => {
    setGameState("waiting");
    setError("");
    setScores(new Map([...scores.keys()].map((key) => [key, 0])));
  };

  useEffect(() => {
    const s = new WebSocket(`${WS_URL}/geo-guessr`);
    socket.current = s;

    s.onopen = () => {
      socket.current.send(
        JSON.stringify({
          type: "LobbyEvent",
          data: {
            event: "Join",
            lobby_code: params.lobbyCode,
            username: username,
          },
        }),
      );
    };
    s.onclose = () => {};
    s.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      console.log(msg.data);
      switch (msg.data.event) {
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
          setImageId(msg.data.image_id);
          break;
        case "RoundEnd":
          setScores(new Map(Object.entries(msg.data.leaderboard)));
          setGuesses(new Map(Object.entries(msg.data.guesses)));
          break;
        case "GameEnd":
          socket.current.send(
            JSON.stringify({ type: "LobbyEvent", data: { event: "Unready" } }),
          );
          setGameState("finished");
          break;
        case "JoinError":
          navigate("/", { state: { error: msg.data.message } });
          break;
        case "LoadingErrorj":
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

  const sendGuess = (guess: [number, number]) => {
    socket.current.send(
      JSON.stringify({
        type: "GameEvent",
        data: {
          event: "Guess",
          lat: guess[0],
          lng: guess[1],
        },
      }),
    );
  };

  const onReady = () => {
    if (playerReady) {
      socket.current.send(
        JSON.stringify({ type: "LobbyEvent", data: { event: "Unready" } }),
      );
      setPlayerReady(false);
    } else {
      socket.current.send(
        JSON.stringify({ type: "LobbyEvent", data: { event: "Ready" } }),
      );
      setPlayerReady(true);
    }
  };

  if (gameState === "connecting") {
    return <Connecting />;
  }

  if (gameState === "waiting") {
    return (
      <Waiting
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
    return <GameLoading text={"Loading map"} />;
  }

  if (gameState === "playing") {
    return <Gameplay imageId={imageId} sendGuess={sendGuess} />;
  }

  return <Ending resetGame={resetGame} players={players} scores={scores} />;
}
