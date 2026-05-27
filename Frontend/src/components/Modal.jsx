// ─── Modal-Basis-Komponente ───
// Generischer Overlay-Dialog mit Terminal-Stil.
// Alle spezifischen Modals (AddEndpoint, Settings, Log, etc.) nutzen dieses Grundgeruest.
// Props: isOpen, onClose (schliesst bei Klick auf Hintergrund), title, children

function Modal({ isOpen, onClose, title, children, wide }) {
  // isOpen = false → gar nichts rendern (DOM-frei, kein visibility-Trick)
  if (!isOpen) return null;

  return (
    // Overlay: fixed, volle Fläche, halbtransparenter schwarzer Hintergrund
    // onClick={onClose}: Klick auf Overlay-Hintergrund schliesst das Modal
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70" onClick={onClose}>
      {/* Modal-Container: onClick stoppt Event-Propagation, damit Klick auf Modal-Inhalt nicht schliesst */}
      <div className={`bg-gray-950 border ac-bd w-full mx-4 max-h-[85vh] flex flex-col relative ${wide ? 'max-w-3xl' : 'max-w-md'}`}
        onClick={e => e.stopPropagation()}
        style={{boxShadow: '0 0 40px rgba(234,88,12,0.1), inset 0 0 40px rgba(234,88,12,0.03)'}}>

        {/* ─── Titel-Leiste ─── */}
        <div className="flex items-center justify-between px-5 py-3 border-b ac-bd bg-gray-900/80 shrink-0">
          <div className="flex items-center gap-2">
            {/* Akzent-Punkt (glowing dot) */}
            <span className="w-2 h-2 ac-bg shadow-[0_0_6px_rgba(234,88,12,0.6)]" />
            <h2 className="text-[11px] font-mono font-bold ac-tx-hover uppercase tracking-[0.2em]">{title}</h2>
          </div>
          {/* X-Button zum Schliessen */}
          <button onClick={onClose} className="text-gray-300 hover:text-white text-lg leading-none">&times;</button>
        </div>

        {/* ─── Inhalt (scrollbar bei Ueberlauf) ─── */}
        <div className="p-5 overflow-y-auto">{children}</div>

        {/* ─── Untere Linie (Akzent) ─── */}
        <div className="h-px bg-gradient-to-r from-transparent via-orange-600/30 to-transparent" />
      </div>
    </div>
  );
}

export default Modal;
