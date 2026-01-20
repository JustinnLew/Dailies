import NavBar from "../components/NavBar.tsx";
import HomeGrid from "../components/HomeGrid.tsx";

export default function Home() {
  return (
    <div className="flex flex-col items-center h-screen scanlines bg-black">
      <NavBar />
      <HomeGrid />
    </div>
  );
}
