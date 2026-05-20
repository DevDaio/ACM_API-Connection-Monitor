/* ═══════════════════════════════════════════════════════
 * 📦 MODAL-WRAPPER (Wiederverwendbares Overlay)
 *
 * 🎯 ZWECK:
 * Gemeinsames Layout für alle Modals:
 * - Overlay (klick zum Schließen)
 * - Top-Bar mit Titel
 * - Content-Bereich (children)
 * - Bottom-Accent-Linie
 *
 * 💡 KONZEPTE:
 * - Conditional Rendering: `if (!isOpen) return null`
 * - Event Propagation stoppen: `e.stopPropagation()`
 * - Children-Prop für flexiblen Inhalt
 *
 * ⚠️ WICHTIG:
 * Klick auf Overlay → schließt Modal
 * Klick auf Modal-Inhalt → stoppt Propagation
 * ═══════════════════════════════════════════════════════ */

function Modal({ isOpen, onClose, title, children }) {
    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70" onClick={onClose}>
            <div className="bg-gray-950 border ac-bd w-full max-w-md mx-4 max-h-[85vh] flex flex-col relative"
                onClick={e => e.stopPropagation()}
                style={{boxShadow: '0 0 40px rgba(234,88,12,0.1), inset 0 0 40px rgba(234,88,12,0.03)'}}>
                <div className="flex items-center justify-between px-5 py-3 border-b ac-bd bg-gray-900/80 shrink-0">
                    <div className="flex items-center gap-2">
                        <span className="w-2 h-2 ac-bg shadow-[0_0_6px_rgba(234,88,12,0.6)]" />
                        <h2 className="text-[11px] font-mono font-bold ac-tx-hover uppercase tracking-[0.2em]">{title}</h2>
                    </div>
                    <button onClick={onClose} className="text-gray-300 hover:text-white text-lg leading-none">&times;</button>
                </div>
                <div className="p-5 overflow-y-auto">{children}</div>
                <div className="h-px bg-gradient-to-r from-transparent via-orange-600/30 to-transparent" />
            </div>
        </div>
    );
}

export default Modal;
