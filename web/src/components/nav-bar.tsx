import Link from "next/link";
import type { Dictionary } from "@/lib/get-dictionary";
import type { Locale } from "@/site.config";
import { MobileMenu } from "@/components/mobile-menu";

function withLocale(path: string, locale: Locale) {
  if (locale === "en") return path;
  return path === "/" ? "/es" : `/es${path}`;
}

export function NavBar({ dict, locale }: { dict: Dictionary; locale: Locale }) {
  const otherLocale: Locale = locale === "en" ? "es" : "en";
  const links = [
    { href: "/", label: dict.nav.home },
    { href: "/pricing", label: dict.nav.pricing },
    { href: "/guide", label: dict.nav.docs },
  ];

  return (
    <header className="border-b border-border">
      <nav className="mx-auto flex max-w-7xl items-center justify-between gap-4 px-6 py-4">
        <Link
          href={withLocale("/", locale)}
          className="cloak-wordmark font-display text-xl font-extrabold tracking-[-0.03em] text-foreground"
          aria-label="CloakView"
        >
          {"CloakView".split("").map((letter, i) => (
            <span
              key={i}
              aria-hidden="true"
              className="cloak-letter inline-block"
              style={{ animationDelay: `${i * 90}ms` }}
            >
              {letter}
            </span>
          ))}
        </Link>

        <div className="hidden items-center gap-6 text-sm text-muted sm:flex">
          {links.map((link) => (
            <Link
              key={link.href}
              href={withLocale(link.href, locale)}
              className="inline-flex items-center tap-target hover:text-foreground transition-colors"
            >
              {link.label}
            </Link>
          ))}
          <Link
            href={withLocale("/", otherLocale)}
            className="inline-flex items-center tap-target rounded-[3px] border border-border px-2 py-1 text-xs uppercase hover:text-foreground transition-colors"
          >
            {otherLocale}
          </Link>
          <Link
            href={withLocale("/download", locale)}
            className="btn btn-primary btn-sm"
          >
            {dict.nav.download}
          </Link>
        </div>

        <MobileMenu label={dict.nav.menu}>
            {links.map((link) => (
              <Link
                key={link.href}
                href={withLocale(link.href, locale)}
                className="tap-target inline-flex items-center rounded-[2px] px-2 hover:text-foreground"
              >
                {link.label}
              </Link>
            ))}
            <Link
              href={withLocale("/", otherLocale)}
              className="tap-target inline-flex items-center rounded-[2px] px-2 text-xs uppercase hover:text-foreground"
            >
              {otherLocale}
            </Link>
            <Link
              href={withLocale("/download", locale)}
              className="btn btn-primary btn-sm mt-1 justify-center"
            >
              {dict.nav.download}
            </Link>
        </MobileMenu>
      </nav>
    </header>
  );
}