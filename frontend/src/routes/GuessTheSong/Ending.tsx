import { useNavigate } from "react-router-dom";
import type { Player } from "../../utils/types";

export default function Ending({
  players,
  scores,
}: {
  players: Map<string, Player>;
  scores: Map<string, number>;
}) {
  let leaderboard = Array.from(players.entries())
    .map(([id, player]) => ({
      id,
      username: player.username,
      score: scores.get(id) || 0,
    }))
    .sort((a, b) => b.score - a.score);

  const navigate = useNavigate();

  return (
    <div className="h-screen overflow-y-auto custom-scrollbar flex pt-12 justify-center bg-black">
      <div className="max-w-4xl p-4 w-full flex flex-col gap-8">
        {/* Title */}
        <div className="text-center mb-6">
          <h1 className="text-4xl font-bold mb-4 font-press-start text-neon-yellow text-shadow-(--text-shadow-title)">
            GAME OVER
          </h1>
          <p className="font-vt323 text-neon-yellow text-3xl">Final Results</p>
        </div>

        {/* Top 3 */}
        <div className="flex items-end justify-center gap-4 mb-2 min-h-48">
          {/* 2nd Place */}
          <div className="w-1/6 h-4/5">
            {/* <Trophy place={2} /> */}
            {leaderboard[1] && (
              <div
                className="flex flex-col gap-2 h-full w-full text-center justify-center border-4 border-gray-400"
                style={{
                  clipPath:
                    "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
                }}
              >
                <div className="font-press-start text-gray-400">2ND</div>
                <div className="font-press-start text-white text-sm">
                  {leaderboard[1].username}
                </div>
                <div className="font-vt323 text-2xl text-neon-yellow">
                  {leaderboard[1].score}
                </div>
              </div>
            )}
          </div>

          {/* 1st Place */}
          <div className="w-1/5 h-full">
            {/* <Crown /> */}
            {/* <Trophy place={1} /> */}
            <div
              className="flex flex-col gap-4 h-full w-full text-center justify-center border-4 border-yellow-500"
              style={{
                clipPath:
                  "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
              }}
            >
              <div className="font-press-start text-neon-yellow">WINNER</div>
              <div className="font-press-start text-sm text-white">
                {leaderboard[0].username}
              </div>
              <div className="font-vt323 text-2xl text-neon-yellow">
                {leaderboard[0].score}
              </div>
            </div>
          </div>

          {/* 3rd Place */}
          <div className="w-1/6 h-4/6">
            {/* <Trophy place={3} /> */}
            {leaderboard[2] && (
              <div
                className="flex flex-col gap-1 h-full w-full text-center justify-center border-4 border-yellow-700"
                style={{
                  clipPath:
                    "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
                }}
              >
                <div className="font-press-start text-yellow-700 text-xs">
                  3RD
                </div>
                <div className="font-press-start text-xs text-white">
                  {leaderboard[2].username}
                </div>
                <div className="font-vt323 text-2xl text-neon-yellow">
                  {leaderboard[2].score}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Full Leaderboard */}
        <div
          className="mb-8 p-6 bg-neon-blue/20 border-4 border-neon-blue"
          style={{
            clipPath:
              "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
          }}
        >
          <h3 className="text-center mb-4 font-press-start text-neon-yellow text-shadow-(--text-shadow-icon)">
            &gt; FULL STANDINGS &lt;
          </h3>
          <div className="space-y-2">
            {leaderboard.map((player, i) => (
              <div
                key={i}
                className={`font-vt323 text-2xl flex items-center gap-4 p-3 ${i < 3 ? "bg-neon-pink/20" : "bg-neon-card"}
                            border-2 ${i < 3 ? "border-neon-pink" : "border-neon-blue"}`}
              >
                <div className="w-12 text-center text-neon-yellow">
                  #{i + 1}
                </div>
                <div className="flex-1 text-white">{player.username}</div>
                <div className="text-right text-neon-yellow">
                  {player.score} pts
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex flex-col sm:flex-row gap-4 justify-center pb-10">
          <button
            className="font-press-start text-white px-8 py-4 transition-all hover:scale-105 bg-neon-pink border-4 border-neon-blue"
            style={{
              clipPath:
                "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
            }}
          >
            PLAY AGAIN
          </button>
          <button
            className="font-press-start px-8 py-4 transition-all hover:scale-105 border-4 border-neon-blue text-neon-yellow"
            onClick={() => navigate("/")}
            style={{
              clipPath:
                "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
            }}
          >
            EXIT
          </button>
        </div>
      </div>
    </div>
  );
}
