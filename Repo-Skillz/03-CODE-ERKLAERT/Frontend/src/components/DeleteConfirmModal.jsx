/* ═══════════════════════════════════════════════════════
 * 📦 DELETE CONFIRM MODAL
 *
 * 🎯 ZWECK:
 * Löschbestätigung mit Deny/Confirm.
 *
 * 💡 KONZEPTE:
 * - Einfachste Modal-Komponente (nur Text + 2 Buttons)
 * - Kein eigener State (nur Props)
 * ═══════════════════════════════════════════════════════ */

import Modal from './Modal';

function DeleteConfirmModal({ isOpen, onClose, onConfirm }) {
    return (
        <Modal isOpen={isOpen} onClose={onClose} title="Delete Confirmation">
            <p className="text-gray-300 mb-6">Bist du sicher, dass du diesen Endpunkt löschen möchtest?</p>
            <div className="flex gap-3">
                <button onClick={onClose} className="flex-1 bg-gray-800 hover:bg-gray-700 text-gray-300 py-2.5 border border-gray-700 transition-colors">
                    Deny
                </button>
                <button onClick={onConfirm} className="flex-1 bg-red-600 hover:bg-red-700 text-white py-2.5 transition-colors">
                    Confirm
                </button>
            </div>
        </Modal>
    );
}

export default DeleteConfirmModal;
