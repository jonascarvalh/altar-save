import { useEffect, useMemo, useState, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Square, X } from "lucide-react";
import { cn } from "@/lib/format";

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
        <img src="/logo.svg" alt="" className="pointer-events-none size-5" />
        <span data-tauri-drag-region className="text-[13px] font-semibold tracking-tight text-fg">
          save<span className="text-accent-gradient">nuvem</span>
        </span>
        <span className="rounded bg-white/5 px-1.5 py-0.5 text-[10px] font-medium text-subtle">protótipo</span>
      </div>
      <div data-tauri-drag-region className="h-full flex-1" />
      <div className="flex h-full">
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
        danger ? "hover:bg-[#e81123] hover:text-white" : "hover:bg-white/8 hover:text-fg",
      )}
    >
      {children}
    </button>
  );
}
