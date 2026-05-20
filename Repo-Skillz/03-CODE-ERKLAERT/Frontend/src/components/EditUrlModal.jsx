/* ═══════════════════════════════════════════════════════
 * 📦 EDIT URL MODAL
 *
 * 🎯 ZWEK:
 * URL eines bestehenden Endpoints ändern.
 *
 * 💡 KONZEPTE:
 * - Controlled Component (value + onChange)
 * - Error/Loading-State
 * ═══════════════════════════════════════════════════════ */

import { useState } from 'react';
import Modal from './Modal';

function EditUrlModal({ isOpen, onClose, endpoint, value, onChange, onSave }) {
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState('');

    async function handleSave(e) {
        e.preventDefault();
        setError('');
        setLoading(true);
        try {
            await onSave();
            onClose();
        } catch (err) {
            setError(err.message || 'Fehler beim Speichern');
        } finally {
            setLoading(false);
        }
    }

    return (
        <Modal isOpen={isOpen} onClose={onClose} title="Edit URL">
            <form onSubmit={handleSave} className="space-y-4">
                <div>
                    <label className="block text-xs text-gray-300 mb-1 font-mono tracking-wider">ENDPOINT_URL</label>
                    <input type="text" required value={value} onChange={e => onChange(e.target.value)}
                        className="w-full bg-gray-800 border border-gray-700 px-4 py-2.5 text-white font-mono text-sm focus:outline-none ac-ring" />
                </div>
                {error && <p className="text-red-400 text-xs font-mono border-l-2 border-red-500 pl-2">{error}</p>}
                <button type="submit" disabled={loading}
                    className="w-full ac-bg disabled:opacity-50 text-white font-mono font-bold py-2.5 uppercase tracking-widest text-xs">
                    {loading ? '...' : 'SAVE'}
                </button>
            </form>
        </Modal>
    );
}

export default EditUrlModal;
