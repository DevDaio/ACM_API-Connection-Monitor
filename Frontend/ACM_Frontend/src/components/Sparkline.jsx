// ─── Sparkline – Mini-Diagramm ───
// Zeigt die letzten 30 Status-Einträge als Liniendiagramm.
// Jeder Punkt: oben = up (grün), unten = down (rot).
// Zeigt darunter den Prozentsatz der Up-Zeit an.
function Sparkline({ data, width = 70, height = 24 }) {
  // Mindestens 2 Datenpunkte für eine Linie nötig
  if (!data || data.length < 2) return <span className="text-gray-500 text-[11px] font-mono">N/A</span>;

  const w = width;
  const h = height;
  const n = data.length;
  const stepX = (w - 2) / (n - 1);  // Horizontaler Abstand zwischen Punkten
  const pad = 1;                     // Padding (1px Rand)

  // Punkte berechnen: y = pad (oben) bei true, y = h - pad (unten) bei false
  const points = data.map((val, i) => {
    const x = pad + i * stepX;
    const y = val ? pad : h - pad;
    return `${x},${y}`;
  });

  const up = data.filter(Boolean).length;
  const pct = Math.round((up / n) * 100);  // Up-Prozentsatz
  const color = data[data.length - 1] ? '#22c55e' : '#ef4444';  // Farbe basierend auf letztem Wert

  return (
    <div className="flex items-center gap-2">
      {/* SVG-Line-Chart */}
      <svg width={w} height={h} viewBox={`0 0 ${w} ${h}`} className="shrink-0">
        <polyline
          fill="none"
          stroke={color}
          strokeWidth="2"
          strokeLinecap="square"
          strokeLinejoin="miter"
          points={points.join(' ')}
        />
      </svg>
      {/* Prozentzahl */}
      <span className={`text-[11px] font-mono font-bold ${data[data.length - 1] ? 'text-green-500/70' : 'text-red-500/70'}`}>{pct}%</span>
    </div>
  );
}

export default Sparkline;
