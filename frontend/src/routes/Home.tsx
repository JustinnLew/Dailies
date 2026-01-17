import NavBar from '../components/NavBar.tsx';
import HomeGrid from '../components/HomeGrid.tsx';

export default function Home() {
	return (
		<div className="flex flex-col items-center min-h-screen bg-slate-950 text-white">
		<NavBar />
		<HomeGrid />
		</div>
	);
}
