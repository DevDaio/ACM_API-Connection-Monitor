import Sparkline from './Sparkline';
import { fmtDuration } from '../utils/helpers';

function EndpointCard({ endpoint, onRemove, onToggle, onSetIntervall, onShowLog, onEditUrl }) {
  // Status-Bestimmung
  const isRunning = endpoint.status === 'Running';
  const isDown = endpoint.status === 'Down';
  // LED-Farbe: gruen (Running) / rot (Down) / grau (Unknown)
  const ledColor = isRunning
    ? 'bg-green-500 shadow-[0_0_12px_rgba(34,197,94,0.8)]'
    : isDown
    ? 'bg-red-500 shadow-[0_0_12px_rgba(239,68,68,0.8)]'
    : 'bg-gray-700';

  return (
    // Doppelklick öffnet Log-Modal
    <tr className="border-b ac-bd hover:bg-orange-950/[0.08] transition-colors cursor-pointer" onDoubleClick={onShowLog}>
      {/* ─── Status-LED + Text ─── */}
      <td className="py-3.5 px-4">
        <div className="flex items-center gap-3">
          <span className={`inline-block w-3 h-3 shrink-0 ${ledColor}`} />
          <span className={`text-xs font-mono font-bold tracking-wider ${
            isRunning ? 'text-green-400' : isDown ? 'text-red-400' : 'text-gray-300'
          }`}>{endpoint.status.toUpperCase()}</span>
        </div>
      </td>
      {/* ─── URL + EDIT-Button ─── */}
      <td className="py-3.5 px-4">
        <div className="flex items-center gap-2 max-w-xs">
          {/* text-truncate: Lange URLs werden mit "..." abgeschnitten */}
          <span className="text-sm text-gray-200 font-mono truncate">{endpoint.url}</span>
          <button onClick={onEditUrl} className="text-gray-300 ac-tx-hover shrink-0 text-[10px] font-bold border border-gray-700 px-1.5 py-0.5 leading-none" title="Edit URL">EDIT</button>
        </div>
      </td>
      {/* ─── Uptime-Dauer ─── */}
      <td className="py-3.5 px-4 text-sm text-gray-200 font-mono tabular-nums tracking-wider">{fmtDuration(endpoint.durationSeconds)}</td>
      {/* ─── Scan-Intervall ─── */}
      <td className="py-3.5 px-4 text-sm text-gray-300 font-mono">{endpoint.interval}</td>
      {/* ─── Sparkline-Chart ─── */}
      <td className="py-3.5 px-4">
        <Sparkline data={endpoint.sparkHistory} />
      </td>
      {/* ─── Letzte Aenderung (Datum + Uhrzeit) ─── */}
      <td className="py-3.5 px-4 text-xs text-gray-300 font-mono whitespace-nowrap">{endpoint.changedate}<br />{endpoint.changetime}</td>
      {/* ─── Steuerungs-Buttons ─── */}
      <td className="py-3.5 px-4">
        <div className="flex items-center justify-end gap-2">
          {/* ON/OFF-Toggle (individuell pro Endpunkt) */}
          <div className="flex items-center gap-1.5">
            <button onClick={onToggle}
              className={`relative w-8 h-1 transition-all rounded-full ${endpoint.active ? 'bg-green-500 shadow-[0_0_6px_rgba(34,197,94,0.5)]' : 'bg-gray-700'}`}>
              <span className={`absolute -top-[3.5px] w-2 h-2 rounded-full transition-all border ${
                endpoint.active
                  ? 'right-0 bg-green-400 border-green-400 shadow-[0_0_6px_rgba(34,197,94,0.7)]'
                  : 'left-0 bg-gray-500 border-gray-500'
              }`} />
            </button>
            <span className={`text-[8px] font-mono tracking-wider ${
              endpoint.active ? 'text-green-500' : 'text-gray-300'
            }`}>{endpoint.active ? 'ON' : 'OFF'}</span>
          </div>
          {/* Intervall-Button (Uhr-Symbol) */}
          <button onClick={onSetIntervall} className="text-gray-300 ac-tx-hover shrink-0" title="Interval">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
          </button>
          {/* Log-Button (Hamburger-Menü) */}
          <button onClick={onShowLog} className="text-gray-300 hover:text-white shrink-0 text-lg" title="Log">&#9776;</button>
          {/* Remove-Button (X) */}
          <button onClick={onRemove} className="text-gray-300 hover:text-red-400 shrink-0 text-lg ml-1" title="Remove">&times;</button>
        </div>
      </td>
    </tr>
  );
}

export default EndpointCard;
