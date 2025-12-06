import NavBar from '../components/NavBar.tsx';
import HomeGrid from '../components/HomeGrid.tsx';

export default function Home() {
  return (
    <div className="flex flex-col items-center min-h-screen bg-gray-100">
      <NavBar />
      <HomeGrid />
    </div>
  );
}
