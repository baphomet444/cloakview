import { isGithubPending, siteConfig } from "@/site.config";

type DownloadButtonProps = {
  label: string;
  soonLabel: string;
  className?: string;
};

/**
 * CTA de descarga que respeta isGithubPending():
 * mientras el repo no exista, se renderiza como botón deshabilitado con el
 * texto "próximamente" — nunca como un enlace roto.
 */
export function DownloadButton({
  label,
  soonLabel,
  className = "",
}: DownloadButtonProps) {
  const base = "btn btn-primary";

  const ready = !isGithubPending() && Boolean(siteConfig.SHA256_ARM64);

  if (!ready) {
    return (
      <span
        aria-disabled="true"
        title={soonLabel}
        className={`${base} cursor-not-allowed border border-border !bg-transparent !text-muted ${className}`}
      >
        {soonLabel}
      </span>
    );
  }

  return (
    <a
      href={siteConfig.GITHUB_RELEASES}
      rel="noopener noreferrer"
      className={`${base} ${className}`}
    >
      {label}
    </a>
  );
}
