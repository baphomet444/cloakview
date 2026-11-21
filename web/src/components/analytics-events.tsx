"use client";

import { useEffect } from "react";
import { track } from "@vercel/analytics";
import { eventForHref } from "@/lib/analytics-targets";

/**
 * Los CTA que nos importan son enlaces (`github.com/.../releases` para el DMG,
 * `buy.polar.sh` para el checkout de Pro), así que un único listener delegado en el
 * documento cubre cada uno sin convertir las páginas de servidor en componentes de
 * cliente. La clasificación vive en `lib/analytics-targets.ts`, que tiene sus tests.
 *
 * Eventos, todos con el path desde el que se hizo clic:
 * - "download_dmg": el numerador de "visitas → descarga" (el pageview de <Analytics />
 *   da las visitas). Cuenta el clic, no una descarga terminada.
 * - "checkout_click": el CTA de compra de CloakView Pro (siteConfig.PRO_CHECKOUT_URL).
 *   Es la intención de compra, no una venta: la venta la cuenta Polar.
 *
 * `track` manda un beacon y no bloquea la navegación: el destino se abre igual.
 */
export function AnalyticsEvents() {
  useEffect(() => {
    function onClick(event: MouseEvent) {
      const anchor = (event.target as Element | null)?.closest("a[href]");
      if (!anchor) return;
      const name = eventForHref(anchor.getAttribute("href"));
      if (name) track(name, { from: window.location.pathname });
    }
    document.addEventListener("click", onClick);
    return () => document.removeEventListener("click", onClick);
  }, []);

  return null;
}
