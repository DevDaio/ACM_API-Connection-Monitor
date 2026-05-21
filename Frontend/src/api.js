// ─── API-Client ───
// Basis-URL: import.meta.env.VITE_API_URL (wird von Vite via .env gesetzt)
// Fallback: '/acm' (wenn kein env gesetzt ist, z. B. im Dev-Mode)
const BASE = (typeof import.meta !== 'undefined' && import.meta.env.VITE_API_URL) || '/acm';

// Generische Request-Funktion: Methode, Pfad, optionaler JSON-Body
async function request(method, path, body) {
  const opts = {
    method,
    headers: { 'Content-Type': 'application/json' },
  };
  if (body) opts.body = JSON.stringify(body);  // POST/PUT-Body serialisieren
  const res = await fetch(`${BASE}${path}`, opts);
  const data = await res.json();
  // Nicht-2xx-Status => Fehler werfen (wird in den Komponenten gecatcht)
  if (!res.ok) throw new Error(data.error || 'Request failed');
  return data;
}

// ─── API-Funktionen ───
// Jede Funktion ruft request() mit der passenden Methode + Pfad auf.
// Die Pfad-Signaturen müssen mit den Backend-Routen uebereinstimmen.
export const api = {
  login: (email, password) =>
    request('POST', '/login', { email, password }),

  createAccount: (email, password) =>
    request('POST', '/createAccount', { email, password }),

  getHome: (userid) =>
    request('GET', `/home?id=${userid}`),  // Query-Parameter: ?id=

  getUser: (userid) =>
    request('GET', `/user?id=${userid}`),

  changePassword: (userid, oldPassword, newPassword) =>
    request('PUT', '/user/changePassword', { userid, old_password: oldPassword, new_password: newPassword }),

  changeEmail: (userid, newEmail) =>
    request('PUT', '/user/changeEmail', { userid, new_email: newEmail }),

  deleteAccount: (userid) =>
    request('DELETE', '/user/deleteAccount', { userid }),

  addEndpoint: (userid, url) =>
    request('PUT', '/addEndpoint', { userid, url }),

  setIntervall: (endpointid, seconds) =>
    request('PUT', '/setIntervall', { endpointid, seconds }),

  deleteEndpoint: (endpointid) =>
    request('PUT', '/deleteConfirm', { endpointid }),

  getLog: (endpointid) =>
    request('GET', `/log?id=${endpointid}`),

  updateEndpoint: (endpointid, url) =>
    request('PUT', '/updateEndpoint', { endpointid, url }),
};
