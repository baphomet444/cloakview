import { ReactNode } from 'react';
import { siteConfig } from '@/site.config';
import Link from 'next/link';

interface LegalLayoutProps {
  children: ReactNode;
  title: string;
  updated: string;
  jurisdiction: string;
  locale: string;
}

export function LegalLayout({
  children,
  title,
  updated,
  jurisdiction,
  locale,
}: LegalLayoutProps) {
  const isPlaceholder = siteConfig.LEGAL_ENTITY.startsWith('PLACEHOLDER');

  return (
    <main className="min-h-screen bg-background text-foreground">
      <article className="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-12 sm:py-16 lg:py-20">
        {/* Header */}
        <header className="mb-12">
          <h1 className="text-4xl sm:text-5xl font-bold tracking-tight mb-4">
            {title}
          </h1>

          {/* Legal Entity Placeholder Warning */}
          {isPlaceholder && (
            <div className="bg-accent/10 border border-accent/40 rounded-lg p-4 mb-6">
              <p className="text-foreground text-sm">
                ⚠️ <strong>This policy uses a placeholder legal entity:</strong> {siteConfig.LEGAL_ENTITY}
                <br />
                <em>The official legal entity and contact details will be updated before v1 release.</em>
              </p>
            </div>
          )}

          {/* Metadata */}
          <div className="text-sm text-muted space-y-2">
            <p>
              <strong>{updated}</strong>: {siteConfig.LEGAL_UPDATED}
            </p>
            <p>{jurisdiction}</p>
          </div>
        </header>

        {/* Content */}
        <div>
          {children}
        </div>

        {/* Cross-links */}
        <nav className="mt-12 pt-8 border-t border-border">
          <div className="text-sm text-muted space-y-3">
            <p className="font-semibold text-muted">Related policies:</p>
            <div className="flex flex-wrap gap-6">
              {locale === 'en' ? (
                <>
                  <Link
                    href="/en/privacy"
                    className="text-blue-400 hover:text-blue-300 transition"
                  >
                    Privacy Policy
                  </Link>
                  <Link
                    href="/en/terms"
                    className="text-blue-400 hover:text-blue-300 transition"
                  >
                    Terms of Service
                  </Link>
                  <Link
                    href="/en/cookies"
                    className="text-blue-400 hover:text-blue-300 transition"
                  >
                    Cookies Policy
                  </Link>
                </>
              ) : (
                <>
                  <Link
                    href="/es/privacy"
                    className="text-blue-400 hover:text-blue-300 transition"
                  >
                    Política de Privacidad
                  </Link>
                  <Link
                    href="/es/terms"
                    className="text-blue-400 hover:text-blue-300 transition"
                  >
                    Términos de Servicio
                  </Link>
                  <Link
                    href="/es/cookies"
                    className="text-blue-400 hover:text-blue-300 transition"
                  >
                    Política de Cookies
                  </Link>
                </>
              )}
            </div>
          </div>
        </nav>
      </article>
    </main>
  );
}
