// ─── AccountSettingsModal – Account-Einstellungen ───
// Drei Bereiche: Passwort ändern, E-Mail ändern, Account löschen.
import { useState, useRef } from 'react';
import Modal from './Modal';

function AccountSettingsModal({ isOpen, onClose, onChangePassword, onChangeEmail, onDeleteAccount }) {
  // useRef für alle Input-Felder
  const oldPwRef = useRef(null);
  const newPwRef = useRef(null);
  const confirmPwRef = useRef(null);
  const newEmailRef = useRef(null);
  const confirmEmailRef = useRef(null);
  const [pwError, setPwError] = useState('');
  const [emailError, setEmailError] = useState('');
  const [pwLoading, setPwLoading] = useState(false);
  const [emailLoading, setEmailLoading] = useState(false);

  // ─── Passwort ändern ───
  async function handlePasswordSubmit(e) {
    e.preventDefault();
    setPwError('');
    if (newPwRef.current.value !== confirmPwRef.current.value) {
      setPwError('Neue Passwörter stimmen nicht überein');
      return;
    }
    setPwLoading(true);
    try {
      await onChangePassword(oldPwRef.current.value, newPwRef.current.value);
      // Nach erfolgreicher Änderung Felder leeren
      oldPwRef.current.value = '';
      newPwRef.current.value = '';
      confirmPwRef.current.value = '';
    } catch (err) {
      setPwError(err.message);
    } finally {
      setPwLoading(false);
    }
  }

  // ─── E-Mail ändern ───
  async function handleEmailSubmit(e) {
    e.preventDefault();
    setEmailError('');
    if (newEmailRef.current.value !== confirmEmailRef.current.value) {
      setEmailError('E-Mail-Adressen stimmen nicht überein');
      return;
    }
    setEmailLoading(true);
    try {
      await onChangeEmail(newEmailRef.current.value);
      newEmailRef.current.value = '';
      confirmEmailRef.current.value = '';
    } catch (err) {
      setEmailError(err.message);
    } finally {
      setEmailLoading(false);
    }
  }

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Account Settings">
      <div className="space-y-6">
        {/* ─── Passwort-Änderungs-Bereich ─── */}
        <div>
          <h3 className="text-sm font-semibold text-gray-300 uppercase tracking-wider mb-3">Change Password</h3>
          <form onSubmit={handlePasswordSubmit} className="space-y-3">
            <input ref={oldPwRef} type="password" required className="w-full bg-gray-800 border border-gray-700  px-4 py-2.5 text-white focus:outline-none focus:ring-2 ac-ring" placeholder="Actual Password" />
            <input ref={newPwRef} type="password" required className="w-full bg-gray-800 border border-gray-700  px-4 py-2.5 text-white focus:outline-none focus:ring-2 ac-ring" placeholder="New Password" />
            <input ref={confirmPwRef} type="password" required className="w-full bg-gray-800 border border-gray-700  px-4 py-2.5 text-white focus:outline-none focus:ring-2 ac-ring" placeholder="Confirm Password" />
            {pwError && <p className="text-red-400 text-sm">{pwError}</p>}
            <button type="submit" disabled={pwLoading} className="bg-orange-600 hover:bg-orange-600 disabled:opacity-50 text-white text-sm px-4 py-2  transition-colors">
              {pwLoading ? '...' : 'Submit'}
            </button>
          </form>
        </div>

        {/* ─── E-Mail-Änderungs-Bereich ─── */}
        <div className="border-t border-gray-800 pt-4">
          <h3 className="text-sm font-semibold text-gray-300 uppercase tracking-wider mb-3">Change Email</h3>
          <form onSubmit={handleEmailSubmit} className="space-y-3">
            <input ref={newEmailRef} type="email" required className="w-full bg-gray-800 border border-gray-700  px-4 py-2.5 text-white focus:outline-none focus:ring-2 ac-ring" placeholder="New Email" />
            <input ref={confirmEmailRef} type="email" required className="w-full bg-gray-800 border border-gray-700  px-4 py-2.5 text-white focus:outline-none focus:ring-2 ac-ring" placeholder="Confirm Email" />
            {emailError && <p className="text-red-400 text-sm">{emailError}</p>}
            <button type="submit" disabled={emailLoading} className="bg-orange-600 hover:bg-orange-600 disabled:opacity-50 text-white text-sm px-4 py-2  transition-colors">
              {emailLoading ? '...' : 'Submit'}
            </button>
          </form>
        </div>

        {/* ─── Account-Lösch-Button (separater Bereich) ─── */}
        <div className="border-t border-gray-800 pt-4">
          <button onClick={onDeleteAccount} className="w-full bg-red-600 hover:bg-red-700 text-white py-2.5  transition-colors">
            Delete Account
          </button>
        </div>
      </div>
    </Modal>
  );
}

export default AccountSettingsModal;
