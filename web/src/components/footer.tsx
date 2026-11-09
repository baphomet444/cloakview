import Link from "next/link";
import type { Dictionary } from "@/lib/get-dictionary";
import type { Locale } from "@/site.config";
import { siteConfig } from "@/site.config";

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
    { href: "/privacy", label: dict.footer.privacy },
    { href: "/terms", label: dict.footer.terms },
    { href: "/cookies", label: dict.footer.cookies },
    { href: "/changelog", label: dict.footer.changelog },
  ];

  return (
    <footer className="mt-auto border-t border-border">
      <div className="mx-auto flex max-w-7xl flex-col gap-4 px-6 py-8 text-sm text-muted sm:flex-row sm:items-center sm:justify-between">
        <p>{dict.footer.tagline}</p>
        <div className="flex flex-wrap gap-4">
          {links.map((link) => (
            <Link
              key={link.href}
              href={withLocale(link.href, locale)}
              className="inline-flex items-center tap-target hover:text-foreground transition-colors"
            >
              {link.label}
            </Link>
          ))}
        </div>
        <p>
          © {BUILD_YEAR} {siteConfig.name}. {dict.footer.rights}
        </p>
      </div>
    </footer>
  );
}
