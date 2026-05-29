import { useEffect, useRef, useState } from "react";
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
  guesses,
}: {
  sendGuess: (guess: string) => void;
  question: Question;
  style: string;
  roundLengthSeconds: number;
  roundStartTime: number;
  currentRound: number;
  totalRounds: number;
  guesses: { username: string; content: string }[];
}) {
  const [shortAnswerInput, setShortAnswerInput] = useState("");
  const [answered, setAnswered] = useState<number | null>(null);
  const navigate = useNavigate();

  return (
    <div className="scanlines h-screen flex flex-col bg-black text-white font-press-start overflow-hidden">
      {/* Header bar: round info + timer */}
      <div className="flex items-center justify-between px-4 py-3 border-b-2 border-neon-pink">
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
        <div className="flex flex-col min-h-0 h-1/2 border-b border-neon-pink">
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
                    setAnswered(index);
                  }}
                  disabled={answered === index}
                  className={`${color.bg} ${color.border} ${color.hover} border-4 rounded-lg p-4 text-sm md:text-base lg:text-lg transition-colors
                  hover:brightness-110 hover:border-white text-center text-white text-shadow-(--text-shadow-icon)`}
                >
                  <p className="text-shadow-(--text-shadow-icon)">{option}</p>
                </button>
              );
            })}
          </div>
        )}

        {style === "Short Answer" && (
          <div className="flex h-1/2">
            {/* Left: input */}
            <div className="flex flex-col items-center justify-center w-4/5 px-6 gap-4">
              <div className="w-full max-w-2xl flex gap-3">
                <input
                  type="text"
                  value={shortAnswerInput}
                  onChange={(e) => setShortAnswerInput(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && shortAnswerInput.trim()) {
                      sendGuess(shortAnswerInput.trim());
                      setShortAnswerInput("");
                    }
                  }}
                  placeholder="TYPE YOUR ANSWER..."
                  className={`
                  flex-1 bg-black border-2 px-4 py-3 font-press-start text-sm
                  outline-none transition-all duration-300
                  border-neon-yellow text-white caret-neon-yellow focus:border-white focus:shadow-[0_0_12px_rgba(255,255,0,0.4)]
                  }
                `}
                />
                <button
                  onClick={() => {
                    if (shortAnswerInput.trim()) {
                      sendGuess(shortAnswerInput.trim());
                      setShortAnswerInput("");
                    }
                  }}
                  className={`
                  px-5 py-3 border-2 font-press-start text-sm transition-all duration-300
                border-neon-yellow text-neon-yellow hover:bg-neon-yellow hover:text-black cursor-pointer
                  }
                `}
                >
                  SUBMIT
                </button>
              </div>
            </div>

            <GuessFeed guesses={guesses} />
          </div>
        )}
      </div>
    </div>
  );
}

function GuessFeed({
  guesses,
}: {
  guesses: { username: string; content: string }[];
}) {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [guesses]);

  return (
    <div className="w-1/2 flex flex-col h-full border-neon-pink">
      <div className="px-4 py-2 border border-t-0 border-neon-pink">
        <span className="text-white text-shadow-(--text-shadow-icon) text-xs font-press-start tracking-widest">
          WRONG ANSWERS
        </span>
      </div>
      <div className="flex-1 overflow-y-auto px-4 py-2 flex flex-col gap-2 scrollbar-none border-l border-neon-pink">
        {guesses.length === 0 ? (
          <p className="text-gray-700 text-xs font-press-start text-center mt-4">
            NO GUESSES YET...
          </p>
        ) : (
          guesses.map((g, i) => (
            <motion.div
              key={i}
              initial={{ opacity: 0, x: 10 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.2 }}
              className="flex gap-2 items-baseline"
            >
              <span className="text-neon-yellow text-md font-press-start shrink-0">
                {g.username}
              </span>
              <span className="text-gray-400 text-md font-vt323 tracking-wide">
                {g.content}
              </span>
            </motion.div>
          ))
        )}
        <div ref={bottomRef} />
      </div>
    </div>
  );
}
