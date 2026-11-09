import { Fragment } from "react";
import { siteConfig } from "@/site.config";

/** Texto legal que sustituye el token {{email}} por un enlace mailto con el valor de site.config. */
export function LegalText({ text, className = "" }: { text: string; className?: string }) {
  const parts = text.split("{{email}}");
  return (
    <p className={`whitespace-pre-wrap leading-relaxed ${className}`}>
      {parts.map((part, i) => (
        <Fragment key={i}>
          {part}
          {i < parts.length - 1 && (
            <a
              className="text-accent underline focus-visible:outline-2 focus-visible:outline-accent"
              href={`mailto:${siteConfig.SUPPORT_EMAIL}`}
            >
              {siteConfig.SUPPORT_EMAIL}
            </a>
          )}
        </Fragment>
      ))}
    </p>
  );
}
