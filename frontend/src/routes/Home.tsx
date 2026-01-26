import NavBar from "../components/NavBar.tsx";
import HomeGrid from "../components/HomeGrid.tsx";
import { useState, useEffect } from "react";
import { useLocation } from "react-router-dom";
import ErrorSnackbar from "../components/ErrorSnackbar.tsx";

export default function Home() {
  const [error, setError] = useState("");
  const location = useLocation();

  useEffect(() => {
    if (location.state?.error) {
      setError(location.state.error);
    }
  }, [location]);

  return (
    <div className="flex flex-col items-center h-screen scanlines bg-black">
      <NavBar />
      <HomeGrid setError={setError} />
      <ErrorSnackbar error={error} setError={setError} />
    </div>
  );
}
