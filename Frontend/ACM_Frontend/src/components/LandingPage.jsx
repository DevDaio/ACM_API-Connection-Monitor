// ─── LandingPage – Login-Bildschirm ───
// Terminal-inspiriertes Login-Formular mit Scanlines, Gitter und HUD-Elementen.
// Bei erfolgreichem Login wird die App an das Dashboard uebergeben.
import { useState, useRef } from 'react';

function LandingPage({ onLogin, onCreateAccount }) {
  // useRef statt useState für Input-Werte (vermeidet Re-Renders bei jedem Tastendruck)
  const emailRef = useRef(null);
  const passwordRef = useRef(null);
  const [error, setError] = useState('');       // Fehlermeldung (z. B. falsches Passwort)
  const [loading, setLoading] = useState(false); // Lade-Zustand für Button-Text + Disabled

  async function handleSubmit(e) {
    e.preventDefault();  // Verhindert Seiten-Neuladen bei Form-Submit
    setError('');
    setLoading(true);
    try {
      await onLogin(emailRef.current.value, passwordRef.current.value);
    } catch (err) {
      setError(err.message);  // Fehler von der API anzeigen
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="min-h-screen bg-gray-950 text-white flex flex-col relative overflow-hidden"
      style={{background: 'radial-gradient(ellipse at 50% 0%, #1a0800 0%, #030712 60%)'}}>

      {/* Scanlines: horizontale Linien für CRT-Effekt */}
      <div className="absolute inset-0 pointer-events-none opacity-[0.04]"
        style={{backgroundImage: 'repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(234,88,12,0.3) 2px, rgba(234,88,12,0.3) 4px)'}} />
      {/* Grid: punktförmiges Raster */}
      <div className="absolute inset-0 pointer-events-none opacity-[0.06]"
        style={{backgroundImage: 'linear-gradient(rgba(234,88,12,0.5) 1px, transparent 1px), linear-gradient(90deg, rgba(234,88,12,0.5) 1px, transparent 1px)', backgroundSize: '60px 60px'}} />

      <div className="flex-1 flex flex-col items-center justify-center px-4 py-16 relative z-10">
        {/* ─── Eck-Klammern ─── */}
        {/* Dekorative Elemente: 4 L-förmige Linien in den Ecken */}
        <div className="absolute top-8 left-8 w-16 h-16 border-t-2 border-l-2 ac-bd" />
        <div className="absolute top-8 right-8 w-16 h-16 border-t-2 border-r-2 ac-bd" />
        <div className="absolute bottom-8 left-8 w-16 h-16 border-b-2 border-l-2 ac-bd" />
        <div className="absolute bottom-8 right-8 w-16 h-16 border-b-2 border-r-2 ac-bd" />

        {/* ─── HUD-Ring-Logo ─── */}
        {/* Animiertes SVG mit Zielscheiben-Optik + "ACM"-Text */}
        <div className="relative mb-10">
          <svg width="100" height="100" viewBox="0 0 100 100" className="animate-pulse">
            <circle cx="50" cy="50" r="45" fill="none" stroke="#ea580c" strokeWidth="1" opacity="0.3" />
            <circle cx="50" cy="50" r="38" fill="none" stroke="#ef4444" strokeWidth="0.5" opacity="0.5" />
            <circle cx="50" cy="50" r="30" fill="none" stroke="#ea580c" strokeWidth="2" opacity="0.8" />
            <line x1="20" y1="50" x2="35" y2="50" stroke="#ea580c" strokeWidth="2" />
            <line x1="65" y1="50" x2="80" y2="50" stroke="#ea580c" strokeWidth="2" />
            <line x1="50" y1="20" x2="50" y2="35" stroke="#ea580c" strokeWidth="2" />
            <line x1="50" y1="65" x2="50" y2="80" stroke="#ea580c" strokeWidth="2" />
            <text x="50" y="55" textAnchor="middle" fill="#ea580c" fontSize="20" fontFamily="monospace" fontWeight="bold">ACM</text>
          </svg>
          {/* Unterer Strich unter dem Logo */}
          <div className="absolute -bottom-2 left-1/2 -translate-x-1/2 w-24 h-px bg-gradient-to-r from-transparent via-orange-600 to-transparent" />
        </div>

        {/* ─── Titel ─── */}
        <div className="text-center mb-10">
          <div className="text-[10px] font-mono ac-tx/60 tracking-[0.3em] mb-2">SYS.INIT // NODE.ACTIVE</div>
          <h1 className="text-5xl md:text-7xl font-black tracking-tighter text-white uppercase">
            <span className="text-orange-600 [text-shadow:0_0_30px_rgba(234,88,12,0.5)]">ACM</span>
            <br />
            <span className="text-2xl md:text-3xl tracking-[0.15em] text-gray-300">API CONNECTION</span>
            <br />
            <span className="text-2xl md:text-3xl tracking-[0.15em] text-red-500 [text-shadow:0_0_20px_rgba(239,68,68,0.4)]">MONITOR</span>
          </h1>
          <div className="flex items-center justify-center gap-3 mt-4">
            <div className="h-px w-12 bg-gradient-to-r from-transparent to-orange-600/50" />
            <span className="text-[10px] font-mono text-gray-300 tracking-[0.2em]">v1.0 // REALTIME</span>
            <div className="h-px w-12 bg-gradient-to-r from-orange-600/50 to-transparent" />
          </div>
        </div>

        {/* ─── Login-Formular (Terminal-Design) ─── */}
        <div className="w-full max-w-sm border ac-bd bg-gray-950/90 relative"
          style={{boxShadow: '0 0 30px rgba(234,88,12,0.1), inset 0 0 30px rgba(234,88,12,0.03)'}}>

          {/* Titel-Leiste mit roten/orangen Punkten (MacOS-Terminal-Stil) */}
          <div className="flex items-center gap-2 px-4 py-2 border-b ac-bd bg-gray-900/80">
            <span className="w-2.5 h-2.5 bg-red-500" />
            <span className="w-2.5 h-2.5 bg-orange-500" />
            <span className="w-2.5 h-2.5 bg-gray-700" />
            <span className="text-[10px] font-mono text-gray-300 ml-2 tracking-wider">ACM::ACCESS_TERMINAL</span>
          </div>

          <div className="p-6">
            <div className="text-[10px] font-mono ac-tx/50 mb-4">AUTH_REQUIRED // ENTER_CREDENTIALS</div>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <label className="block text-[10px] font-mono text-gray-300 mb-1 tracking-widest">USER_IDENT // EMAIL</label>
                <input ref={emailRef} type="email" required
                  className="w-full bg-gray-900 border border-gray-800 px-3 py-2.5 text-white font-mono text-sm focus:outline-none focus:shadow-[0_0_10px_rgba(234,88,12,0.15)] transition-all"
                  placeholder="deine@email.de" />
              </div>
              <div>
                <label className="block text-[10px] font-mono text-gray-300 mb-1 tracking-widest">PASSKEY // PASSWORD</label>
                <input ref={passwordRef} type="password" required
                  className="w-full bg-gray-900 border border-gray-800 px-3 py-2.5 text-white font-mono text-sm focus:outline-none focus:shadow-[0_0_10px_rgba(234,88,12,0.15)] transition-all"
                  placeholder="••••••••" />
              </div>
              {error && <p className="text-red-500 text-xs font-mono border-l-2 border-red-500 pl-2">{error}</p>}
              <button type="submit" disabled={loading}
                className="w-full ac-bg hover:bg-orange-600/80 disabled:opacity-40 text-white font-mono font-bold py-2.5 uppercase tracking-widest text-xs transition-all"
                style={{boxShadow: loading ? 'none' : '0 0 15px rgba(234,88,12,0.3)'}}>
                {loading ? '>_ AUTHENTICATING...' : '>_ LOGIN'}
              </button>
            </form>
            {/* Link zum Account-Erstellen */}
            <button onClick={onCreateAccount} className="mt-4 text-red-500/60 hover:text-red-400 text-[10px] font-mono text-center w-full tracking-wider">
              [ CREATE_ACCOUNT ]
            </button>
          </div>
        </div>

        <div className="mt-8 text-[9px] font-mono text-gray-300 tracking-[0.3em]">SYSTEM_STANDBY // AWAITING_AUTH</div>
      </div>
    </div>
  );
}

export default LandingPage;
