import { useId, useState, type InputHTMLAttributes, type ReactNode } from "react";
import { Eye, EyeOff } from "lucide-react";
import { cn } from "@/lib/format";

export interface TextFieldProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "onChange"> {
  label: string;
  hint?: ReactNode;
  error?: string | null;
  secret?: boolean;
  mono?: boolean;
  trailing?: ReactNode;
  onValueChange?: (value: string) => void;
}

export function TextField({
  label,
  hint,
  error,
  secret,
  mono,
  trailing,
  className,
  onValueChange,
  ...props
}: TextFieldProps) {
  const id = useId();
  const [revealed, setRevealed] = useState(false);

  return (
    <div className={cn("flex flex-col gap-1.5", className)}>
      <label htmlFor={id} className="text-xs font-medium text-muted">
        {label}
        {props.required && <span className="ml-0.5 text-accent">*</span>}
      </label>
      <div
        className={cn(
          "flex h-10 items-center rounded-lg border bg-surface-2 transition-colors",
          "focus-within:border-accent/60 focus-within:bg-surface-3",
          error ? "border-danger/60" : "border-line-strong",
        )}
      >
        <input
          id={id}
          type={secret && !revealed ? "password" : "text"}
          spellCheck={false}
          autoComplete="off"
          className={cn(
            "h-full min-w-0 flex-1 bg-transparent px-3 text-[13px] text-fg outline-none placeholder:text-subtle",
            mono && "font-mono text-xs",
          )}
          onChange={(e) => onValueChange?.(e.target.value)}
          {...props}
        />
        {secret && (
          <button
            type="button"
            tabIndex={-1}
            onClick={() => setRevealed((v) => !v)}
            className="grid size-9 place-items-center text-subtle hover:text-fg"
            aria-label={revealed ? "Ocultar" : "Mostrar"}
          >
            {revealed ? <EyeOff className="size-4" /> : <Eye className="size-4" />}
          </button>
        )}
        {trailing}
      </div>
      {error ? (
        <p className="text-xs text-danger">{error}</p>
      ) : (
        hint && <p className="text-xs text-subtle">{hint}</p>
      )}
    </div>
  );
}
