"use client";

import { useEffect } from "react";
import { track } from "@vercel/analytics";

/**
 * Los botones de descarga apuntan todos a GitHub Releases (siteConfig.GITHUB_RELEASES),
 * así que un único listener delegado en el documento cubre cada CTA sin convertir
 * las páginas de servidor en componentes de cliente.
 *
 * Evento: "download_dmg" con el path desde el que se hizo clic. Es la métrica
 * que el plan de lanzamiento llama "visitas → descarga": visitas las da el
 * pageview de <Analytics />, y esto da el numerador.
 */
export function AnalyticsEvents() {
  useEffect(() => {
    function onClick(event: MouseEvent) {
      const anchor = (event.target as Element | null)?.closest("a[href]");
      if (!anchor) return;
      const href = anchor.getAttribute("href") ?? "";
      if (!href.includes("github.com/baphomet444/cloakview/releases")) return;
      track("download_dmg", { from: window.location.pathname });
    }
    document.addEventListener("click", onClick);
    return () => document.removeEventListener("click", onClick);
  }, []);

  return null;
}
