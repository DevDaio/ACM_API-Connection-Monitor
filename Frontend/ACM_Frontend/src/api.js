const BASE = (typeof import.meta !== 'undefined' && import.meta.env.VITE_API_URL) || '/acm';

async function request(method, path, body) {
  const opts = {
    method,
    headers: { 'Content-Type': 'application/json' },
  };
  if (body) opts.body = JSON.stringify(body);
  const res = await fetch(`${BASE}${path}`, opts);
  const data = await res.json();
  if (!res.ok) throw new Error(data.error || 'Request failed');
  return data;
}

export const api = {
  login: (email, password) =>
    request('POST', '/login', { email, password }),

  createAccount: (email, password) =>
    request('POST', '/createAccount', { email, password }),

  getHome: (userid) =>
    request('GET', `/home?id=${userid}`),

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
