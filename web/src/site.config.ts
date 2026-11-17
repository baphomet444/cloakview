// Configuración centralizada de URLs externas y datos legales de CloakView.
// Todas las referencias externas del sitio DEBEN pasar por este archivo.
// Mientras GITHUB_RELEASES/GITHUB_REPO contengan "OWNER", el repo aún no
// existe: la UI debe mostrar un estado "próximamente" explícito, nunca un
// enlace roto ni un error silencioso.

export const GITHUB_API = "https://api.github.com";
export const OPENAI_API_KEYS = "https://platform.openai.com/api-keys";
export const OPENAI_STATUS = "https://status.openai.com/";

export const siteConfig = {
  name: "CloakView",
  // Dominios con marca libre; .com queda explícitamente excluido (brief).
  url: "https://getcloakview.com",
  description: {
    en: "Your private meeting copilot. Your API key, your control.",
    es: "Tu copiloto privado de reuniones. Tu clave, tu control.",
  },
  // El eslogan oficial, palabra por palabra (el del hero y el de las imágenes para redes).
  // Va en el schema y en cualquier lugar donde una frase corta valga más que el resumen.
  tagline: {
    en: "A meeting copilot only you can see",
    es: "Un copiloto de reuniones que solo tú puedes ver",
  },

  // --- Placeholders pendientes de valores reales ---
  GITHUB_RELEASES: "https://github.com/baphomet444/cloakview/releases/latest",
  GITHUB_REPO: "https://github.com/baphomet444/cloakview",
  // Perfil limpio, sin los parámetros del QR con el que se compartió.
  INSTAGRAM_URL: "https://www.instagram.com/cloak_view",
  SUPPORT_EMAIL: "support@getcloakview.com",
  LEGAL_ENTITY: "PLACEHOLDER LLC", // TODO: razón social real antes de producción
  // Sustituir solo por hashes SHA-256 publicados para los artefactos reales.
  SHA256_ARM64:
    "0e90fc3e8317fba8c05af9bd13dd5dab53f941000667f1d63d4cf80fcf27c793" as
      string | null,
  SHA256_INTEL: null as string | null,
  SHA256_WINDOWS: null as string | null,
  // CloakView Pro: URL del checkout de Polar (el link del producto, "Buy").
  // Mientras sea null, la tarjeta de precio muestra "Coming soon". Las keys las
  // emite Polar por el beneficio License keys del producto; la app las valida
  // contra api.polar.sh (ver src-tauri/src/license/polar.rs).
  PRO_CHECKOUT_URL:
    "https://buy.polar.sh/polar_cl_ogcfheRmLpOwHZJtGYQlcNTJcoaNp58vHcpiY30iI2g" as
      string | null,

  // --- Términos legales ---
  LEGAL_UPDATED: "2026-11-13", // ISO date: Fase 7
  LEGAL_JURISDICTION: "Florida, USA",

  locales: ["en", "es"] as const,
  defaultLocale: "en" as const,
} as const;

export type Locale = (typeof siteConfig.locales)[number];

/** Redes con perfil público. Para sumar una: agregá acá su URL y su glifo en social-links.tsx. */
export type SocialName =
  | "github"
  | "instagram"
  | "x"
  | "youtube"
  | "linkedin"
  | "discord"
  | "telegram"
  | "tiktok"
  | "mastodon"
  | "bluesky";

export const SOCIAL_LINKS: readonly {
  name: SocialName;
  url: string;
  label: string;
}[] = [
  { name: "github", url: siteConfig.GITHUB_REPO, label: "GitHub" },
  { name: "instagram", url: siteConfig.INSTAGRAM_URL, label: "Instagram" },
];

/** True mientras el repo de GitHub no se haya creado todavía. */
export function isGithubPending(): boolean {
  return (
    siteConfig.GITHUB_RELEASES.includes("OWNER") ||
    siteConfig.GITHUB_REPO.includes("OWNER")
  );
}

/** True mientras la razón social legal sea un placeholder. */
export function isLegalEntityPending(): boolean {
  return siteConfig.LEGAL_ENTITY.startsWith("PLACEHOLDER");
}
