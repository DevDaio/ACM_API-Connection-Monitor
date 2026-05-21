// ─── Theme-Context: Farb-Themen (Lava, Hacker Green, Void Purple) ───
// Verwendet React Context, um das aktuelle Theme global bereitzustellen.
// Das `<html data-theme="...">`-Attribut wird gesetzt, damit index.css die CSS-Variablen wechseln kann.
import { createContext, useContext, useState, useEffect } from 'react';

// Definition der verfuegbaren Themes
const themes = [
  { id: 'lava', label: 'LAVA RED', color: '#ea580c' },
  { id: 'green', label: 'HACKER GREEN', color: '#16a34a' },
  { id: 'purple', label: 'VOID PURPLE', color: '#7c3aed' },
];

// Context erstellen (initialer Wert wird vom Provider gesetzt)
const ThemeContext = createContext();

// Initialen Theme-Wert aus localStorage laden und auf <html> setzen
function getInitial() {
  const t = localStorage.getItem('acm_theme') || 'lava';  // Fallback: lava
  document.documentElement.setAttribute('data-theme', t);
  return t;
}

// ThemeProvider: wrappt die App und stellt Theme-Funktionen bereit
function ThemeProvider({ children }) {
  const [theme, setTheme] = useState(getInitial);

  // Bei Theme-Wechsel: localStorage speichern + data-theme Attribut setzen
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

// useTheme: Hook zum Zugriff auf den Theme-Context
function useTheme() {
  return useContext(ThemeContext);
}

export { ThemeProvider, useTheme, themes };
