// ─── CreateAccountModal – Benutzerkonto erstellen ───
// Formular: E-Mail, Passwort, Passwort-Bestätigung.
import { useState, useRef } from 'react';
import Modal from './Modal';

function CreateAccountModal({ isOpen, onClose, onSubmit }) {
  const emailRef = useRef(null);
  const passwordRef = useRef(null);
  const confirmRef = useRef(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e) {
    e.preventDefault();
    setError('');

    // Client-seitige Validierung: Passwörter müssen übereinstimmen
    if (passwordRef.current.value !== confirmRef.current.value) {
      setError('Passwörter stimmen nicht überein');
      return;
    }

    setLoading(true);
    try {
      await onSubmit(emailRef.current.value, passwordRef.current.value);
      // Kein onClose() – bei Erfolg wird der User gesetzt und das Modal geht automatisch zu
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Create Account">
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="block text-sm text-gray-300 mb-1">E-Mail-Adresse</label>
          <input ref={emailRef} type="email" required className="w-full bg-gray-800 border border-gray-700  px-4 py-2.5 text-white focus:outline-none focus:ring-2 ac-ring" placeholder="deine@email.de" />
        </div>
        <div>
          <label className="block text-sm text-gray-300 mb-1">Passwort</label>
          <input ref={passwordRef} type="password" required className="w-full bg-gray-800 border border-gray-700  px-4 py-2.5 text-white focus:outline-none focus:ring-2 ac-ring" placeholder="••••••••" />
        </div>
        <div>
          <label className="block text-sm text-gray-300 mb-1">Passwort bestätigen</label>
          <input ref={confirmRef} type="password" required className="w-full bg-gray-800 border border-gray-700  px-4 py-2.5 text-white focus:outline-none focus:ring-2 ac-ring" placeholder="••••••••" />
        </div>
        {error && <p className="text-red-400 text-sm text-center">{error}</p>}
        <button type="submit" disabled={loading} className="w-full ac-bg hover:bg-orange-600 disabled:opacity-50 text-white font-medium py-2.5  transition-colors">
          {loading ? '...' : 'Submit'}
        </button>
      </form>
    </Modal>
  );
}

export default CreateAccountModal;
