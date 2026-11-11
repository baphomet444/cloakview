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

  // --- Placeholders pendientes de valores reales ---
  GITHUB_RELEASES: "https://github.com/baphomet444/cloakview/releases/latest",
  GITHUB_REPO: "https://github.com/baphomet444/cloakview",
  SUPPORT_EMAIL: "support@getcloakview.com",
  LEGAL_ENTITY: "PLACEHOLDER LLC", // TODO: razón social real antes de producción
  // Sustituir solo por hashes SHA-256 publicados para los artefactos reales.
  SHA256_ARM64: "7cc569777e13e8fc87c81b53c1366048ab51fa374c7cdc929e36326982a8d2a2" as string | null,
  SHA256_INTEL: null as string | null,
  SHA256_WINDOWS: null as string | null,

  // --- Términos legales ---
  LEGAL_UPDATED: "2026-11-11", // ISO date: Fase 7
  LEGAL_JURISDICTION: "Florida, USA",

  locales: ["en", "es"] as const,
  defaultLocale: "en" as const,
} as const;

export type Locale = (typeof siteConfig.locales)[number];

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
