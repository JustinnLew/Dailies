import { useState, type JSX } from "react";

export default function HomeGridCell({
  title,
  description,
  icon,
  renderModal,
}: {
  title: string;
  description: string;
  icon: JSX.Element;
  renderModal?: (open: boolean, onClose: () => void) => JSX.Element;
}) {
  const [modalOpen, setModalOpen] = useState(false);
  const handleOpen = () => setModalOpen(true);
  const handleClose = () => setModalOpen(false);

  return (
    <>
      <div
        onClick={handleOpen}
        className="p-6 transition-all hover:scale-105 cursor-pointer flex flex-col bg-neon-bg border-6 border-neon-pink border-solid"
        style={{
          boxShadow: "8px 8px 0 oklch(0.60 0.30 240)",
          clipPath:
            "polygon(0 8px, 8px 8px, 8px 0, calc(100% - 8px) 0, calc(100% - 8px) 8px, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 8px calc(100% - 8px), 0 calc(100% - 8px))",
        }}
      >
        <div className="flex items-center gap-4 mb-4">
          <div>{icon}</div>
          <h2 className="text-xl font-bold font-press-start text-white text-shadow-(--text-shadow-icon)">
            {title}
          </h2>
        </div>
        <p className="font-vt323 text-neon-yellow text-xl leading-tight">
          {description}
        </p>
        <div className="mt-auto pt-6 text-right font-bold text-sm wave-container font-press-start text-red-500">
          <span>&gt;</span>
          <span>&gt;</span>
          <span>&gt;</span>
        </div>
      </div>
      {renderModal && renderModal(modalOpen, handleClose)}
    </>
  );
}
