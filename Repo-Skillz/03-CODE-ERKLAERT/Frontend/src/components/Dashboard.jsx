/* ═══════════════════════════════════════════════════════
 * 📦 DASHBOARD (Hauptansicht)
 *
 * 🎯 ZWECK:
 * Hauptbildschirm nach dem Login. Zeigt:
 * - Header mit Status, Theme-Switcher, Exit
 * - MAIN_SWITCH (Toggle alle Endpoints)
 * - Tabelle: EndpointCard × N
 * - ADD_ENDPOINT-Button
 *
 * 💡 KONZEPTE:
 * - Reine Präsentationskomponente (kein State)
 * - Erhält alles per Props
 * - Emittiert Events (Callbacks nach oben)
 * ═══════════════════════════════════════════════════════ */

import EndpointCard from './EndpointCard';
import ThemeSwitcher from './ThemeSwitcher';

function Dashboard({
    endpoints,
    mainSwitch,
    onToggleMainSwitch,
    onRemove,
    onToggleEndpoint,
    onSetIntervall,
    onShowLog,
    onEditUrl,
    onAddEndpoint,
    onLogout,
    onAccountSettings,
}) {
    return (
        <div className="min-h-screen bg-gray-950 text-white flex flex-col relative overflow-hidden"
            style={{background: 'radial-gradient(ellipse at 50% 0%, #1a0800 0%, #030712 60%)'}}>
            <div className="absolute inset-0 pointer-events-none opacity-[0.03]"
                style={{backgroundImage: 'repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(234,88,12,0.2) 2px, rgba(234,88,12,0.2) 4px)'}} />

            <header className="border-b ac-bd px-8 py-4 flex items-center justify-between shrink-0 bg-gray-950/80 backdrop-blur relative" style={{ zIndex: 100 }}>
                <div className="flex items-center gap-5">
                    <svg width="32" height="32" viewBox="0 0 24 24">
                        <circle cx="12" cy="12" r="10" fill="none" stroke="#ea580c" strokeWidth="1.5" opacity="0.6" />
                        <circle cx="12" cy="12" r="6" fill="none" stroke="#ef4444" strokeWidth="1" opacity="0.8" />
                        <circle cx="12" cy="12" r="2" fill="#ea580c" />
                    </svg>
                    <div>
                        <div className="text-sm font-black tracking-[0.3em] uppercase">
                            <span className="text-orange-600">ACM</span>
                            <span className="text-gray-300 ml-3 font-mono text-xs">v1.0</span>
                        </div>
                        <div className="text-[10px] font-mono text-gray-300 tracking-[0.25em]">NODE_MONITOR // ACTIVE</div>
                    </div>
                </div>
                <div className="flex items-center gap-5">
                    <span className="text-xs font-mono text-gray-300">{endpoints.length} endpoints</span>
                    <div className="h-5 w-px ac-bg/20" />
                    <ThemeSwitcher />
                    <div className="h-5 w-px ac-bg/20" />
                    <button onClick={onAccountSettings} className="text-gray-300 ac-tx-hover text-lg" title="Settings">{'\u2699'}</button>
                    <button onClick={onLogout} className="text-gray-300 hover:text-red-400 text-xs font-mono tracking-wider uppercase transition-colors">Exit</button>
                </div>
            </header>

            <div className="flex-1 w-full px-8 py-6 relative z-10">
                <div className="flex items-center justify-between mb-6 border ac-bd bg-gray-900/50 px-6 py-4 relative"
                    style={{boxShadow: 'inset 0 0 20px rgba(234,88,12,0.05)'}}>
                    <div className="flex items-center gap-4">
                        <span className={`inline-block w-2 h-2 ${mainSwitch ? 'bg-green-500 shadow-[0_0_10px_rgba(34,197,94,0.8)]' : 'bg-gray-700'}`} />
                        <span className="text-sm font-mono ac-tx-hover tracking-widest uppercase font-bold">MAIN_SWITCH</span>
                        <span className="text-[10px] font-mono text-gray-300 tracking-wider">TOGGLE_ALL_JOBS</span>
                    </div>
                    <div className="flex items-center gap-3">
                        <button onClick={onToggleMainSwitch}
                            className={`relative w-14 h-1.5 transition-all rounded-full ${mainSwitch ? 'bg-orange-600 shadow-[0_0_10px_rgba(234,88,12,0.5)]' : 'bg-gray-800'}`}>
                            <span className={`absolute -top-[5px] w-3.5 h-3.5 rounded-full transition-all border-2 ${
                                mainSwitch
                                    ? 'right-0 bg-orange-500 border-orange-400 shadow-[0_0_8px_rgba(234,88,12,0.6)]'
                                    : 'left-0 bg-gray-600 border-gray-600'
                            }`} />
                        </button>
                        <span className={`text-[10px] font-mono tracking-wider ${
                            mainSwitch ? 'text-green-400' : 'text-gray-300'
                        }`}>{mainSwitch ? 'ARMED' : 'OFF'}</span>
                    </div>
                </div>

                <div className="border ac-bd relative"
                    style={{boxShadow: 'inset 0 0 30px rgba(234,88,12,0.03)'}}>
                    <div className="bg-gray-900/80 border-b ac-bd px-5 py-3 flex items-center gap-4">
                        <span className="text-[10px] font-mono ac-tx/50 tracking-[0.3em] font-bold">ENDPOINT_LOG</span>
                        <div className="h-4 w-px ac-bg/10" />
                        <span className="text-[10px] font-mono text-gray-300">{endpoints.length} RECORDS</span>
                    </div>

                    <div className="overflow-x-auto">
                        <table className="w-full text-left">
                            <thead>
                                <tr className="text-xs font-mono text-gray-300 uppercase tracking-widest border-b ac-bd">
                                    <th className="py-3 px-4 font-normal">STATUS</th>
                                    <th className="py-3 px-4 font-normal">TARGET</th>
                                    <th className="py-3 px-4 font-normal">UPTIME</th>
                                    <th className="py-3 px-4 font-normal">SCAN_INT</th>
                                    <th className="py-3 px-4 font-normal">SIGNAL</th>
                                    <th className="py-3 px-4 font-normal">LAST_CHG</th>
                                    <th className="py-3 px-4 font-normal text-right">CTRL</th>
                                </tr>
                            </thead>
                            <tbody>
                                {endpoints.length === 0 ? (
                                    <tr><td colSpan="7" className="py-20 text-gray-300 text-center font-mono text-sm">[ NO_ENDPOINTS // AWAITING_INPUT ]</td></tr>
                                ) : (
                                    endpoints.map((ep, i) => (
                                        <EndpointCard
                                            key={ep.endpointid}
                                            endpoint={ep}
                                            onRemove={() => onRemove(i)}
                                            onToggle={() => onToggleEndpoint(i)}
                                            onSetIntervall={() => onSetIntervall(i)}
                                            onShowLog={() => onShowLog(i)}
                                            onEditUrl={() => onEditUrl(i)}
                                        />
                                    ))
                                )}
                            </tbody>
                        </table>
                    </div>
                </div>

                <button onClick={onAddEndpoint}
                    className="mt-5 w-full border border-dashed ac-bd ac-bd-hover ac-tx/60 ac-tx-hover py-4 uppercase tracking-[0.3em] font-mono text-sm font-bold transition-all hover:shadow-[0_0_25px_rgba(234,88,12,0.15)] bg-gray-950/50">
                    + ADD_ENDPOINT
                </button>

                <div className="mt-5 flex items-center gap-5 text-[10px] font-mono text-gray-300 tracking-[0.25em]">
                    <span>SYS::ONLINE</span>
                    <span className="w-1.5 h-1.5 ac-bg/30" />
                    <span>MONITOR::ACTIVE</span>
                    <span className="w-1.5 h-1.5 ac-bg/30" />
                    <span>{new Date().toLocaleTimeString()} UTC</span>
                </div>
            </div>
        </div>
    );
}

export default Dashboard;
