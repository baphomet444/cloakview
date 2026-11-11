import { describe, expect, it } from "vitest";
import { isDiagram, parseBlocks, parseInline } from "./markdown";

describe("parseInline", () => {
  it("splits bold, italics and code out of plain text", () => {
    expect(parseInline("The **answer** is `80 km/h`, *roughly*.")).toEqual([
      { kind: "text", text: "The " },
      { kind: "bold", text: "answer" },
      { kind: "text", text: " is " },
      { kind: "code", text: "80 km/h" },
      { kind: "text", text: ", " },
      { kind: "em", text: "roughly" },
      { kind: "text", text: "." },
    ]);
  });

  it("leaves arithmetic and snake_case alone", () => {
    expect(parseInline("2 * 3 * 4 and my_var_name")).toEqual([{ kind: "text", text: "2 * 3 * 4 and my_var_name" }]);
  });

  it("never interprets HTML: it stays text", () => {
    expect(parseInline("<img src=x onerror=alert(1)>")).toEqual([{ kind: "text", text: "<img src=x onerror=alert(1)>" }]);
  });
});

describe("parseBlocks", () => {
  it("reads headings, paragraphs and both kinds of list", () => {
    const blocks = parseBlocks("## Answer\n\n**80 km/h**\n\n1. Convert the time\n2. Divide\n\n- one\n- two");
    expect(blocks.map((b) => b.kind)).toEqual(["h", "p", "ol", "ul"]);
    expect(blocks[2]).toMatchObject({ kind: "ol", items: [[{ text: "Convert the time" }], [{ text: "Divide" }]] });
  });

  it("keeps a code block verbatim, with its language", () => {
    const [block] = parseBlocks("```python\ndef f(x):\n    return x * 2  # **not bold**\n```");
    expect(block).toEqual({ kind: "code", lang: "python", text: "def f(x):\n    return x * 2  # **not bold**", closed: true });
  });

  it("marks a code block that is still streaming as not closed", () => {
    const [block] = parseBlocks("```mermaid\nflowchart TD\n  A --> B");
    expect(block).toMatchObject({ kind: "code", lang: "mermaid", closed: false });
    expect(isDiagram(block)).toBe(true);
  });

  it("a paragraph keeps its line breaks", () => {
    expect(parseBlocks("line one\nline two")).toEqual([{ kind: "p", inlines: [{ kind: "text", text: "line one\nline two" }] }]);
  });
});
