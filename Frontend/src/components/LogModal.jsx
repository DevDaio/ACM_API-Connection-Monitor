// ─── LogModal – Monitoring-Logs anzeigen ───
// Zeigt eine Tabelle mit Status-Einträgen.
// Bietet Filter: all / up / down + Datumsfilter.
// Die Logs werden live aktualisiert (Polling in App.jsx:useEffect).
import { useState, useMemo } from 'react';
import Modal from './Modal';

function LogModal({ isOpen, onClose, entries }) {
  const [filter, setFilter] = useState('all');        // all | up | down
  const [methodFilter, setMethodFilter] = useState('all'); // all | http | tcp | icmp | none
  const [dateFilter, setDateFilter] = useState('');     // optionales Datum (YYYY-MM-DD)

  // useMemo: filtert Logs nur bei Aenderung von entries, filter, methodFilter oder dateFilter
  const filtered = useMemo(() => {
    let list = filter === 'all' ? entries
      : filter === 'up' ? entries.filter(e => e.status === true)
      : entries.filter(e => e.status === false);

    if (methodFilter !== 'all') {
      if (methodFilter === 'none') {
        list = list.filter(e => !e.check_type);
      } else {
        list = list.filter(e => e.check_type === methodFilter);
      }
    }

    if (dateFilter) {
      list = list.filter(e => (e.statusdate || e.date || '').startsWith(dateFilter));
    }
    return list;
  }, [entries, filter, methodFilter, dateFilter]);

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Log" wide>
      {/* ─── Filter-Leiste ─── */}
      <div className="flex flex-wrap gap-2 mb-4 items-center">
        {/* Status-Buttons (All / Up / Down) */}
        {['all', 'up', 'down'].map(f => (
          <button key={f} onClick={() => setFilter(f)}
            className={`px-3 py-1.5 text-sm capitalize transition-colors ${
              filter === f
                ? 'bg-orange-600 text-white'
                : 'bg-gray-800 text-gray-300 hover:text-white border border-gray-700'
            }`}>
            {f === 'all' ? 'All' : f === 'up' ? 'Up' : 'Down'}
          </button>
        ))}
        <span className="w-px h-6 bg-gray-700 mx-1" />
        {/* Method-Buttons (HTTP / TCP / ICMP / —) */}
        {[
          { key: 'all', label: 'All' },
          { key: 'http', label: 'HTTP', cls: 'text-orange-400 border-orange-700 hover:border-orange-500' },
          { key: 'tcp', label: 'TCP', cls: 'text-purple-400 border-purple-700 hover:border-purple-500' },
          { key: 'icmp', label: 'ICMP', cls: 'text-cyan-400 border-cyan-700 hover:border-cyan-500' },
          { key: 'none', label: '—', cls: 'text-gray-500 border-gray-600' },
        ].map(m => (
          <button key={m.key} onClick={() => setMethodFilter(m.key)}
            className={`px-3 py-1.5 text-xs font-mono font-bold border transition-colors ${
              methodFilter === m.key
                ? 'bg-gray-700 ' + m.cls
                : 'bg-gray-800 text-gray-300 border-gray-700 hover:border-gray-500'
            }`}>
            {m.label}
          </button>
        ))}
        <div className="ml-auto" />
        {/* Datums-Filter */}
        <input type="date" value={dateFilter} onChange={e => setDateFilter(e.target.value)}
          className="bg-gray-800 border border-gray-700 px-2 py-1.5 text-xs text-white font-mono focus:outline-none ac-ring" />
        {dateFilter && <button onClick={() => setDateFilter('')} className="text-gray-500 hover:text-white text-xs font-mono">&times;</button>}
      </div>

      {/* ─── Log-Tabelle ─── */}
      <div className="max-h-[55vh]">
          <table className="w-full text-sm text-left">
            <thead>
              <tr className="text-gray-300 border-b border-gray-700">
                <th className="pb-2 pr-4 whitespace-nowrap">ID</th>
                <th className="pb-2 pr-4 whitespace-nowrap">Status</th>
                <th className="pb-2 pr-4 whitespace-nowrap">Time</th>
                <th className="pb-2 pr-4 whitespace-nowrap">Method</th>
                <th className="pb-2 pr-4 whitespace-nowrap">Date</th>
                <th className="pb-2 whitespace-nowrap">URL</th>
              </tr>
            </thead>
            <tbody>
              {filtered.length === 0 ? (
                <tr><td colSpan="6" className="py-6 text-gray-300 text-center">Keine Einträge für diesen Filter</td></tr>
              ) : (
                filtered.map((entry, i) => (
                  <tr key={i} className="border-b border-gray-800 text-white">
                    <td className="py-2 pr-4 font-mono whitespace-nowrap">{entry.endpointid}</td>
                    <td className="py-2 pr-4 whitespace-nowrap">
                      {entry.status === null || entry.status === undefined ? (
                        <span className="text-yellow-400 text-xs font-mono tracking-wider">EDITED</span>
                      ) : (
                        <>
                          <span className={`inline-block w-2 h-2 ${entry.status ? 'bg-green-500' : 'bg-red-500'} mr-2`} />
                          {entry.status ? 'Up' : 'Down'}
                        </>
                      )}
                    </td>
                    <td className="py-2 pr-4 font-mono whitespace-nowrap">{(entry.statustime || entry.time || '').split('.').shift()}</td>
                    <td className="py-2 pr-4 whitespace-nowrap">
                      {entry.check_type === 'icmp' ? (
                        <span className="text-cyan-400 text-xs font-mono font-bold border border-cyan-700 px-1.5 py-0.5">ICMP</span>
                      ) : entry.check_type === 'tcp' ? (
                        <span className="text-purple-400 text-xs font-mono font-bold border border-purple-700 px-1.5 py-0.5">TCP</span>
                      ) : entry.check_type === 'http' ? (
                        <span className="text-orange-400 text-xs font-mono font-bold border border-orange-700 px-1.5 py-0.5">HTTP</span>
                      ) : (
                        <span className="text-gray-500 text-xs font-mono">—</span>
                      )}
                    </td>
                    <td className="py-2 pr-4 font-mono whitespace-nowrap">{entry.statusdate || entry.date}</td>
                    <td className="py-2 font-mono text-xs">{entry.url || '—'}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
      </div>
    </Modal>
  );
}

export default LogModal;
