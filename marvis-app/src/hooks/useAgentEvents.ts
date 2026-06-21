import { useState, useEffect, useRef } from "react";

export function useAgentEvents(onEvent: (event: any) => void) {
  const [isConnected, setIsConnected] = useState(false);
  const callbackRef = useRef(onEvent);
  callbackRef.current = onEvent;

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let mounted = true;

    async function setup() {
      try {
        const { listen } = await import("@tauri-apps/api/event");

        // Listen for agent events from Rust backend
        const unlistenFn = await listen<any>("agent-event", (event) => {
          if (mounted) {
            callbackRef.current(event.payload);
          }
        });

        if (mounted) {
          setIsConnected(true);
          console.log("✅ Tauri event listener connected");
        }

        unlisten = unlistenFn;
      } catch (e) {
        console.warn("⚠️ Not in Tauri environment, events disabled:", e);
        if (mounted) setIsConnected(false);
      }
    }

    setup();

    return () => {
      mounted = false;
      if (unlisten) unlisten();
    };
  }, []);

  return { isConnected };
}
