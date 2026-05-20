/* ═══════════════════════════════════════════════════════
 * 📦 HAUPT-KOMPONENTE (STATE-HUB)
 *
 * 🎯 ZWECK:
 * Zentrale State-Verwaltung der gesamten App.
 * Hält User-Session, Endpoint-Liste und Modal-Status.
 * Verteilt Callbacks per Props an Child-Komponenten.
 *
 * 🔗 DEPENDENCIES:
 * - React (useState, useEffect, useRef)
 * - api.js (HTTP-Requests)
 * - ThemeContext (ThemeProvider)
 * - Alle Modal-Komponenten
 * - Dashboard + LandingPage
 *
 * 💡 KONZEPTE:
 * - State-Hub Pattern (kein Router, kein Redux)
 * - useEffect für Polling (10s-Intervall)
 * - useEffect für localStorage-Persistenz
 * - useRef für Modal-Check (Polling-Pause)
 *
 * 🎓 LERN-TIPP:
 * App.jsx ist bewusst monolithisch. Das ist ein
 * legitimes Pattern für kleine bis mittlere Apps.
 * Alle State-Änderungen passieren hier zentral.
 * ═══════════════════════════════════════════════════════ */

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

function App() {
    /* ─── STATE-VARIABLEN ─── */
    // User-Session: aus localStorage wiederhergestellt
    const [user, setUser] = useState(() => {
        const saved = localStorage.getItem('acm_user');
        return saved ? JSON.parse(saved) : null;
    });
    const [endpoints, setEndpoints] = useState([]);
    const [mainSwitch, setMainSwitch] = useState(true);

    // Modal-Open-States (alle als einzelne Booleans)
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

    /* ─── LOCALSTORAGE-PERSISTENZ ─── */
    useEffect(() => {
        if (user) {
            localStorage.setItem('acm_user', JSON.stringify(user));
        } else {
            localStorage.removeItem('acm_user');
        }
    }, [user]);

    /* ─── MODAL-REF (für Polling-Pause) ─── */
    // useRef: mutable Referenz, die keinen Re-Render auslöst
    const anyModalOpen =
        showCreateAccount || showAddEndpoint || showSetIntervall ||
        showDeleteConfirm || showAccountSettings || showLog;
    const anyModalRef = useRef(anyModalOpen);
    useEffect(() => { anyModalRef.current = anyModalOpen; }, [anyModalOpen]);

    /* ─── HELPERS ─── */
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

    /* ─── API-DATEN → FRONTEND-FORMAT ─── */
    function mapEndpoints(data) {
        return data.map(ep => ({
            endpointid: ep.endpointid,
            url: ep.url,
            active: true,
            status: ep.status === null ? 'Unknown' : (ep.status ? 'Running' : 'Down'),
            durationSeconds: ep.duration_seconds,
            interval: fmtInterval(ep.interval_seconds),
            sparkHistory: ep.status_history,
            changedate: ep.statusdate || '--',
            changetime: ep.statustime ? ep.statustime.split('.').shift() : '--',
        }));
    }

    /* ─── INITIALER DATEN-LOAD ─── */
    useEffect(() => {
        if (!user) return;
        api.getHome(user.userid).then(d => setEndpoints(mapEndpoints(d))).catch(() => {});
    }, [user]);

    /* ─── POLLING (10s Intervall) ─── */
    useEffect(() => {
        if (!user) return;
        const tick = setInterval(() => {
            setEndpoints(prev => prev.map(ep =>
                ep.durationSeconds != null ? { ...ep, durationSeconds: ep.durationSeconds + 1 } : ep
            ));
        }, 1000);
        const poll = setInterval(() => {
            if (anyModalRef.current) return;  // Kein Polling bei offenem Modal
            api.getHome(user.userid).then(d => setEndpoints(mapEndpoints(d))).catch(() => {});
        }, 10000);
        return () => { clearInterval(tick); clearInterval(poll); };
    }, [user]);

    /* ─── EVENT-HANDLER ─── */
    async function handleLogin(email, password) {
        const data = await api.login(email, password);
        setUser({ userid: data.userid, email: data.emailadress });
    }

    async function handleCreateAccount(email, password) {
        const data = await api.createAccount(email, password);
        setUser({ userid: data.userid, email: data.emailadress });
        setShowCreateAccount(false);
    }

    async function handleAddEndpoint(rawUrl, seconds) {
        const url = rawUrl.match(/^[a-zA-Z][a-zA-Z0-9+\-.]*:\/\//) ? rawUrl : `https://${rawUrl}`;
        const data = await api.addEndpoint(user.userid, url);
        await api.setIntervall(data.endpointid, seconds);
        const fresh = await api.getHome(user.userid);
        setEndpoints(mapEndpoints(fresh));
    }

    async function handleShowLog(i) {
        const ep = endpoints[i];
        setSelectedEndpoint(ep);
        await fetchLog(ep.endpointid);
        setShowLog(true);
    }

    async function fetchLog(endpointid) {
        try {
            const entries = await api.getLog(endpointid);
            setLogEntries(entries);
        } catch {
            setLogEntries([]);
        }
    }

    // Log-Polling alle 5s bei offenem Log-Modal
    useEffect(() => {
        if (!showLog || !selectedEndpoint) return;
        const id = setInterval(() => fetchLog(selectedEndpoint.endpointid), 5000);
        return () => clearInterval(id);
    }, [showLog, selectedEndpoint]);

    function handleSetIntervall(i) {
        setSelectedEndpoint(endpoints[i]);
        setShowSetIntervall(true);
    }

    async function handleSetIntervallSubmit(endpointid, seconds) {
        await api.setIntervall(endpointid, seconds);
    }

    function handleEditUrl(i) {
        setSelectedEndpoint(endpoints[i]);
        setEditUrlValue(endpoints[i].url);
        setShowEditUrl(true);
    }

    async function handleSaveUrl() {
        const ep = selectedEndpoint;
        const url = editUrlValue.match(/^[a-zA-Z][a-zA-Z0-9+\-.]*:\/\//) ? editUrlValue : `https://${editUrlValue}`;
        await api.updateEndpoint(ep.endpointid, url);
        setEndpoints(prev => prev.map(e => e.endpointid === ep.endpointid ? { ...e, url } : e));
    }

    function handleRemove(i) {
        setDeleteIndex(i);
        setShowDeleteConfirm(true);
    }

    async function confirmDelete() {
        const ep = endpoints[deleteIndex];
        await api.deleteEndpoint(ep.endpointid);
        setEndpoints(prev => prev.filter((_, i) => i !== deleteIndex));
        setShowDeleteConfirm(false);
        setDeleteIndex(null);
    }

    function handleToggleEndpoint(i) {
        setEndpoints(prev => {
            const next = prev.map((ep, j) => j === i ? { ...ep, active: !ep.active } : ep);
            if (next.every(ep => ep.active)) setMainSwitch(true);
            else if (next.every(ep => !ep.active)) setMainSwitch(false);
            return next;
        });
    }

    async function handleChangePassword(oldPassword, newPassword) {
        return await api.changePassword(user.userid, oldPassword, newPassword);
    }

    async function handleChangeEmail(newEmail) {
        const data = await api.changeEmail(user.userid, newEmail);
        setUser(prev => ({ ...prev, email: newEmail }));
        return data;
    }

    async function handleDeleteAccount() {
        if (!window.confirm('Willst du deinen Account wirklich unwiderruflich löschen?')) return;
        await api.deleteAccount(user.userid);
        setUser(null);
        setEndpoints([]);
        setShowAccountSettings(false);
    }

    function handleLogout() {
        setUser(null);
        setEndpoints([]);
    }

    /* ─── RENDER ─── */
    if (!user) {
        return (
            <ThemeProvider>
                <LandingPage onLogin={handleLogin} onCreateAccount={() => setShowCreateAccount(true)} />
                <CreateAccountModal isOpen={showCreateAccount} onClose={() => setShowCreateAccount(false)} onSubmit={handleCreateAccount} />
            </ThemeProvider>
        );
    }

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
