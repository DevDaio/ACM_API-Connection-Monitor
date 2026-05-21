// ─── ESLint-Flat-Config (ESLint 10) ───
// defineConfig erzeugt ein Array von Config-Objekten (flat config statt .eslintrc)
import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'       // Regeln für React Hooks (rules-of-hooks, exhaustive-deps)
import reactRefresh from 'eslint-plugin-react-refresh'   // Erzwingt, dass Komponenten exportiert werden (für Fast-Refresh)
import { defineConfig, globalIgnores } from 'eslint/config'

export default defineConfig([
  globalIgnores(['dist']),  // dist/ ignoriert (Build-Ausgabe)
  {
    files: ['**/*.{js,jsx}'],
    extends: [
      js.configs.recommended,           // Standard-JS-Regeln
      reactHooks.configs.flat.recommended, // react-hooks Regeln
      reactRefresh.configs.vite,        // react-refresh Regeln
    ],
    languageOptions: {
      globals: globals.browser,   // Browser-Globals (window, fetch, etc.)
      parserOptions: { ecmaFeatures: { jsx: true } },  // JSX-Parsing aktivieren
    },
  },
])
