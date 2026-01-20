export default function NavBar() {
  return (
    <div
      className="w-full h-20 flex items-center px-8 relative overflow-hidden"
      style={{
        backgroundColor: "#16213E",
        borderBottom: "4px solid #FF006E",
        boxShadow: "0 4px 0 #8338EC",
      }}
    >
      <h1 className="font-bold tracking-wider text-3xl font-press-start text-neon-yellow text-shadow-(--text-shadow-title)">
        DAILIES
      </h1>
    </div>
  );
}
