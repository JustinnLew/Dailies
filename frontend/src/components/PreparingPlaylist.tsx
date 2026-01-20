export default function PreparingPlaylist() {
    return (
      <div className="scanlines h-screen flex flex-col gap-6 items-center justify-center bg-black text-white wave-container-auto ">
        <div className="flex text-2xl font-bold font-press-start text-shadow-(--text-shadow-icon)">
            <h2 className="pr-3">Preparing Playlist</h2>
            <span>.</span>
            <span>.</span>
            <span>.</span>
        </div>
        <p className="text-2xl text-gray-400 font-vt323">The game will start in a moment</p>
      </div>
    );
}