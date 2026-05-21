// ─── ThemeSwitcher – Theme-Umschalter ───
// Dropdown-Button im Header zum Wechseln zwischen lava / hacker green / void purple.
import { useRef, useEffect, useState } from 'react';
import { useTheme } from '../ThemeContext';

function ThemeSwitcher() {
  const { theme, setTheme, themes, current } = useTheme();
  const [open, setOpen] = useState(false);
  const ref = useRef(null);

  // Schliesst das Dropdown bei Klick ausserhalb (Click-Away-Handler)
  useEffect(() => {
    if (!open) return;
    function handle(e) {
      if (ref.current && !ref.current.contains(e.target)) setOpen(false);
    }
    document.addEventListener('mousedown', handle);
    return () => document.removeEventListener('mousedown', handle);
  }, [open]);

  return (
    <div className="relative" ref={ref}>
      {/* Button: zeigt current Theme an */}
      <button onClick={() => setOpen(!open)}
        className="flex items-center gap-2 text-gray-300 hover:text-white text-xs font-mono tracking-wider uppercase transition-colors">
        {/* Farbpunkt mit Glow */}
        <span className="inline-block w-2 h-2"
          style={{ backgroundColor: current.color, boxShadow: `0 0 6px ${current.color}88` }} />
        {current.label}
      </button>

      {/* Dropdown-Menü (nur sichtbar wenn open=true) */}
      {open && (
        <div className="absolute right-0 top-full mt-2 border border-gray-700 bg-gray-900 min-w-[160px]"
          style={{ zIndex: 9999 }}>
          {themes.map(t => (
            <button key={t.id} onClick={() => { setTheme(t.id); setOpen(false); }}
              className={`w-full text-left px-4 py-2.5 text-xs font-mono tracking-wider uppercase transition-colors flex items-center gap-3 ${
                theme === t.id ? 'text-white bg-gray-700' : 'text-gray-300 hover:text-white hover:bg-gray-800'
              }`}>
              <span className="inline-block w-2 h-2"
                style={{ backgroundColor: t.color, boxShadow: `0 0 6px ${t.color}88` }} />
              {t.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

export default ThemeSwitcher;
