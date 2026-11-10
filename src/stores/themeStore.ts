import { create } from "zustand";

export type ThemeMode = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";

/** Mesma chave lida pelo script inline do index.html (que aplica o tema antes do primeiro paint). */
const STORAGE_KEY = "altar-save:theme";
const systemQuery = window.matchMedia("(prefers-color-scheme: light)");

function readMode(): ThemeMode {
  try {
    const value = localStorage.getItem(STORAGE_KEY);
    if (value === "light" || value === "dark" || value === "system") return value;
  } catch {
    // Armazenamento indisponível: segue o sistema.
  }
  return "system";
}

function resolve(mode: ThemeMode): ResolvedTheme {
  if (mode !== "system") return mode;
  return systemQuery.matches ? "light" : "dark";
}

function apply(theme: ResolvedTheme) {
  document.documentElement.dataset.theme = theme;
}

interface ThemeState {
  mode: ThemeMode;
  resolved: ResolvedTheme;
  setMode: (mode: ThemeMode) => void;
}

export const useTheme = create<ThemeState>((set, get) => {
  const mode = readMode();
  const resolved = resolve(mode);
  apply(resolved);

  // No modo "sistema", acompanha a troca de tema do SO com o app aberto.
  systemQuery.addEventListener("change", () => {
    if (get().mode !== "system") return;
    const next = resolve("system");
    apply(next);
    set({ resolved: next });
  });

  return {
    mode,
    resolved,
    setMode: (mode) => {
      try {
        localStorage.setItem(STORAGE_KEY, mode);
      } catch {
        // Sem persistência, o tema vale só para esta sessão.
      }
      const next = resolve(mode);
      apply(next);
      set({ mode, resolved: next });
    },
  };
});
