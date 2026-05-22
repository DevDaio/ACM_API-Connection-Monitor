import { useState, useEffect, useRef, useCallback } from 'react';
import { api } from '../api';
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

  const refreshEndpoints = useCallback(async () => {
    if (!user) return;
    try {
      const d = await api.getHome(user.userid);
      setEndpoints(mapEndpoints(d));
    } catch (e) {
      console.error('refreshEndpoints failed', e);
    }
  }, [user]);

  useEffect(() => {
    if (!user) return;
    api.getHome(user.userid).then(d => setEndpoints(mapEndpoints(d))).catch(console.error);
  }, [user]);

  useEffect(() => {
    if (!user) return;
    const tick = setInterval(() => {
      setEndpoints(prev => prev.map(ep =>
        ep.durationSeconds != null ? { ...ep, durationSeconds: ep.durationSeconds + 1 } : ep
      ));
    }, 1000);
    const poll = setInterval(() => {
      if (anyModalRef.current) return;
      api.getHome(user.userid).then(d => setEndpoints(mapEndpoints(d))).catch(() => {});
    }, 10000);
    return () => { clearInterval(tick); clearInterval(poll); };
  }, [user]);

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

  const handleAddEndpoint = useCallback(async (rawUrl, seconds) => {
    const url = normalizeUrl(rawUrl);
    const data = await api.addEndpoint(user.userid, url);
    await api.setIntervall(data.endpointid, seconds);
    await refreshEndpoints();
    pollUntilReady();
  }, [user, refreshEndpoints, pollUntilReady]);

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

  const handleSaveUrl = useCallback(async () => {
    const ep = selectedEndpoint;
    const url = normalizeUrl(editUrlValue);
    await api.updateEndpoint(ep.endpointid, url);
    await refreshEndpoints();
    pollUntilReady();
  }, [selectedEndpoint, editUrlValue, refreshEndpoints, pollUntilReady]);

  const handleChangePassword = useCallback(async (oldPassword, newPassword) => {
    return await api.changePassword(user.userid, oldPassword, newPassword);
  }, [user]);

  const handleChangeEmail = useCallback(async (newEmail) => {
    await api.changeEmail(user.userid, newEmail);
    setUser(prev => ({ ...prev, email: newEmail }));
  }, [user]);

  const handleDeleteAccount = useCallback(async () => {
    if (!window.confirm('Willst du deinen Account wirklich unwiderruflich löschen?')) return;
    await api.deleteAccount(user.userid);
    setUser(null);
    setEndpoints([]);
    setShowAccountSettings(false);
  }, [user]);

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
