/* ═══════════════════════════════════════════════════════
 * 📦 LOG MODAL
 *
 * 🎯 ZWECK:
 * Zeigt alle Monitoring-Logs eines Endpoints.
 * Filter: All / Up / Down + Datums-Filter.
 *
 * 💡 KONZEPTE:
 * - useMemo für gefilterte Liste (Performance)
 * - Filter-Buttons (all | up | down)
 * - Datums-Input (<input type="date">)
 * - Live-Aktualisierung (Log-Polling alle 5s in App.jsx)
 * ═══════════════════════════════════════════════════════ */

import { useState, useMemo } from 'react';
import Modal from './Modal';

function LogModal({ isOpen, onClose, entries }) {
    const [filter, setFilter] = useState('all');
    const [dateFilter, setDateFilter] = useState('');

    const filtered = useMemo(() => {
        let list = filter === 'all' ? entries
            : filter === 'up' ? entries.filter(e => e.status === true)
            : entries.filter(e => e.status === false);

        if (dateFilter) {
            list = list.filter(e => (e.statusdate || e.date || '').startsWith(dateFilter));
        }
        return list;
    }, [entries, filter, dateFilter]);

    return (
        <Modal isOpen={isOpen} onClose={onClose} title="Log">
            <div className="flex flex-wrap gap-2 mb-4 items-center">
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
                <div className="ml-auto" />
                <input type="date" value={dateFilter} onChange={e => setDateFilter(e.target.value)}
                    className="bg-gray-800 border border-gray-700 px-2 py-1.5 text-xs text-white font-mono focus:outline-none ac-ring" />
                {dateFilter && <button onClick={() => setDateFilter('')} className="text-gray-500 hover:text-white text-xs font-mono">&times;</button>}
            </div>
            <div className="overflow-auto max-h-[55vh]">
                <table className="w-full text-sm text-left">
                    <thead>
                        <tr className="text-gray-300 border-b border-gray-700">
                            <th className="pb-2 pr-4">EndpointID</th>
                            <th className="pb-2 pr-4">Status</th>
                            <th className="pb-2 pr-4">StatusTime</th>
                            <th className="pb-2">StatusDay</th>
                        </tr>
                    </thead>
                    <tbody>
                        {filtered.length === 0 ? (
                            <tr><td colSpan="4" className="py-6 text-gray-300 text-center">Keine Einträge für diesen Filter</td></tr>
                        ) : (
                            filtered.map((entry, i) => (
                                <tr key={i} className="border-b border-gray-800 text-white">
                                    <td className="py-2 pr-4 font-mono">{entry.endpointid}</td>
                                    <td className="py-2 pr-4">
                                        <span className={`inline-block w-2 h-2 ${entry.status ? 'bg-green-500' : 'bg-red-500'} mr-2`} />
                                        {entry.status ? 'Up' : 'Down'}
                                    </td>
                                    <td className="py-2 pr-4 font-mono">{(entry.statustime || entry.time || '').split('.').shift()}</td>
                                    <td className="py-2 font-mono">{entry.statusdate || entry.date}</td>
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
