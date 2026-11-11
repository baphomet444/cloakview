/**
 * A small Markdown reader for the copilot's answers, free of React so it can be
 * unit-tested. It covers what the answers use (paragraphs, headings, lists, bold,
 * italics, inline code, fenced code) and nothing else. The output is plain data that
 * `RichText` turns into elements, so no HTML from the answer is ever inserted.
 */

export type Inline = { kind: "text" | "bold" | "em" | "code"; text: string };

export type Block =
  | { kind: "p"; inlines: Inline[] }
  | { kind: "h"; level: 1 | 2 | 3; inlines: Inline[] }
  | { kind: "ul" | "ol"; items: Inline[][] }
  /** `closed` is false while the answer is still streaming the block. */
  | { kind: "code"; lang: string; text: string; closed: boolean };

const INLINE = /(`[^`\n]+`)|(\*\*[^*\n]+?\*\*)|(__[^_\n]+?__)|(\*[^*\s][^*\n]*?\*)|(\b_[^_\s][^_\n]*?_\b)/g;

export function parseInline(text: string): Inline[] {
  const out: Inline[] = [];
  let last = 0;
  for (const match of text.matchAll(INLINE)) {
    const at = match.index ?? 0;
    if (at > last) out.push({ kind: "text", text: text.slice(last, at) });
    const token = match[0];
    if (match[1]) out.push({ kind: "code", text: token.slice(1, -1) });
    else if (match[2] || match[3]) out.push({ kind: "bold", text: token.slice(2, -2) });
    else out.push({ kind: "em", text: token.slice(1, -1) });
    last = at + token.length;
  }
  if (last < text.length) out.push({ kind: "text", text: text.slice(last) });
  return out;
}

const FENCE = /^\s*```\s*([\w+-]*)\s*$/;
const HEADING = /^(#{1,6})\s+(.*)$/;
const BULLET = /^\s*[-*•]\s+(.*)$/;
const NUMBERED = /^\s*\d+[.)]\s+(.*)$/;

export function parseBlocks(source: string): Block[] {
  const blocks: Block[] = [];
  const lines = source.replace(/\r\n?/g, "\n").split("\n");
  let paragraph: string[] = [];
  const flush = () => {
    if (paragraph.length > 0) blocks.push({ kind: "p", inlines: parseInline(paragraph.join("\n")) });
    paragraph = [];
  };

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const fence = FENCE.exec(line);
    if (fence) {
      flush();
      const body: string[] = [];
      let closed = false;
      for (i++; i < lines.length; i++) {
        if (FENCE.test(lines[i])) {
          closed = true;
          break;
        }
        body.push(lines[i]);
      }
      blocks.push({ kind: "code", lang: fence[1].toLowerCase(), text: body.join("\n"), closed });
      continue;
    }
    if (line.trim() === "") {
      flush();
      continue;
    }
    const heading = HEADING.exec(line);
    if (heading) {
      flush();
      const level = Math.min(heading[1].length, 3) as 1 | 2 | 3;
      blocks.push({ kind: "h", level, inlines: parseInline(heading[2].trim()) });
      continue;
    }
    const bullet = BULLET.exec(line);
    const numbered = bullet ? null : NUMBERED.exec(line);
    if (bullet || numbered) {
      flush();
      const kind = bullet ? "ul" : "ol";
      const item = parseInline((bullet ?? numbered)![1]);
      const previous = blocks[blocks.length - 1];
      if (previous && previous.kind === kind) previous.items.push(item);
      else blocks.push({ kind, items: [item] });
      continue;
    }
    paragraph.push(line);
  }
  flush();
  return blocks;
}

/** A fenced block the diagram renderer understands. */
export function isDiagram(block: Block): block is Extract<Block, { kind: "code" }> {
  return block.kind === "code" && block.lang === "mermaid";
}
