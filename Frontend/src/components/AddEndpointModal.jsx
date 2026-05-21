// ─── AddEndpointModal – Neuen Endpunkt hinzufügen ───
// Formular mit URL-Eingabe + Intervall (HH:MM:SS).
import { useState, useRef } from 'react';
import Modal from './Modal';

function AddEndpointModal({ isOpen, onClose, onSubmit }) {
  // useRef: Input-Werte ohne Re-Renders (performance-optimiert)
  const urlRef = useRef(null);
  const hourRef = useRef(null);
  const minRef = useRef(null);
  const secRef = useRef(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e) {
    e.preventDefault();
    setError('');
    // Intervall aus Stunden/Minuten/Sekunden zusammenrechnen
    // || 0: Falls das Feld leer ist, 0 verwenden
    const h = parseInt(hourRef.current.value) || 0;
    const m = parseInt(minRef.current.value) || 0;
    const s = parseInt(secRef.current.value) || 30;  // Default: 30 Sekunden
    const total = h * 3600 + m * 60 + s;
    if (total < 1) {
      setError('Bitte mindestens 1 Sekunde eingeben');
      return;
    }
    setLoading(true);
    try {
      await onSubmit(urlRef.current.value, total);
      // Felder leeren nach erfolgreichem Submit
      urlRef.current.value = '';
      hourRef.current.value = '';
      minRef.current.value = '';
      secRef.current.value = '';
      onClose();
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Add New Endpoint">
      <form onSubmit={handleSubmit} className="space-y-4">
        {/* ─── URL-Eingabe ─── */}
        <div>
          <label className="block text-sm text-gray-300 mb-1">URL</label>
          <input ref={urlRef} type="text" required className="w-full bg-gray-800 border border-gray-700  px-4 py-2.5 text-white focus:outline-none focus:ring-2 ac-ring" placeholder="https://api.example.com" />
        </div>
        {/* ─── Intervall-Eingabe (Stunden / Minuten / Sekunden) ─── */}
        <div>
          <label className="block text-sm text-gray-300 mb-2">Set Intervall</label>
          <div className="grid grid-cols-3 gap-3">
            <div>
              <label className="block text-xs text-gray-300 mb-1 text-center">Hour</label>
              <input ref={hourRef} type="number" min="0" className="no-spinner w-full bg-gray-800 border border-gray-700 px-3 py-2 text-white text-center focus:outline-none focus:ring-2 ac-ring" defaultValue="0" />
            </div>
            <div>
              <label className="block text-xs text-gray-300 mb-1 text-center">Minutes</label>
              <input ref={minRef} type="number" min="0" className="no-spinner w-full bg-gray-800 border border-gray-700 px-3 py-2 text-white text-center focus:outline-none focus:ring-2 ac-ring" defaultValue="0" />
            </div>
            <div>
              <label className="block text-xs text-gray-300 mb-1 text-center">Seconds</label>
              <input ref={secRef} type="number" min="0" className="no-spinner w-full bg-gray-800 border border-gray-700 px-3 py-2 text-white text-center focus:outline-none focus:ring-2 ac-ring" defaultValue="30" />
            </div>
          </div>
        </div>
        {error && <p className="text-red-400 text-sm text-center">{error}</p>}
        <button type="submit" disabled={loading} className="w-full ac-bg hover:bg-orange-600 disabled:opacity-50 text-white font-medium py-2.5  transition-colors">
          {loading ? '...' : 'Submit'}
        </button>
      </form>
    </Modal>
  );
}

export default AddEndpointModal;
