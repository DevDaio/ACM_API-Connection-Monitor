// ─── API-Client ───
// Basis-URL: import.meta.env.VITE_API_URL (wird von Vite via .env gesetzt)
// Fallback: '/acm' (wenn kein env gesetzt ist, z. B. im Dev-Mode)
const BASE = (typeof import.meta !== 'undefined' && import.meta.env.VITE_API_URL) || '/acm';

// ─── Session-Token-Management ───
// Wird beim Login/Registrieren gesetzt und bei 401 oder Logout gelöscht.
let _token = localStorage.getItem('acm_token');

export function setToken(t) {
  _token = t;
  if (t) localStorage.setItem('acm_token', t);
  else localStorage.removeItem('acm_token');
}

// Token aus vorheriger Sitzung wiederherstellen (Page-Refresh überlebt)
const savedToken = localStorage.getItem('acm_token');
if (savedToken) _token = savedToken;

// ─── Generische Request-Funktion ───
// Hängt bei vorhandenem Token automatisch Authorization: Bearer an.
async function request(method, path, body) {
  const opts = {
    method,
    headers: {},
  };
  if (_token) opts.headers['Authorization'] = `Bearer ${_token}`;
  if (body) {
    opts.headers['Content-Type'] = 'application/json';
    opts.body = JSON.stringify(body);
  }

  const res = await fetch(`${BASE}${path}`, opts);
  const data = await res.json();

  // Bei 401 (ungültiger/abgelaufener Token) Session löschen
  if (!res.ok) {
    if (res.status === 401) setToken(null);
    throw new Error(data.error || 'Request failed');
  }
  return data;
}

// ─── API-Funktionen ───
// userid wird nicht mehr mitgesendet – der Server extrahiert sie aus dem Token.
export const api = {
  login: (email, password) =>
    request('POST', '/login', { email, password }).then(data => {
      setToken(data.token);
      return data;
    }),

  createAccount: (email, password) =>
    request('POST', '/createAccount', { email, password }).then(data => {
      setToken(data.token);
      return data;
    }),

  getHome: () =>
    request('GET', '/home'),

  getUser: () =>
    request('GET', '/user'),

  changePassword: (oldPassword, newPassword) =>
    request('PUT', '/user/changePassword', { old_password: oldPassword, new_password: newPassword }),

  changeEmail: (newEmail) =>
    request('PUT', '/user/changeEmail', { new_email: newEmail }),

  deleteAccount: () =>
    request('DELETE', '/user/deleteAccount'),

  addEndpoint: (url, checkType = 'http') =>
    request('PUT', '/addEndpoint', { url, check_type: checkType }),

  setIntervall: (endpointid, seconds) =>
    request('PUT', '/setIntervall', { endpointid, seconds }),

  deleteEndpoint: (endpointid) =>
    request('PUT', '/deleteConfirm', { endpointid }),

  getLog: (endpointid) =>
    request('GET', `/log?id=${endpointid}`),

  updateEndpoint: (endpointid, url, checkType) =>
    request('PUT', '/updateEndpoint', { endpointid, url, check_type: checkType || null }),
};
