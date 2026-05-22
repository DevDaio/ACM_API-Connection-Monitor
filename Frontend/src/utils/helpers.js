export function fmtDuration(secs) {
  if (secs == null) return '--:--:--';
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

export function fmtInterval(secs) {
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

export function normalizeUrl(raw) {
  if (/^[a-zA-Z][a-zA-Z0-9+\-.]*:\/\//.test(raw)) return raw;
  if (/^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}(:\d+)?(\/.*)?$/.test(raw)) return `http://${raw}`;
  if (/^\[[\da-fA-F:]+\](:\d+)?(\/.*)?$/.test(raw)) return `http://${raw}`;
  const ipv6 = raw.match(/^([\da-fA-F:]+):(\d{2,5})(\/.*)?$/);
  if (ipv6) return `http://[${ipv6[1]}]:${ipv6[2]}${ipv6[3]||''}`;
  if (/^[\da-fA-F:]+$/.test(raw) && raw.includes(':')) return `http://[${raw}]`;
  if (/^localhost(:\d+)?(\/.*)?$/i.test(raw)) return `http://${raw}`;
  return `https://${raw}`;
}

export function mapEndpoints(data) {
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
