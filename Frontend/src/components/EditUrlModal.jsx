// ─── EditUrlModal – URL + Check-Typ eines Endpunkts bearbeiten ───
import { useState, useEffect } from 'react';
import Modal from './Modal';

function EditUrlModal({ isOpen, onClose, endpoint, value, onChange, onSave }) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [checkType, setCheckType] = useState(endpoint?.checkType || 'http');

  useEffect(() => {
    setCheckType(endpoint?.checkType || 'http');
  }, [endpoint]);

  async function handleSave(e) {
    e.preventDefault();
    setError('');
    setLoading(true);
    try {
      await onSave(checkType);
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
        <div>
          <label className="block text-xs text-gray-300 mb-1 font-mono tracking-wider">CHECK_TYPE</label>
          <div className="flex gap-4">
            <label className="flex items-center gap-2 cursor-pointer">
              <input type="radio" name="editCheckType" value="http" checked={checkType === 'http'} onChange={e => setCheckType(e.target.value)} className="accent-orange-500" />
              <span className="text-sm text-gray-200">HTTP</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input type="radio" name="editCheckType" value="icmp" checked={checkType === 'icmp'} onChange={e => setCheckType(e.target.value)} className="accent-orange-500" />
              <span className="text-sm text-gray-200">ICMP</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input type="radio" name="editCheckType" value="tcp" checked={checkType === 'tcp'} onChange={e => setCheckType(e.target.value)} className="accent-orange-500" />
              <span className="text-sm text-gray-200">TCP (Port-Check)</span>
            </label>
          </div>
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
