import { test } from "node:test";
import assert from "node:assert/strict";
import { mint, verify } from "../src/lib/dl-token.ts";

const SECRET = "x".repeat(32);

function withSecret<T>(value: string | undefined, run: () => T): T {
  const before = process.env.DL_SECRET;
  if (value === undefined) delete process.env.DL_SECRET;
  else process.env.DL_SECRET = value;
  try {
    return run();
  } finally {
    if (before === undefined) delete process.env.DL_SECRET;
    else process.env.DL_SECRET = before;
  }
}

test("un token recién emitido vale 15 minutos y luego caduca", () => {
  withSecret(SECRET, () => {
    const now = 1_000_000;
    const token = mint(now);
    assert.equal(verify(token, now), true);
    assert.equal(verify(token, now + 15 * 60 * 1000 - 1), true);
    assert.equal(verify(token, now + 15 * 60 * 1000), false);
  });
});

test("un token alterado o de otro secreto no vale", () => {
  withSecret(SECRET, () => {
    const now = 1_000_000;
    const token = mint(now);
    const [payload, sig] = token.split(".");
    assert.equal(verify(`${Number(payload) + 60_000}.${sig}`, now), false, "caducidad alargada");
    assert.equal(verify(`${payload}.${sig.slice(0, -1)}A`, now), false, "firma cambiada");
    for (const bad of ["", ".", "abc", `${payload}.`, null]) assert.equal(verify(bad, now), false);
    const other = withSecret("y".repeat(32), () => mint(now));
    assert.equal(verify(other, now), false, "firmado con otro secreto");
  });
});

test("sin DL_SECRET, o con uno corto, no se emite nada", () => {
  withSecret(undefined, () => assert.throws(() => mint(), /DL_SECRET/));
  withSecret("short", () => assert.throws(() => mint(), /DL_SECRET/));
});
