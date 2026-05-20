/* ═══════════════════════════════════════════════════════
 * 🎯 AUFGABE: State-Hub + Polling implementieren
 *
 * 📥 ERWARTETER INPUT:
 * - User-Interaktionen (Login, Endpoint-Hinzufügen, etc.)
 *
 * 📤 ERWARTETER OUTPUT:
 * - Voll funktionsfähige Monitor-App
 *
 * 💭 HINWEISE:
 * - useState für: user, endpoints, modal-Booleans
 * - useEffect für: localStorage, Polling, Log-Refresh
 * - useRef für modal-Check (Polling-Pause)
 * - api.js stellt fetch-Requests bereit
 * - ThemeProvider umschließt alles
 *
 * ✅ TEST:
 * npm run dev → Login → Dashboard sollte erscheinen
 * ═══════════════════════════════════════════════════════ */

// TODO: Importe
// useState, useEffect, useRef
// LandingPage, Dashboard, alle Modal-Komponenten
// ThemeProvider, api

function App() {
    // ─── STATE ───

    // TODO: user-State (aus localStorage initialisieren)
    // TODO: endpoints-State (Array)
    // TODO: mainSwitch-State (Boolean)

    // TODO: Modal-Open-States (alle Booleans)
    // showCreateAccount, showAddEndpoint, showSetIntervall,
    // showDeleteConfirm, showAccountSettings, showLog,
    // deleteIndex, selectedEndpoint, logEntries,
    // showEditUrl, editUrlValue

    // ─── EFFECTS ───

    // TODO: localStorage-Persistenz für user
    // useEffect: user ändert → localStorage setzen/remove

    // TODO: modalRef für Polling-Pause
    // useRef + useEffect

    // TODO: Initialer Daten-Load
    // useEffect: wenn user da → api.getHome()

    // TODO: Polling-Loop
    // useEffect: tick (1s) + poll (10s) Intervalle
    // poll nur wenn kein Modal offen (anyModalRef)

    // ─── HELPERS ───

    // TODO: fmtDuration(secs) → lesbare Uptime
    // TODO: fmtInterval(secs) → lesbares Intervall
    // TODO: mapEndpoints(data) → API-Daten in Frontend-Format

    // ─── EVENT HANDLER ───

    // TODO: handleLogin(email, password)
    // TODO: handleCreateAccount(email, password)
    // TODO: handleAddEndpoint(url, seconds)
    // TODO: handleShowLog(i)
    // TODO: handleSetIntervall(i)
    // TODO: handleEditUrl(i)
    // TODO: handleSaveUrl()
    // TODO: handleRemove(i) / confirmDelete()
    // TODO: handleToggleEndpoint(i)
    // TODO: handleChangePassword / handleChangeEmail
    // TODO: handleDeleteAccount / handleLogout

    // ─── RENDER ───

    // TODO: if (!user) → LandingPage + CreateAccountModal
    // TODO: else → Dashboard + alle Modals
}

export default App;
