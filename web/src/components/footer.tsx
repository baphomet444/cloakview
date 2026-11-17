import Link from "next/link";
import type { Dictionary } from "@/lib/get-dictionary";
import type { Locale } from "@/site.config";
import { siteConfig } from "@/site.config";
import { SocialLinks } from "@/components/social-links";

function withLocale(path: string, locale: Locale) {
  if (locale === "en") return path;
  return path === "/" ? "/es" : `/es${path}`;
}

// Año fijo en build time: evita que Next.js trate new Date() como valor
// dinámico no cacheable durante el prerender estático (Cache Components).
const BUILD_YEAR = new Date().getFullYear();

export function Footer({ dict, locale }: { dict: Dictionary; locale: Locale }) {
  const links = [
    { href: "/support", label: dict.footer.support },
    { href: "/faq", label: dict.faq.title },
    { href: "/vs", label: dict.footer.compare },
    { href: "/privacy", label: dict.footer.privacy },
    { href: "/terms", label: dict.footer.terms },
    { href: "/cookies", label: dict.footer.cookies },
    { href: "/changelog", label: dict.footer.changelog },
  ];

  return (
    <footer className="mt-auto border-t border-border">
      {/* pb-24 en vez de pb-8: el botón flotante de CloakView vive en la esquina
          inferior derecha y sin ese aire tapaba los iconos de las redes. */}
      <div className="mx-auto max-w-7xl px-6 pt-8 pb-24">
        {/* Fila 1: la promesa y el mapa del sitio. */}
        <div className="flex flex-col gap-4 text-sm text-muted sm:flex-row sm:items-center sm:justify-between">
          <p>{dict.footer.tagline}</p>
          <nav aria-label={dict.footer.navLabel}>
            <ul className="flex flex-wrap gap-4">
              {links.map((link) => (
                <li key={link.href}>
                  <Link
                    href={withLocale(link.href, locale)}
                    className="inline-flex items-center tap-target hover:text-foreground transition-colors"
                  >
                    {link.label}
                  </Link>
                </li>
              ))}
            </ul>
          </nav>
        </div>

        {/* Fila 2: el copyright y los perfiles. Las redes van acá y no en la
            fila de arriba porque no son navegación del producto: quien las
            busca baja hasta el final. */}
        <div className="mt-6 flex flex-col gap-4 border-t border-border/60 pt-6 text-sm text-muted sm:flex-row-reverse sm:items-center sm:justify-between">
          <SocialLinks heading={dict.footer.follow} />
          <p>
            © {BUILD_YEAR} {siteConfig.name}. {dict.footer.rights}
          </p>
        </div>
      </div>
    </footer>
  );
}
