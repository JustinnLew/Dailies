import HomeGridCell from "./HomeGridCell";
import GTSIcon from "../../icons/GTSIcon";
import NotFoundIcon from "../../icons/NotFoundIcon";
import type { Dispatch, SetStateAction } from "react";
import HomeModal from "../../components/modals/HomeModal";
import GeoIcon from "../../icons/GeoIcon";

export default function HomeGrid({
  setError,
}: {
  setError: Dispatch<SetStateAction<string>>;
}) {
  const modes = [
    {
      title: "Guess The Song",
      description:
        "Test your music knowledge by guessing songs from short clips.",
      icon: <GTSIcon color="white" />,
      modal: (open: boolean, onClose: () => void) => (
        <HomeModal open={open} onClose={onClose} setError={setError} game_code={"guess-the-song"} title={"GUESS THE SONG"} />
      ),
    },
    {
      title: "Geo Guessr",
      description:
        "Guess locations around the world.",
      icon: <GeoIcon color="white" />,
      modal: (open: boolean, onClose: () => void) => (
        <HomeModal open={open} onClose={onClose} setError={setError} game_code={"geo-guessr"} title={"GEO GUESSR"} />
      ),
    },
    {
      title: "Coming Soon",
      description: "",
      icon: <NotFoundIcon color="white" />,
    },
  ];

  return (
    <div className="overflow-y-scroll custom-scrollbar pt-16 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-x-16 gap-y-8 p-16 w-full">
      {modes.map((mode, index) => (
        <HomeGridCell
          key={index}
          title={mode.title}
          description={mode.description}
          icon={mode.icon}
          renderModal={mode.modal}
        />
      ))}
    </div>
  );
}
