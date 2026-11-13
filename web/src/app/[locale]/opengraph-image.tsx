import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { ImageResponse } from "next/og";
import { siteConfig, type Locale } from "@/site.config";
import { getDictionary } from "@/lib/get-dictionary";
import { OG_IMAGE_SIZE } from "@/lib/seo";

// The social image shared by every page (each page links it through `ogImages`).
export const size = OG_IMAGE_SIZE;
export const contentType = "image/png";
// Static by convention, so English; the other pages pass the localized `meta.ogAlt`.
export const alt = "CloakView — A Meeting Copilot only you can see";

// Unbounded (SIL OFL, licence in assets/fonts), read from disk like the logo so a render
// never depends on a font CDN. Two weights: bold for the name and the feature line.
const FONT = "Unbounded";
const fonts = [
  { name: FONT, data: await readFile(join(process.cwd(), "assets/fonts/unbounded-latin-400-normal.woff")), weight: 400 as const, style: "normal" as const },
  { name: FONT, data: await readFile(join(process.cwd(), "assets/fonts/unbounded-latin-700-normal.woff")), weight: 700 as const, style: "normal" as const },
];

// Read from disk and inlined, never fetched from the live site: the deployed site may
// not have this file yet (a preview, or the first deploy that adds it), and the image
// would then fail to render.
const logo = `data:image/png;base64,${(await readFile(join(process.cwd(), "public/brand/logo.png"))).toString("base64")}`;
const LOGO_PX = 128;

export default async function Image({ params }: { params: Promise<{ locale: string }> }) {
  const { locale: raw } = await params;
  const locale: Locale = raw === "es" ? "es" : "en";
  const { meta } = await getDictionary(locale);
  const domain = siteConfig.url.replace(/^https?:\/\//, "");

  return new ImageResponse(
    (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          justifyContent: "space-between",
          width: "100%",
          height: "100%",
          padding: 64,
          backgroundColor: "#0a0a0a",
          backgroundImage: "radial-gradient(circle at 85% 15%, rgba(255,107,26,0.18), transparent 55%)",
          color: "#ffffff",
          fontFamily: FONT,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 36 }}>
          {/* width/height as attributes: the image renderer needs them to size an <img>. */}
          <img src={logo} width={LOGO_PX} height={LOGO_PX} alt="" />
          <div style={{ display: "flex", flexDirection: "column" }}>
            <div style={{ fontSize: 72, fontWeight: 700, letterSpacing: -1, lineHeight: 1.05 }}>{meta.siteName}</div>
            <div style={{ display: "flex", fontSize: 26, color: "#b4b4b4", marginTop: 14 }}>{meta.ogTagline}</div>
          </div>
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: 18 }}>
          <div style={{ fontSize: 24, fontWeight: 700, color: "#ff6b1a" }}>{meta.ogFeatures}</div>
          <div style={{ fontSize: 18, color: "#8a8a8a" }}>{meta.ogFootnote}</div>
        </div>

        <div style={{ display: "flex", fontSize: 18, color: "#6b6b6b" }}>{domain}</div>
      </div>
    ),
    { ...size, fonts },
  );
}
