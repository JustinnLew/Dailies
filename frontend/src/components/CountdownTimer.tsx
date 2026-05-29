import { useEffect, useState } from "react";
import { motion } from "motion/react";

export default function CountdownTimer({
  roundLengthSeconds,
  roundStartTime,
}: {
  roundLengthSeconds: number;
  roundStartTime: number;
}) {
  const [timeLeft, setTimeLeft] = useState(roundLengthSeconds);

  useEffect(() => {
    if (!roundStartTime) return;

    const localStart = Date.now();
    const durationMs = roundLengthSeconds * 1000;

    const calculateTimeLeft = () => {
      const localNow = Date.now();
      const elapsed = localNow - localStart;
      const difference = durationMs - elapsed;

      if (difference <= 0) {
        setTimeLeft(0);
        return false;
      }
      setTimeLeft(difference / 1000);
      return true;
    };

    calculateTimeLeft();
    const interval = setInterval(() => {
      const isRunning = calculateTimeLeft();
      if (!isRunning) clearInterval(interval);
    }, 50);

    return () => clearInterval(interval);
  }, [roundStartTime, roundLengthSeconds]);

  // Math calculations based on current timeLeft
  const pct = Math.max(0, Math.min(1, timeLeft / roundLengthSeconds));
  const displayTime = Math.ceil(timeLeft);

  const isUrgent = displayTime <= 10;
  const isCritical = displayTime <= 5;

  // SVG arc math
  const R = 18;
  const CIRC = 2 * Math.PI * R;
  const dashOffset = CIRC * (1 - pct);

  const arcColor = isCritical
    ? "#ef4444" // red-500
    : isUrgent
      ? "#eab308" // yellow-500
      : "#22c55e"; // green-500

  return (
    <div className="relative flex items-center justify-center w-14 h-14">
      {/* SVG ring */}
      <svg
        className="absolute inset-0 w-full h-full -rotate-90"
        viewBox="0 0 44 44"
      >
        {/* track */}
        <circle
          cx="22"
          cy="22"
          r={R}
          fill="none"
          stroke="#374151"
          strokeWidth="3"
        />
        {/* progress */}
        <circle
          cx="22"
          cy="22"
          r={R}
          fill="none"
          stroke={arcColor}
          strokeWidth="3"
          strokeLinecap="butt"
          strokeDasharray={CIRC}
          strokeDashoffset={dashOffset}
          style={{ transition: "stroke-dashoffset 0.05s linear, stroke 0.3s" }}
        />
      </svg>

      {/* Number */}
      <motion.span
        key={displayTime} // Triggers animation exactly once per second jump
        initial={{ scale: 1.3, opacity: 0.6 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.15 }}
        className={`relative text-sm font-bold tabular-nums ${
          isCritical
            ? "text-red-400 animate-pulse"
            : isUrgent
              ? "text-yellow-400"
              : "text-green-400"
        }`}
      >
        {displayTime}
      </motion.span>
    </div>
  );
}
