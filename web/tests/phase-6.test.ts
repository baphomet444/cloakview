import { test } from "node:test";
import assert from "node:assert/strict";
import en from "../src/dictionaries/en.json" with { type: "json" };
import es from "../src/dictionaries/es.json" with { type: "json" };
import changelog from "../src/content/changelog.json" with { type: "json" };

test("Phase 6 FAQ has at least 15 bilingual questions in five topics", () => {
  for (const dict of [en, es]) {
    assert.equal(dict.faq.topics.length, 5);
    assert.ok(dict.faq.topics.flatMap((topic) => topic.items).length >= 15);
  }
});

test("Local changelog has matching, nonempty translations", () => {
  assert.ok(changelog.length > 0);
  for (const entry of changelog) {
    assert.ok(entry.version.trim());
    for (const locale of ["en", "es"] as const) {
      assert.ok(entry.title[locale].trim());
      assert.ok(entry.body[locale].trim());
    }
    assert.deepEqual(Object.keys(entry.title).sort(), Object.keys(entry.body).sort());
  }
});
