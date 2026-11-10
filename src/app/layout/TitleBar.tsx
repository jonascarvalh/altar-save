import { useEffect, useMemo, useState, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Moon, Square, Sun, X } from "lucide-react";
import { cn } from "@/lib/format";
import { useTheme } from "@/stores/themeStore";

/** Barra de título customizada (a janela não usa a decoração nativa). */
export function TitleBar() {
  const appWindow = useMemo(() => getCurrentWindow(), []);
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    void appWindow.isMaximized().then(setMaximized);
    const unlisten = appWindow.onResized(() => {
      void appWindow.isMaximized().then(setMaximized);
    });
    return () => {
      void unlisten.then((fn) => fn());
    };
  }, [appWindow]);

  return (
    <div data-tauri-drag-region className="flex h-10 shrink-0 items-center border-b border-line bg-surface select-none">
      <div data-tauri-drag-region className="flex items-center gap-2.5 pl-4">
        <img src="/logo.svg" alt="" className="pointer-events-none h-5 w-auto" />
        <span data-tauri-drag-region className="text-[14px] tracking-[-0.04em] text-fg">
          <span className="font-bold">altar</span>
          <span className="text-accent-gradient font-light">save</span>
        </span>
        <span className="rounded bg-tint/5 px-1.5 py-0.5 text-[10px] font-medium text-subtle">protótipo</span>
      </div>
      <div data-tauri-drag-region className="h-full flex-1" />
      <div className="flex h-full">
        <ThemeButton />
        <WindowButton label="Minimizar" onClick={() => appWindow.minimize()}>
          <Minus className="size-4" />
        </WindowButton>
        <WindowButton label={maximized ? "Restaurar" : "Maximizar"} onClick={() => appWindow.toggleMaximize()}>
          {maximized ? <Copy className="size-3.5 -scale-x-100" /> : <Square className="size-3.5" />}
        </WindowButton>
        <WindowButton label="Fechar" danger onClick={() => appWindow.close()}>
          <X className="size-4" />
        </WindowButton>
      </div>
    </div>
  );
}

/** Alterna entre claro e escuro. Até o primeiro clique, o tema segue o do sistema. */
function ThemeButton() {
  const resolved = useTheme((s) => s.resolved);
  const setMode = useTheme((s) => s.setMode);
  const isLight = resolved === "light";

  return (
    <WindowButton
      label={isLight ? "Mudar para o tema escuro" : "Mudar para o tema claro"}
      onClick={() => setMode(isLight ? "dark" : "light")}
    >
      {isLight ? <Sun className="size-3.5" /> : <Moon className="size-3.5" />}
    </WindowButton>
  );
}

function WindowButton({
  label,
  danger,
  onClick,
  children,
}: {
  label: string;
  danger?: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={onClick}
      className={cn(
        "grid h-full w-12 place-items-center text-muted transition-colors",
        danger ? "hover:bg-[#e81123] hover:text-white" : "hover:bg-tint/8 hover:text-fg",
      )}
    >
      {children}
    </button>
  );
}
