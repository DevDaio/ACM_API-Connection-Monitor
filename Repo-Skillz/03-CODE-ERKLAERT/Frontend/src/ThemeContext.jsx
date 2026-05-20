/* ═══════════════════════════════════════════════════════
 * 📦 THEME-CONTEXT (Farbschema-Verwaltung)
 *
 * 🎯 ZWECK:
 * Globaler React-Context für die drei Themes:
 * - LAVA RED (Default)
 * - HACKER GREEN
 * - VOID PURPLE
 *
 * 💡 KONZEPTE:
 * - createContext / useContext: Globaler State ohne Props
 * - CSS-Variablen via data-theme-Attribut auf <html>
 * - localStorage-Persistenz
 *
 * 📤 OUTPUT:
 * - { theme, setTheme, themes, current }
 *
 * 🎓 LERN-TIPP:
 * ThemeContext ist das einzige Beispiel für React Context
 * in diesem Projekt. Perfekt zum Lernen!
 * ═══════════════════════════════════════════════════════ */

import { createContext, useContext, useState, useEffect } from 'react';

const themes = [
    { id: 'lava', label: 'LAVA RED', color: '#ea580c' },
    { id: 'green', label: 'HACKER GREEN', color: '#16a34a' },
    { id: 'purple', label: 'VOID PURPLE', color: '#7c3aed' },
];

const ThemeContext = createContext();

function getInitial() {
    const t = localStorage.getItem('acm_theme') || 'lava';
    document.documentElement.setAttribute('data-theme', t);
    return t;
}

function ThemeProvider({ children }) {
    const [theme, setTheme] = useState(getInitial);

    useEffect(() => {
        localStorage.setItem('acm_theme', theme);
        document.documentElement.setAttribute('data-theme', theme);
    }, [theme]);

    const current = themes.find(t => t.id === theme) || themes[0];

    return (
        <ThemeContext.Provider value={{ theme, setTheme, themes, current }}>
            {children}
        </ThemeContext.Provider>
    );
}

function useTheme() {
    return useContext(ThemeContext);
}

export { ThemeProvider, useTheme, themes };
