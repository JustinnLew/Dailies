import "./index.css";
import Home from "./routes/Home.tsx";
import { Route, Routes } from "react-router-dom";
import GuessTheSong from "./routes/GuessTheSong/Game.tsx";

function App() {
  return (
    <>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/guess-the-song/:lobbyCode" element={<GuessTheSong />} />
      </Routes>
    </>
  );
}

export default App;
