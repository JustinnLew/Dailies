import { useState } from "react";
import { motion } from "motion/react";
import { useNavigate } from "react-router-dom";
import CountdownTimer from "../../components/CountdownTimer";

type Question = {
  question: string;
  options: string[] | undefined;
};

const MC_COLORS = [
  { bg: "bg-red-600", border: "border-red-400", hover: "hover:bg-red-500" },
  { bg: "bg-blue-600", border: "border-blue-400", hover: "hover:bg-blue-500" },
  {
    bg: "bg-yellow-600",
    border: "border-yellow-400",
    hover: "hover:bg-yellow-500",
  },
  {
    bg: "bg-green-600",
    border: "border-green-400",
    hover: "hover:bg-green-500",
  },
];

export default function Gameplay({
  sendGuess,
  question,
  style,
  roundLengthSeconds,
  roundStartTime,
  currentRound,
  totalRounds,
}: {
  sendGuess: (guess: string) => void;
  question: Question;
  style: string;
  roundLengthSeconds: number;
  roundStartTime: number;
  currentRound: number;
  totalRounds: number;
}) {
  const [shortAnswerInput, setShortAnswerInput] = useState("");
  const navigate = useNavigate();

  return (
    <div className="scanlines h-screen flex flex-col bg-black text-white font-press-start overflow-hidden">
      {/* Header bar: round info + timer */}
      <div className="flex items-center justify-between px-4 py-3 border-b-2 border-gray-800">
        <div className="flex items-center gap-3">
          <span className="text-neon-yellow text-shadow-(--text-shadow-icon) text-md">
            ROUND {currentRound}/{totalRounds}
          </span>
        </div>

        <CountdownTimer
          roundLengthSeconds={roundLengthSeconds}
          roundStartTime={roundStartTime}
        />

        <button
          onClick={() => navigate("/")}
          className="text-md px-3 py-1 border-red-500 border-2 text-red-400 cursor-pointer hover:bg-red-500/20 transition-colors"
        >
          EXIT
        </button>
      </div>

      <div className="flex-1 flex flex-col w-full">
        {/* Question area */}
        <div className="flex flex-col min-h-0 h-1/2">
          <motion.div
            key={question.question}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.4 }}
            className="flex items-center justify-center px-6 py-8 flex-1"
          >
            <div className="max-w-3xl w-full text-center">
              <h1 className="text-base md:text-xl lg:text-2xl leading-relaxed font-vt323 tracking-wider text-white">
                {question.question}
              </h1>
            </div>
          </motion.div>
        </div>

        {style === "Multiple Choice" && (
          <div className="grid grid-cols-2 grid-rows-2 gap-4 p-4 h-1/2">
            {question.options?.map((option, index) => {
              const color = MC_COLORS[index % MC_COLORS.length];
              return (
                <button
                  key={option}
                  onClick={() => {
                    sendGuess(option);
                  }}
                  className={`${color.bg} ${color.border} ${color.hover} border-4 rounded-lg p-4 text-sm md:text-base lg:text-lg transition-colors
                  hover:brightness-110 hover:border-white text-center text-white text-shadow-(--text-shadow-icon)`}
                >
                  <p className="text-shadow-(--text-shadow-icon)">{option}</p>
                </button>
              );
            })}
          </div>
        )}

        {style === "Short Answer" && <div>{/* TODO */}</div>}
      </div>
    </div>
  );
}
