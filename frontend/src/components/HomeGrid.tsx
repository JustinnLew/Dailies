import HomeGridCell from "./HomeGridCell";

export default function HomeGrid() {
  const modes = [
    {
      title: "Guess The Song",
      description:
        "Test your music knowledge by guessing songs from short clips.",
      to: "/guess-the-song",
    },
    { title: "Coming Soon", description: "Maybe...", to: "/not-found" },
  ];

  return (
    <div className="pt-16 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-x-16 gap-y-8 p-16 w-full">
      {modes.map((mode, index) => (
        <HomeGridCell
          key={index}
          title={mode.title}
          description={mode.description}
          to={mode.to}
        />
      ))}
    </div>
  );
}
