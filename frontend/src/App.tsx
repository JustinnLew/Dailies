import './index.css'
import Home from './routes/Home.tsx'
import { Route, Routes } from 'react-router-dom'
import NotFound from './routes/NotFound.tsx'
import GuessTheSongLanding from './routes/GuessTheSong/Landing.tsx';
import GuessTheSong from './routes/GuessTheSong/GuessTheSongGame.tsx';

function App() {

  return (
    <>
      <Routes>
        <Route path='/' element={<Home/>} />
        <Route path='/guess-the-song' element={<GuessTheSongLanding/>} />
        <Route path='/guess-the-song/lobby/:lobbyCode' element={<GuessTheSong/>} />
        <Route path='/not-found' element={<NotFound/>} />
      </Routes>
    </>
  )
}

export default App
