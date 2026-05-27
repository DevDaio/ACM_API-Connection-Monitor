import { useState, useEffect, useRef, useCallback } from 'react';
import { api, setToken } from '../api';
import { normalizeUrl, mapEndpoints } from '../utils/helpers';

export function useAppState() {
  const [user, setUser] = useState(() => {
    const saved = localStorage.getItem('acm_user');
    return saved ? JSON.parse(saved) : null;
  });

  const [endpoints, setEndpoints] = useState([]);
  const [mainSwitch, setMainSwitch] = useState(true);

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

  const anyModalRef = useRef(false);
  const pollRef = useRef(null);

  useEffect(() => {
    if (user) {
      localStorage.setItem('acm_user', JSON.stringify(user));
    } else {
      localStorage.removeItem('acm_user');
    }
  }, [user]);

  const anyModalOpen =
    showCreateAccount || showAddEndpoint || showSetIntervall ||
    showDeleteConfirm || showAccountSettings || showLog;

  useEffect(() => { anyModalRef.current = anyModalOpen; }, [anyModalOpen]);
  useEffect(() => { return () => { if (pollRef.current) clearInterval(pollRef.current); }; }, []);

  // Hilfsfunktion: Bei 401 (ungültiger Token) ausloggen
  const handleAuthError = useCallback(() => {
    if (!localStorage.getItem('acm_token')) {
      setToken(null);
      setUser(null);
      setEndpoints([]);
    }
  }, []);

  const refreshEndpoints = useCallback(async () => {
    if (!user) return;
    try {
      const d = await api.getHome();
      setEndpoints(mapEndpoints(d));
    } catch (e) {
      console.error('refreshEndpoints failed', e);
      handleAuthError();
    }
  }, [user, handleAuthError]);

  useEffect(() => {
    if (!user) return;
    api.getHome()
      .then(d => setEndpoints(mapEndpoints(d)))
      .catch(e => { console.error(e); handleAuthError(); });
  }, [user, handleAuthError]);

  useEffect(() => {
    if (!user) return;
    const tick = setInterval(() => {
      setEndpoints(prev => prev.map(ep =>
        ep.durationSeconds != null ? { ...ep, durationSeconds: ep.durationSeconds + 1 } : ep
      ));
    }, 1000);
    const poll = setInterval(() => {
      if (anyModalRef.current) return;
      api.getHome()
        .then(d => setEndpoints(mapEndpoints(d)))
        .catch(() => handleAuthError());
    }, 10000);
    return () => { clearInterval(tick); clearInterval(poll); };
  }, [user, handleAuthError]);

  const pollUntilReady = useCallback(() => {
    if (pollRef.current) clearInterval(pollRef.current);
    let tries = 0;
    pollRef.current = setInterval(async () => {
      tries++;
      await refreshEndpoints();
      if (tries >= 8) { clearInterval(pollRef.current); pollRef.current = null; }
    }, 2000);
  }, [refreshEndpoints]);

  const handleLogin = useCallback(async (email, password) => {
    const data = await api.login(email, password);
    setUser({ userid: data.userid, email: data.emailadress });
  }, []);

  const handleCreateAccount = useCallback(async (email, password) => {
    const data = await api.createAccount(email, password);
    setUser({ userid: data.userid, email: data.emailadress });
    setShowCreateAccount(false);
  }, []);

  const handleLogout = useCallback(() => {
    setToken(null);
    setUser(null);
    setEndpoints([]);
  }, []);

  const handleToggleMainSwitch = useCallback(() => {
    setMainSwitch(s => {
      const next = !s;
      setEndpoints(prev => prev.map(ep => ({ ...ep, active: next })));
      return next;
    });
  }, []);

  const handleToggleEndpoint = useCallback((i) => {
    setEndpoints(prev => {
      const next = prev.map((ep, j) => j === i ? { ...ep, active: !ep.active } : ep);
      if (next.every(ep => ep.active)) setMainSwitch(true);
      else if (next.every(ep => !ep.active)) setMainSwitch(false);
      return next;
    });
  }, []);

  const handleAddEndpoint = useCallback(async (rawUrl, seconds, checkType = 'http') => {
    const url = checkType !== 'http' ? rawUrl.trim() : normalizeUrl(rawUrl);
    const data = await api.addEndpoint(url, checkType);
    await api.setIntervall(data.endpointid, seconds);
    await refreshEndpoints();
    pollUntilReady();
  }, [refreshEndpoints, pollUntilReady]);

  const handleSetIntervall = useCallback((i) => {
    setSelectedEndpoint(endpoints[i]);
    setShowSetIntervall(true);
  }, [endpoints]);

  const handleSetIntervallSubmit = useCallback(async (endpointid, seconds) => {
    await api.setIntervall(endpointid, seconds);
    await refreshEndpoints();
    pollUntilReady();
  }, [refreshEndpoints, pollUntilReady]);

  const handleRemove = useCallback((i) => {
    setDeleteIndex(i);
    setShowDeleteConfirm(true);
  }, []);

  const confirmDelete = useCallback(async () => {
    const ep = endpoints[deleteIndex];
    await api.deleteEndpoint(ep.endpointid);
    await refreshEndpoints();
    pollUntilReady();
    setShowDeleteConfirm(false);
    setDeleteIndex(null);
  }, [endpoints, deleteIndex, refreshEndpoints, pollUntilReady]);

  const handleEditUrl = useCallback((i) => {
    setSelectedEndpoint(endpoints[i]);
    setEditUrlValue(endpoints[i].url);
    setShowEditUrl(true);
  }, [endpoints]);

  const handleSaveUrl = useCallback(async (checkType) => {
    const ep = selectedEndpoint;
    const url = checkType !== 'http' ? editUrlValue.trim() : normalizeUrl(editUrlValue);
    await api.updateEndpoint(ep.endpointid, url, checkType);
    await refreshEndpoints();
    pollUntilReady();
  }, [selectedEndpoint, editUrlValue, refreshEndpoints, pollUntilReady]);

  const handleChangePassword = useCallback(async (oldPassword, newPassword) => {
    return await api.changePassword(oldPassword, newPassword);
  }, []);

  const handleChangeEmail = useCallback(async (newEmail) => {
    await api.changeEmail(newEmail);
    setUser(prev => ({ ...prev, email: newEmail }));
  }, []);

  const handleDeleteAccount = useCallback(async () => {
    if (!window.confirm('Willst du deinen Account wirklich unwiderruflich löschen?')) return;
    await api.deleteAccount();
    setToken(null);
    setUser(null);
    setEndpoints([]);
    setShowAccountSettings(false);
  }, []);

  const fetchLog = useCallback(async (endpointid) => {
    try {
      const entries = await api.getLog(endpointid);
      setLogEntries(entries);
    } catch {
      setLogEntries([]);
    }
  }, []);

  const handleShowLog = useCallback(async (i) => {
    const ep = endpoints[i];
    setSelectedEndpoint(ep);
    await fetchLog(ep.endpointid);
    setShowLog(true);
  }, [endpoints, fetchLog]);

  useEffect(() => {
    if (!showLog || !selectedEndpoint) return;
    const id = setInterval(() => fetchLog(selectedEndpoint.endpointid), 5000);
    return () => clearInterval(id);
  }, [showLog, selectedEndpoint, fetchLog]);

  return {
    user, endpoints, mainSwitch,
    showCreateAccount, setShowCreateAccount,
    showAddEndpoint, setShowAddEndpoint,
    showSetIntervall, setShowSetIntervall,
    showDeleteConfirm, setShowDeleteConfirm,
    setDeleteIndex,
    showAccountSettings, setShowAccountSettings,
    showLog, setShowLog,
    selectedEndpoint, logEntries,
    showEditUrl, setShowEditUrl,
    editUrlValue, setEditUrlValue,
    handleLogin, handleCreateAccount, handleLogout,
    handleToggleMainSwitch, handleToggleEndpoint,
    handleAddEndpoint,
    handleSetIntervall, handleSetIntervallSubmit,
    handleRemove, confirmDelete,
    handleEditUrl, handleSaveUrl,
    handleChangePassword, handleChangeEmail, handleDeleteAccount,
    handleShowLog,
  };
}
