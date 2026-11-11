import { useEffect, useId, useState } from "react";
import { isDiagram, parseBlocks, type Inline } from "./markdown";

/**
 * An answer, formatted: Markdown text plus diagrams. The text becomes React elements
 * (never HTML from the answer). A fenced `mermaid` block becomes a drawing once the
 * block is complete; while it streams, or if it cannot be drawn, its source is shown.
 */
export function RichText(props: { text: string }) {
  return (
    <div className="rich">
      {parseBlocks(props.text).map((block, index) => {
        switch (block.kind) {
          case "p":
            return <p key={index}>{inlines(block.inlines)}</p>;
          case "h":
            return (
              <p key={index} className={`rich-h rich-h${block.level}`}>
                {inlines(block.inlines)}
              </p>
            );
          case "ul":
            return (
              <ul key={index}>
                {block.items.map((item, i) => (
                  <li key={i}>{inlines(item)}</li>
                ))}
              </ul>
            );
          case "ol":
            return (
              <ol key={index}>
                {block.items.map((item, i) => (
                  <li key={i}>{inlines(item)}</li>
                ))}
              </ol>
            );
          case "code":
            return isDiagram(block) && block.closed ? (
              <Diagram key={index} source={block.text} />
            ) : (
              <pre key={index} className="rich-code">
                <code>{block.text}</code>
              </pre>
            );
        }
      })}
    </div>
  );
}

function inlines(parts: Inline[]) {
  return parts.map((part, index) => {
    if (part.kind === "bold") return <strong key={index}>{part.text}</strong>;
    if (part.kind === "em") return <em key={index}>{part.text}</em>;
    if (part.kind === "code") return <code key={index}>{part.text}</code>;
    return part.text;
  });
}

/**
 * Draws one diagram. The library is large, so it is loaded only when the first
 * diagram appears. `securityLevel: "strict"` makes it sanitize its own output and
 * ignore any HTML or links in the source.
 */
function Diagram(props: { source: string }) {
  const id = `diagram-${useId().replace(/[^a-zA-Z0-9]/g, "")}`;
  const [svg, setSvg] = useState<string | null>(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setSvg(null);
    setFailed(false);
    void (async () => {
      try {
        const { default: mermaid } = await import("mermaid");
        const dark = document.documentElement.dataset.theme !== "light";
        mermaid.initialize({ startOnLoad: false, securityLevel: "strict", theme: dark ? "dark" : "default" });
        const drawn = await mermaid.render(id, props.source);
        if (!cancelled) setSvg(drawn.svg);
      } catch {
        // Not valid diagram syntax: show what the copilot wrote instead of nothing.
        document.getElementById(`d${id}`)?.remove(); // the library's error placeholder
        if (!cancelled) setFailed(true);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [id, props.source]);

  if (failed || svg === null) {
    return (
      <pre className="rich-code">
        <code>{props.source}</code>
      </pre>
    );
  }
  // The library's own sanitized SVG; the answer's text never reaches this as HTML.
  return <div className="rich-diagram" role="img" dangerouslySetInnerHTML={{ __html: svg }} />;
}
