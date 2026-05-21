// ─── Haupt-App-Komponente ───
// Verwaltet den gesamten App-Status: User-Auth, Endpunkte, Modal-Zustaende.
import { useState, useEffect, useRef } from 'react';
import LandingPage from './components/LandingPage';
import Dashboard from './components/Dashboard';
import CreateAccountModal from './components/CreateAccountModal';
import AddEndpointModal from './components/AddEndpointModal';
import SetIntervallModal from './components/SetIntervallModal';
import DeleteConfirmModal from './components/DeleteConfirmModal';
import AccountSettingsModal from './components/AccountSettingsModal';
import LogModal from './components/LogModal';
import EditUrlModal from './components/EditUrlModal';

import { ThemeProvider } from './ThemeContext';
import { api } from './api';
import './App.css';

// ─── URL-Normalisierung ───
// Stellt sicher, dass jede URL ein gültiges Protokoll (http:// oder https://) hat.
// Der User kann "example.com" oder "192.168.1.1:8080" eingeben – die Funktion ergänzt https:// oder http://.
function normalizeUrl(raw) {
  if (/^[a-zA-Z][a-zA-Z0-9+\-.]*:\/\//.test(raw)) return raw;
  if (/^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}(:\d+)?(\/.*)?$/.test(raw)) return `http://${raw}`;
  if (/^\[[\da-fA-F:]+\](:\d+)?(\/.*)?$/.test(raw)) return `http://${raw}`;
  const ipv6 = raw.match(/^([\da-fA-F:]+):(\d{2,5})(\/.*)?$/);
  if (ipv6) return `http://[${ipv6[1]}]:${ipv6[2]}${ipv6[3]||''}`;
  if (/^[\da-fA-F:]+$/.test(raw) && raw.includes(':')) return `http://[${raw}]`;
  if (/^localhost(:\d+)?(\/.*)?$/i.test(raw)) return `http://${raw}`;
  return `https://${raw}`;
}

function App() {
  // ─── State ───
  // User-State: wird aus localStorage initialisiert (Session-Persistenz)
  const [user, setUser] = useState(() => {
    const saved = localStorage.getItem('acm_user');
    return saved ? JSON.parse(saved) : null;
  });
  const [endpoints, setEndpoints] = useState([]);
  const [mainSwitch, setMainSwitch] = useState(true);  // Globaler Toggle (alle Endpunkte ON/OFF)

  // Modal-Controls
  const [showCreateAccount, setShowCreateAccount] = useState(false);
  const [showAddEndpoint, setShowAddEndpoint] = useState(false);
  const [showSetIntervall, setShowSetIntervall] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [showAccountSettings, setShowAccountSettings] = useState(false);
  const [showLog, setShowLog] = useState(false);
  const [deleteIndex, setDeleteIndex] = useState(null);
  const [selectedEndpoint, setSelectedEndpoint] = useState(null);
  const [logEntries, setLogEntries] = useState([]);
  const [showEditUrl, setShowEditUrl] = useState(false);
  const [editUrlValue, setEditUrlValue] = useState('');

  // ─── User in localStorage persistieren ───
  useEffect(() => {
    if (user) {
      localStorage.setItem('acm_user', JSON.stringify(user));
    } else {
      localStorage.removeItem('acm_user');
    }
  }, [user]);

  // ─── Refs für Modal-Pause + Poll-Interval-Cleanup ───
  // anyModalRef: Während ein Modal offen ist, wird das Polling pausiert
  // pollRef: Speichert die pollUntilReady-Interval-ID fuer Cleanup bei Unmount
  const anyModalOpen =
    showCreateAccount || showAddEndpoint || showSetIntervall ||
    showDeleteConfirm || showAccountSettings || showLog;
  const anyModalRef = useRef(anyModalOpen);
  const pollRef = useRef(null);
  useEffect(() => { anyModalRef.current = anyModalOpen; }, [anyModalOpen]);
  useEffect(() => { return () => { if (pollRef.current) clearInterval(pollRef.current); }; }, []);

  // ─── Formatierungs-Hilfsfunktionen ───

  // Dauer/Uptime formatieren: Sekunden → "Xd HH:MM:SS" oder "XXmo Yd HH:MM"
  function fmtDuration(secs) {
    if (secs == null) return '--';
    const d = Math.floor(secs / 86400);
    const h = Math.floor((secs % 86400) / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const s = secs % 60;
    if (d >= 30) {
      const months = Math.floor(d / 30);
      const days = d % 30;
      return `${months}mo ${days}d ${String(h).padStart(2,'0')}:${String(m).padStart(2,'0')}`;
    }
    if (d > 0) return `${d}d ${String(h).padStart(2,'0')}:${String(m).padStart(2,'0')}:${String(s).padStart(2,'0')}`;
    return `${String(h).padStart(2,'0')}:${String(m).padStart(2,'0')}:${String(s).padStart(2,'0')}`;
  }

  // Intervall formatieren: Sekunden → "1h 30m 0s"
  function fmtInterval(secs) {
    if (secs == null) return '--';
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const s = secs % 60;
    const parts = [];
    if (h > 0) parts.push(`${h}h`);
    if (m > 0) parts.push(`${m}m`);
    parts.push(`${s}s`);
    return parts.join(' ');
  }

  // API-Daten in Dashboard-kompatibles Format mappen
  function mapEndpoints(data) {
    return data.map(ep => ({
      endpointid: ep.endpointid,
      url: ep.url,
      active: true,
      status: ep.status === null ? 'Unknown' : (ep.status ? 'Running' : 'Down'),
      durationSeconds: ep.duration_seconds ?? 0,
      interval: fmtInterval(ep.interval_seconds),
      sparkHistory: ep.status_history.length ? ep.status_history : [true, true],
      changedate: ep.statusdate || '--',
      changetime: ep.statustime ? ep.statustime.split('.').shift() : '--',
    }));
  }

  // ─── Endpunkte frisch von der API laden ───
  async function refreshEndpoints() {
    try {
      const d = await api.getHome(user.userid);
      setEndpoints(mapEndpoints(d));
    } catch (e) {
      console.error('refreshEndpoints failed', e);
    }
  }

  // ─── Polling nach Mutationen (bis max 16 s) ───
  // Wartet auf den ersten echten Monitoring-Status nach add/edit/delete.
  // pollRef speichert die Interval-ID → useEffect-Cleanup verhindert Lecks bei Unmount.
  function pollUntilReady() {
    if (pollRef.current) clearInterval(pollRef.current);
    let tries = 0;
    pollRef.current = setInterval(async () => {
      tries++;
      await refreshEndpoints();
      if (tries >= 8) { clearInterval(pollRef.current); pollRef.current = null; }
    }, 2000);
  }

  // ─── Initiale Endpunkte laden ───
  useEffect(() => {
    if (!user) return;
    refreshEndpoints();
  }, [user]);

  // ─── Polling: sekündliches Uptime-Updating + 10s API-Poll ───
  useEffect(() => {
    if (!user) return;
    const tick = setInterval(() => {
      // Lokaler Uptime-Counter (ohne API-Aufruf)
      setEndpoints(prev => prev.map(ep =>
        ep.durationSeconds != null ? { ...ep, durationSeconds: ep.durationSeconds + 1 } : ep
      ));
    }, 1000);
    const poll = setInterval(() => {
      // API-Poll nur, wenn kein Modal offen ist (anyModalRef = aktueller Wert ohne Re-render)
      if (anyModalRef.current) return;
      api.getHome(user.userid).then(d => setEndpoints(mapEndpoints(d))).catch(() => {});
    }, 10000);
    return () => { clearInterval(tick); clearInterval(poll); };
  }, [user]);

  // ════════════════════════════════════════════════════════════════
  //  Event-Handler (werden an Child-Komponenten weitergereicht)
  // ════════════════════════════════════════════════════════════════

  // Login: user setzen (triggert Endpunkt-Laden via useEffect)
  async function handleLogin(email, password) {
    const data = await api.login(email, password);
    setUser({ userid: data.userid, email: data.emailadress });
  }

  // Account erstellen + automatisch einloggen
  async function handleCreateAccount(email, password) {
    const data = await api.createAccount(email, password);
    setUser({ userid: data.userid, email: data.emailadress });
    setShowCreateAccount(false);
  }

  // Endpunkt hinzufuegen: URL normalisieren, speichern, Intervall setzen, Liste refreshen
  async function handleAddEndpoint(rawUrl, seconds) {
    const url = normalizeUrl(rawUrl);
    const data = await api.addEndpoint(user.userid, url);
    await api.setIntervall(data.endpointid, seconds);
    await refreshEndpoints();
    pollUntilReady();
  }

  // Log-Modal oeffnen + Logs laden
  async function handleShowLog(i) {
    const ep = endpoints[i];
    setSelectedEndpoint(ep);
    await fetchLog(ep.endpointid);
    setShowLog(true);
  }

  // Logs fetchen (wird auch vom Polling in LogModal verwendet)
  async function fetchLog(endpointid) {
    try {
      const entries = await api.getLog(endpointid);
      setLogEntries(entries);
    } catch {
      setLogEntries([]);
    }
  }

  // Auto-Refresh fuer offene Logs (alle 5s)
  useEffect(() => {
    if (!showLog || !selectedEndpoint) return;
    const id = setInterval(() => fetchLog(selectedEndpoint.endpointid), 5000);
    return () => clearInterval(id);
  }, [showLog, selectedEndpoint]);

  // Intervall-Modal vorbereiten
  function handleSetIntervall(i) {
    setSelectedEndpoint(endpoints[i]);
    setShowSetIntervall(true);
  }

  async function handleSetIntervallSubmit(endpointid, seconds) {
    await api.setIntervall(endpointid, seconds);
    await refreshEndpoints();
    pollUntilReady();
  }

  // URL-Edit-Modal vorbereiten
  function handleEditUrl(i) {
    setSelectedEndpoint(endpoints[i]);
    setEditUrlValue(endpoints[i].url);
    setShowEditUrl(true);
  }

  // URL speichern: normalisieren + API + Refresh
  async function handleSaveUrl() {
    const ep = selectedEndpoint;
    const url = normalizeUrl(editUrlValue);
    await api.updateEndpoint(ep.endpointid, url);
    await refreshEndpoints();
    pollUntilReady();
  }

  // Delete-Modal vorbereiten
  function handleRemove(i) {
    setDeleteIndex(i);
    setShowDeleteConfirm(true);
  }

  // Endpunkt loeschen (nach Confirmation)
  async function confirmDelete() {
    const ep = endpoints[deleteIndex];
    await api.deleteEndpoint(ep.endpointid);
    await refreshEndpoints();
    pollUntilReady();
    setShowDeleteConfirm(false);
    setDeleteIndex(null);
  }

  // Einzelnen Endpunkt togglen (ON/OFF)
  // Aktualisiert auch den mainSwitch-Status (alle an / alle aus)
  function handleToggleEndpoint(i) {
    setEndpoints(prev => {
      const next = prev.map((ep, j) => j === i ? { ...ep, active: !ep.active } : ep);
      if (next.every(ep => ep.active)) setMainSwitch(true);
      else if (next.every(ep => !ep.active)) setMainSwitch(false);
      return next;
    });
  }

  // Passwort aendern
  async function handleChangePassword(oldPassword, newPassword) {
    const data = await api.changePassword(user.userid, oldPassword, newPassword);
    return data;
  }

  // E-Mail aendern + lokalen State aktualisieren
  async function handleChangeEmail(newEmail) {
    const data = await api.changeEmail(user.userid, newEmail);
    setUser(prev => ({ ...prev, email: newEmail }));
    return data;
  }

  // Account loeschen (mit Confirm-Bestätigung)
  async function handleDeleteAccount() {
    if (!window.confirm('Willst du deinen Account wirklich unwiderruflich löschen?')) return;
    await api.deleteAccount(user.userid);
    setUser(null);
    setEndpoints([]);
    setShowAccountSettings(false);
  }

  // Logout: User-State zurücksetzen
  function handleLogout() {
    setUser(null);
    setEndpoints([]);
  }

  // ════════════════════════════════════════════════════════════════
  //  Render
  // ════════════════════════════════════════════════════════════════

  // Wenn nicht eingeloggt: LandingPage anzeigen (Login/Account erzeugen)
  if (!user) {
    return (
      <ThemeProvider>
        <LandingPage onLogin={handleLogin} onCreateAccount={() => setShowCreateAccount(true)} />
        <CreateAccountModal isOpen={showCreateAccount} onClose={() => setShowCreateAccount(false)} onSubmit={handleCreateAccount} />
      </ThemeProvider>
    );
  }

  // Eingeloggt: Dashboard + alle Modals
  return (
    <ThemeProvider>
      <Dashboard
        endpoints={endpoints}
        mainSwitch={mainSwitch}
        onToggleMainSwitch={() => setMainSwitch(s => { const next = !s; setEndpoints(prev => prev.map(ep => ({ ...ep, active: next }))); return next; })}
        onRemove={handleRemove}
        onToggleEndpoint={handleToggleEndpoint}
        onSetIntervall={handleSetIntervall}
        onShowLog={handleShowLog}
        onEditUrl={handleEditUrl}
        onAddEndpoint={() => setShowAddEndpoint(true)}
        onLogout={handleLogout}
        onAccountSettings={() => setShowAccountSettings(true)}
      />

      {/* Modals – Conditional Rendering via isOpen-Prop */}
      <AddEndpointModal isOpen={showAddEndpoint} onClose={() => setShowAddEndpoint(false)} onSubmit={handleAddEndpoint} />
      <SetIntervallModal isOpen={showSetIntervall} onClose={() => setShowSetIntervall(false)} endpoint={selectedEndpoint} onSubmit={handleSetIntervallSubmit} />
      <DeleteConfirmModal isOpen={showDeleteConfirm} onClose={() => { setShowDeleteConfirm(false); setDeleteIndex(null); }} onConfirm={confirmDelete} />
      <AccountSettingsModal isOpen={showAccountSettings} onClose={() => setShowAccountSettings(false)} onChangePassword={handleChangePassword} onChangeEmail={handleChangeEmail} onDeleteAccount={handleDeleteAccount} />
      <LogModal isOpen={showLog} onClose={() => setShowLog(false)} entries={logEntries} />
      <EditUrlModal isOpen={showEditUrl} onClose={() => setShowEditUrl(false)} endpoint={selectedEndpoint} value={editUrlValue} onChange={setEditUrlValue} onSave={handleSaveUrl} />
    </ThemeProvider>
  );
}

export default App;
