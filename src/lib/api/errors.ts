/** Erro vindo do backend (`AppError` serializado como `{ kind, message }`). */
export class AppError extends Error {
  readonly kind: string;

  constructor(kind: string, message: string) {
    super(message);
    this.name = "AppError";
    this.kind = kind;
  }
}

export function toAppError(error: unknown): AppError {
  if (error instanceof AppError) return error;
  if (error && typeof error === "object" && "message" in error) {
    const { kind, message } = error as { kind?: unknown; message?: unknown };
    return new AppError(typeof kind === "string" ? kind : "internal", String(message));
  }
  return new AppError("internal", typeof error === "string" ? error : "Erro inesperado.");
}

export function errorMessage(error: unknown): string {
  return toAppError(error).message;
}
