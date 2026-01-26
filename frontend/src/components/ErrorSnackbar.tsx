import Snackbar, {
  type SnackbarCloseReason,
  type SnackbarOrigin,
} from "@mui/material/Snackbar";
import {
  useEffect,
  useState,
  type CSSProperties,
  type Dispatch,
  type SetStateAction,
  type SyntheticEvent,
} from "react";

export default function ErrorSnackbar({
  error,
  setError,
  anchorOrigin,
  style,
}: {
  error: string;
  setError: Dispatch<SetStateAction<string>>;
  anchorOrigin?: SnackbarOrigin;
  style?: CSSProperties;
}) {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (error !== "") {
      setOpen(true);
    } else {
      setOpen(false);
    }
  }, [error]);

  const handleExited = () => {
    setError("");
  };

  const handleClose = (
    _?: SyntheticEvent | Event,
    reason?: SnackbarCloseReason,
  ) => {
    if (reason === "clickaway") {
      return;
    }

    setOpen(false);
  };

  return (
    <Snackbar
      key={error}
      anchorOrigin={anchorOrigin || { vertical: "bottom", horizontal: "right" }}
      autoHideDuration={10000}
      onClose={handleClose}
      open={open}
      slotProps={{
        transition: {
          onExited: handleExited,
        },
      }}
      style={style || {}}
    >
      <div
        className="max-w-4/5 text-end bg-black font-bold scanlines flex gap-3 font-press-start text-red-500 border-white border-2 p-4 text-sm md:text-md"
        style={{
          clipPath:
            "polygon(0 6px, 6px 6px, 6px 0, calc(100% - 6px) 0, calc(100% - 6px) 6px, 100% 6px, 100% calc(100% - 6px), calc(100% - 6px) calc(100% - 6px), calc(100% - 6px) 100%, 6px 100%, 6px calc(100% - 6px), 0 calc(100% - 6px))",
        }}
      >
        <span className="animate-pulse">&gt;</span>
        <p>{error}</p>
        <button
          onClick={handleClose}
          className="ml-4 hover:text-white cursor-pointer"
        >
          X
        </button>
      </div>
    </Snackbar>
  );
}
