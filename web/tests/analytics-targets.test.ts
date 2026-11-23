import { test } from "node:test";
import assert from "node:assert/strict";
import { eventForHref } from "../src/lib/analytics-targets.ts";

test("los enlaces de descarga emiten download_dmg", () => {
  assert.equal(eventForHref("/api/dl"), "download_dmg");
  assert.equal(eventForHref("https://getcloakview.com/api/dl"), "download_dmg");
  // Enlaces viejos ya compartidos: /dl/mac redirige a /api/dl y sigue contando.
  assert.equal(eventForHref("/dl/mac"), "download_dmg");
  assert.equal(
    eventForHref("https://github.com/baphomet444/cloakview/releases/latest"),
    "download_dmg",
  );
  assert.equal(
    eventForHref("https://github.com/baphomet444/cloakview/releases/download/v0.3.2/CloakView.dmg"),
    "download_dmg",
  );
});

test("el checkout de Polar emite checkout_click", () => {
  assert.equal(
    eventForHref("https://buy.polar.sh/polar_cl_ogcfheRmLpOwHZJtGYQlcNTJcoaNp58vHcpiY30iI2g"),
    "checkout_click",
  );
});

test("el resto de los enlaces no emite eventos", () => {
  for (const href of ["/pricing", "/es/download", "/api/assistant", "#how-it-works", "https://github.com/baphomet444/cloakview", "", null, undefined]) {
    assert.equal(eventForHref(href), null, `no debería emitir para ${String(href)}`);
  }
});
