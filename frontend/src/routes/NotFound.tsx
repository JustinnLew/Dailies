import { Link } from "react-router-dom";
import Navbar from "../components/NavBar";

export default function NotFound() {
  return (
    <div className="h-screen flex flex-col bg-gray-900 text-white selection:bg-emerald-500/30">
      <Navbar />

      <div className="flex-1 flex flex-col items-center justify-center px-4 text-center">
        <div className="mb-8 relative">
          <div className="absolute inset-0 bg-emerald-500 blur-3xl opacity-20 animate-pulse"></div>
          <div className="relative bg-gray-800 border border-gray-700 p-8 rounded-3xl shadow-2xl">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-16 w-16 text-emerald-500 animate-bounce"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"
              />
            </svg>
          </div>
        </div>

        <div className="relative z-10">
          <span className="text-emerald-500 font-mono text-sm tracking-widest uppercase mb-2 block">
            New Game Mode
          </span>
          <h2 className="text-4xl md:text-5xl font-bold mb-4 tracking-tight">
            Tuning the Strings...
          </h2>
          <p className="text-gray-400 text-lg mb-8 max-w-sm mx-auto leading-relaxed">
            We're currently building something special here. This game mode will
            be ready to play very soon!
          </p>

          <Link
            to="/"
            className="inline-flex items-center gap-2 bg-gray-800 hover:bg-gray-700 text-white border border-gray-700 font-semibold py-3 px-8 rounded-xl transition-all"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-5 w-5"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fillRule="evenodd"
                d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z"
                clipRule="evenodd"
              />
            </svg>
            Back to Home
          </Link>
        </div>
      </div>
    </div>
  );
}
