import "./index.css";
import Home from "./routes/Home/Home.tsx";
import { Route, Routes } from "react-router-dom";
import GuessTheSong from "./routes/GuessTheSong/Game.tsx";
import GeoGuessr from "./routes/GeoGuessr/Game.tsx";
import AudioProvider from "./components/audio/AudioProvider.tsx";

function App() {
  return (
    <>
      <Routes>
        <Route path="/" element={<Home />} />
        <Route
          path="/guess-the-song/:lobbyCode"
          element={
            <AudioProvider>
              <GuessTheSong />
            </AudioProvider>
          }
        />
        <Route
          path="/geo-guessr/:lobbyCode"
          element={
            <GeoGuessr />
          }
        />
      </Routes>
    </>
  );
}

export default App;
