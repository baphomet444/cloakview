// Qué CTA es cada enlace, para los eventos de analytics del sitio.
// Sin imports: `tests/analytics-targets.test.ts` importa este módulo con node:test.

/** Todos los CTA de descarga apuntan a la descarga firmada (siteConfig.DOWNLOAD_URL). */
export const DOWNLOAD_URL = "/api/dl";

/** El enlace anterior (siteConfig.LEGACY_DOWNLOAD_URL), que redirige a DOWNLOAD_URL. */
export const LEGACY_DOWNLOAD_URL = "/dl/mac";

/** Se mantiene por compatibilidad: las releases de GitHub siguen contando como descarga. */
export const GITHUB_RELEASES = "github.com/baphomet444/cloakview/releases";

/** El checkout de CloakView Pro es el link de producto de Polar (siteConfig.PRO_CHECKOUT_URL). */
export const POLAR_CHECKOUT = "buy.polar.sh";

export type AnalyticsEvent = "download_dmg" | "checkout_click";

/**
 * El evento que corresponde a un `href`, o null si el clic no es de ninguno de los
 * dos CTAs que nos importan. Se compara el href y no el texto del botón, para que
 * agregar un CTA nuevo que apunte al mismo destino quede medido sin tocar nada.
 */
export function eventForHref(href: string | null | undefined): AnalyticsEvent | null {
  if (!href) return null;
  if (href.includes(DOWNLOAD_URL)) return "download_dmg";
  if (href.includes(LEGACY_DOWNLOAD_URL)) return "download_dmg";
  if (href.includes(GITHUB_RELEASES)) return "download_dmg";
  if (href.includes(POLAR_CHECKOUT)) return "checkout_click";
  return null;
}
