export default function Connecting() {
  return (
    <div className="scanlines h-screen flex items-center justify-center bg-black text-white font-press-start text-3xl text-shadow-(--text-shadow-icon)">
      <p className="pr-3">Connecting to lobby</p>
      <div className="wave-container-auto tracking-widest">
        <span>.</span>
        <span>.</span>
        <span>.</span>
      </div>
    </div>
  );
}
