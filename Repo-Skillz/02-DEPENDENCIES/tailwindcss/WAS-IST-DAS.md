# Tailwind CSS v4

**Was macht es?** Utility-First CSS-Framework. Styling über Klassen direkt im HTML/JSX.

**Warum?** Kein CSS schreiben, konsistentes Design-System, extrem klein im Build.

**Wo?** Alle .jsx-Dateien in `Frontend/ACM_Frontend/src/`

**Besonderheiten in v4:**
- Keine tailwind.config.js nötig
- CSS-Import statt PostCSS: `@import "tailwindcss"`
- Vite-Plugin: `@tailwindcss/vite`

**Theme-System:**
CSS-Variablen pro Theme überschreiben die Tailwind-Farben:
```css
:root[data-theme="lava"] {
    --color-orange-600: #ea580c;
}
```
Danach ist `ac-bg` (accent-bg) überall nutzbar.

**Mini-Tutorial:**
```bash
npm install tailwindcss @tailwindcss/vite
```
```css
/* index.css */
@import "tailwindcss";
```
```js
// vite.config.js
import tailwindcss from '@tailwindcss/vite';
export default defineConfig({ plugins: [tailwindcss()] });
```
