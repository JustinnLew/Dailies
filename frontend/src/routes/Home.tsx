import NavBar from "../components/NavBar.tsx";
import HomeGrid from "../components/HomeGrid.tsx";
import Snackbar, { type SnackbarCloseReason } from "@mui/material/Snackbar";
import { useState, useEffect } from "react";
import { useLocation } from "react-router-dom";

export default function Home() {
  const [open, setOpen] = useState(false);
  const [errorMessage, setErrorMessage] = useState("");
  const location = useLocation();

  useEffect(() => {
    if (location.state?.error) {
      setErrorMessage(location.state.error);
    }
  }, [location]);

  useEffect(() => {
    if (errorMessage != "") {
      setOpen(true);
    }
  }, [errorMessage]);

  const handleClose = (
    _?: React.SyntheticEvent | Event,
    reason?: SnackbarCloseReason,
  ) => {
    if (reason === "clickaway") {
      return;
    }

    setOpen(false);
  };

  const handleExited = () => {
    setErrorMessage("");
    window.history.replaceState({}, document.title);
  };

  return (
    <div className="flex flex-col items-center h-screen scanlines bg-black">
      <NavBar />
      <HomeGrid setErrorMessage={setErrorMessage} />
      <Snackbar
        anchorOrigin={{
          vertical: "bottom",
          horizontal: "right",
        }}
        autoHideDuration={10000}
        onClose={handleClose}
        open={open}
        slotProps={{
          transition: {
            onExited: handleExited,
          },
        }}
      >
        <div
          className="font-bold scanlines flex gap-3 font-press-start text-red-500 border-white border-2 p-4 text-sm md:text-lg"
          style={{
            clipPath:
              "polygon(0 6px, 6px 6px, 6px 0, calc(100% - 6px) 0, calc(100% - 6px) 6px, 100% 6px, 100% calc(100% - 6px), calc(100% - 6px) calc(100% - 6px), calc(100% - 6px) 100%, 6px 100%, 6px calc(100% - 6px), 0 calc(100% - 6px))",
          }}
        >
          <span className="animate-pulse">&gt;</span>
          {errorMessage}
          <button
            onClick={handleClose}
            className="ml-4 hover:text-white cursor-pointer"
          >
            X
          </button>
        </div>
      </Snackbar>
    </div>
  );
}
