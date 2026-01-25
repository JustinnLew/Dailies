import { useState, useEffect } from "react";
import PencilIcon from "../icons/PencilIcon";

export default function NavBar() {
  const [username, setUsername] = useState("");

  useEffect(() => {
    const savedUsername = localStorage.getItem("username");
    setUsername(savedUsername || "PLAYER");
  }, []);

  const handleNameChange = (event: React.FormEvent<HTMLDivElement>) => {
    const newName = event.currentTarget.innerText.trim();
    if (newName) {
      setUsername(newName);
      localStorage.setItem("username", newName);
    }
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "Enter") {
      event.preventDefault();
      event.currentTarget.blur();
    }
    if (event.key === "Escape") {
      event.preventDefault();
      event.currentTarget.innerText = username;
      event.currentTarget.blur();
    }
  };

  return (
    <div
      className="w-full h-20 flex items-center justify-between px-8 relative overflow-hidden gap-6"
      style={{
        backgroundColor: "#16213E",
        borderBottom: "4px solid #FF006E",
        boxShadow: "0 4px 0 #8338EC",
      }}
    >
      <h1 className="text-lg font-bold tracking-wider md:text-3xl font-press-start text-neon-yellow text-shadow-(--text-shadow-title)">
        DAILIES
      </h1>

      <div className="flex items-center gap-8">
        <p
          className="text-right text-xs md:text-lg font-press-start text-white outline-none cursor-text"
          contentEditable="true"
          suppressContentEditableWarning
          onBlur={handleNameChange}
          onKeyDown={handleKeyDown}
          style={{
            minWidth: "100px",
            maxWidth: "300px",
            caretShape: "underscore",
          }}
        >
          {username}
        </p>
        <PencilIcon size={18} color={"white"} />
      </div>
    </div>
  );
}
