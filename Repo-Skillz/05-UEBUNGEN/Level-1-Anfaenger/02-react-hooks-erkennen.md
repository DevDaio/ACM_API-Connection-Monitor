# Übung: React Hooks erkennen

**Level:** 1 – Anfänger

## Aufgabe
Identifiziere die verwendeten Hooks in App.jsx.

## Frage 1
Welcher Hook wird verwendet, um die User-Session zu speichern?

## Frage 2
Was macht `useRef` in diesem Projekt?

a) Speichert DOM-Referenzen
b) Hält einen Wert ohne Re-Render
c) Erstellt ein neues Element

## Frage 3
Warum wird `useEffect` für das Polling verwendet?

## Lösung 1
`useState` + `useEffect` (für localStorage)

## Lösung 2
b) Hält einen mutable Wert, der keinen Re-Render auslöst

## Lösung 3
useEffect erlaubt uns, nach dem ersten Render ein Intervall zu starten und beim Unmount zu bereinigen (cleanup-Funktion).
