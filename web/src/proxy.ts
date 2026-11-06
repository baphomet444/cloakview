import { NextRequest, NextResponse } from "next/server";
import { siteConfig } from "@/site.config";

// Localización "as-needed": inglés (default) vive en la raíz sin prefijo,
// español vive bajo /es. Reescribimos internamente a /en o /es dentro de
// src/app/[locale] para que ambos compartan el mismo árbol de rutas.

const PUBLIC_FILE = /\.(.*)$/;

export default function proxy(request: NextRequest) {
  const { pathname } = request.nextUrl;

  if (
    pathname.startsWith("/_next") ||
    pathname.startsWith("/api") ||
    PUBLIC_FILE.test(pathname)
  ) {
    return NextResponse.next();
  }

  const isSpanish = pathname === "/es" || pathname.startsWith("/es/");

  const locale = isSpanish ? "es" : siteConfig.defaultLocale;
  const rest = isSpanish ? pathname.replace(/^\/es/, "") || "/" : pathname;

  const url = request.nextUrl.clone();
  url.pathname = `/${locale}${rest}`;
  return NextResponse.rewrite(url);
}

export const config = {
  matcher: ["/((?!_next|api|.*\\..*).*)"],
};
