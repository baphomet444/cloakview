import { ImageResponse } from "next/og";
import { siteConfig, type Locale } from "@/site.config";
import { getDictionary } from "@/lib/get-dictionary";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string }>;
}) {
  return {
    openGraph: {
      type: "image",
    },
  };
}

export default async function Image({
  params,
}: {
  params: Promise<{ locale: Locale }>;
}) {
  const { locale } = await params;
  const dict = await getDictionary(locale);
  const isSpanish = locale === "es";

  return new ImageResponse(
    (
      <div
        style={{
          display: "flex",
          width: "1200px",
          height: "630px",
          backgroundColor: "#0a0a0a",
          padding: "60px",
          flexDirection: "column",
          justifyContent: "space-between",
          alignItems: "flex-start",
          fontFamily: "system-ui, -apple-system, sans-serif",
        }}
      >
        {/* Logo + Título */}
        <div
          style={{
            display: "flex",
            alignItems: "flex-start",
            gap: "32px",
            marginBottom: "40px",
          }}
        >
          {/* Logo circular 128px */}
          <img
            src={`${siteConfig.url}/brand/logo.png`}
            alt="CloakView Logo"
            style={{
              width: "128px",
              height: "128px",
              borderRadius: "16px",
              flexShrink: 0,
            }}
          />

          {/* Título */}
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              justifyContent: "center",
            }}
          >
            <h1
              style={{
                fontSize: "72px",
                fontWeight: 800,
                margin: "0",
                color: "#ffffff",
                lineHeight: "1.1",
                letterSpacing: "-0.03em",
              }}
            >
              CloakView
            </h1>
            <p
              style={{
                fontSize: "32px",
                fontWeight: 400,
                margin: "8px 0 0 0",
                color: "#a0a0a0",
                lineHeight: "1.4",
              }}
            >
              {isSpanish
                ? "Tu copiloto privado de reuniones"
                : "Your private meeting copilot"}
            </p>
          </div>
        </div>

        {/* Descripción detallada */}
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: "16px",
          }}
        >
          <p
            style={{
              fontSize: "28px",
              fontWeight: 600,
              margin: "0",
              color: "#ffffff",
              lineHeight: "1.4",
            }}
          >
            {isSpanish
              ? "Transcripciones en directo • Sugerencias privadas • Tu clave de API"
              : "Live transcription • Private suggestions • Your API key"}
          </p>
          <p
            style={{
              fontSize: "20px",
              fontWeight: 400,
              margin: "0",
              color: "#808080",
              lineHeight: "1.5",
            }}
          >
            {isSpanish
              ? "Funciona en Zoom, Teams y Google Meet. Sin cuentas, sin telemetría."
              : "Works with Zoom, Teams, and Google Meet. No accounts, no telemetry."}
          </p>
        </div>

        {/* Pie con dominio */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "12px",
            fontSize: "18px",
            color: "#606060",
            fontWeight: 500,
          }}
        >
          <span>→</span>
          <span>{siteConfig.url}</span>
        </div>
      </div>
    ),
    {
      width: 1200,
      height: 630,
    }
  );
}
