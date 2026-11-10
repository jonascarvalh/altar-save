import { useEffect } from "react";
import { createHashRouter, RouterProvider } from "react-router";
import { AppLayout } from "./layout/AppLayout";
import { LibraryPage } from "@/features/library/LibraryPage";
import { CloudPage } from "@/features/cloud/CloudPage";
import { AutomationsPage } from "@/features/automations/AutomationsPage";
import { ActivityPage } from "@/features/activity/ActivityPage";
import { onSyncEvent } from "@/lib/api/events";
import { errorMessage } from "@/lib/api/errors";
import { useSettings } from "@/stores/settingsStore";
import { useSync } from "@/stores/syncStore";
import { toast } from "@/components/ui/Toaster";

const router = createHashRouter([
  {
    element: <AppLayout />,
    children: [
      { index: true, element: <LibraryPage /> },
      { path: "automations", element: <AutomationsPage /> },
      { path: "cloud", element: <CloudPage /> },
      { path: "activity", element: <ActivityPage /> },
    ],
  },
]);

export function App() {
  useBootstrap();
  return <RouterProvider router={router} />;
}

/** Carrega o estado inicial e assina os eventos de sync do backend. */
function useBootstrap() {
  useEffect(() => {
    const { load } = useSettings.getState();
    const { loadLibrary, loadWatchStatus, handleEvent } = useSync.getState();

    load().catch((e) => toast.error("Falha ao carregar configurações", errorMessage(e)));
    loadLibrary().catch((e) => toast.error("Falha ao ler os saves", errorMessage(e)));
    loadWatchStatus().catch(() => {});

    const unlisten = onSyncEvent((event) => {
      handleEvent(event);
      if (event.type === "finished" && event.summary.error) {
        toast.error("Falha na sincronização", event.summary.error);
      }
    });
    // Arquivos podem mudar com o app em segundo plano: relê a biblioteca ao voltar o foco.
    const onFocus = () => void loadLibrary().catch(() => {});
    window.addEventListener("focus", onFocus);

    return () => {
      window.removeEventListener("focus", onFocus);
      void unlisten.then((fn) => fn());
    };
  }, []);
}
