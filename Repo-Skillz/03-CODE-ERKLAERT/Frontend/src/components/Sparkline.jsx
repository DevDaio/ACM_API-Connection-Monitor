/* ═══════════════════════════════════════════════════════
 * 📦 SPARKLINE (Mini-Status-Chart)
 *
 * 🎯 ZWECK:
 * Zeigt die letzten 30 Status-Checks als Liniendiagramm.
 * Oben = Running, Unten = Down.
 *
 * 💡 KONZEPTE:
 * - SVG Polyline (einfach & schnell)
 * - Y-Wert: true → oben (pad), false → unten (h-pad)
 * - Prozent-Anzeige (Running-Anteil)
 * - Farbe: grün (letzter = true), rot (letzter = false)
 *
 * 📥 INPUT:
 * - data: Vec<bool> (oder Array)
 * - width/height: SVG-Dimensionen
 *
 * 🎓 LERN-TIPP:
 * Das ist die kleinste Komponente im Projekt – perfekt
 * zum Verstehen von React + SVG.
 * ═══════════════════════════════════════════════════════ */

function Sparkline({ data, width = 70, height = 24 }) {
    if (!data || data.length < 2) return <span className="text-gray-500 text-[11px] font-mono">N/A</span>;

    const w = width;
    const h = height;
    const n = data.length;
    const stepX = (w - 2) / (n - 1);
    const pad = 1;

    const points = data.map((val, i) => {
        const x = pad + i * stepX;
        const y = val ? pad : h - pad;
        return `${x},${y}`;
    });

    const up = data.filter(Boolean).length;
    const pct = Math.round((up / n) * 100);
    const color = data[data.length - 1] ? '#22c55e' : '#ef4444';

    return (
        <div className="flex items-center gap-2">
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
            <span className={`text-[11px] font-mono font-bold ${data[data.length - 1] ? 'text-green-500/70' : 'text-red-500/70'}`}>{pct}%</span>
        </div>
    );
}

export default Sparkline;
