import './index.css'
import Home from './routes/Home.tsx'
import { Route, Routes } from 'react-router-dom'
import NotFound from './routes/NotFound.tsx'

function App() {

  return (
    <>
      <Routes>
        <Route path='/' element={<Home/>} />
        <Route path='/guess-the-song' element={<div>Guess The Song Game</div>} />
        <Route path='/not-found' element={<NotFound/>} />
      </Routes>
    </>
  )
}

export default App
