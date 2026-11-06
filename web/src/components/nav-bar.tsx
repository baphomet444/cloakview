import Link from "next/link";
import type { Dictionary } from "@/lib/get-dictionary";
import type { Locale } from "@/site.config";

function withLocale(path: string, locale: Locale) {
  if (locale === "en") return path;
  return path === "/" ? "/es" : `/es${path}`;
}

export function NavBar({ dict, locale }: { dict: Dictionary; locale: Locale }) {
  const otherLocale: Locale = locale === "en" ? "es" : "en";
  const links = [
    { href: "/", label: dict.nav.home },
    { href: "/pricing", label: dict.nav.pricing },
    { href: "/docs", label: dict.nav.docs },
    { href: "/faq", label: dict.nav.faq },
  ];

  return (
    <header className="border-b border-border">
      <nav className="mx-auto flex max-w-5xl items-center justify-between px-6 py-4">
        <Link
          href={withLocale("/", locale)}
          className="font-semibold tracking-tight text-foreground"
        >
          CloakView
        </Link>
        <div className="flex items-center gap-6 text-sm text-muted">
          {links.map((link) => (
            <Link
              key={link.href}
              href={withLocale(link.href, locale)}
              className="hover:text-foreground transition-colors"
            >
              {link.label}
            </Link>
          ))}
          <Link
            href={withLocale("/", otherLocale)}
            className="rounded border border-border px-2 py-1 text-xs uppercase hover:text-foreground transition-colors"
          >
            {otherLocale}
          </Link>
          <Link
            href={withLocale("/pricing", locale)}
            className="rounded bg-accent px-3 py-1.5 font-medium text-accent-foreground hover:opacity-90 transition-opacity"
          >
            {dict.nav.download}
          </Link>
        </div>
      </nav>
    </header>
  );
}
