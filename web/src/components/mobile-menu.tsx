"use client";

import { usePathname } from "next/navigation";
import { useEffect, useRef, type ReactNode } from "react";

/**
 * The phone menu: a native <details>, closed whenever the visitor picks something.
 *
 * The nav bar lives in the layout, which Next.js keeps mounted across client-side
 * navigations, so a plain <details> stayed open after tapping a link. It now closes
 * on a tap on any link inside it, on a route change, on a tap outside, and on Escape.
 */
export function MobileMenu({ label, children }: { label: string; children: ReactNode }) {
  const ref = useRef<HTMLDetailsElement>(null);
  const pathname = usePathname();

  const close = () => {
    if (ref.current) ref.current.open = false;
  };

  // A navigation that did not come from a tap in the menu (back button, a link elsewhere).
  useEffect(close, [pathname]);

  useEffect(() => {
    const onPointer = (event: PointerEvent) => {
      if (ref.current?.open && !ref.current.contains(event.target as Node)) close();
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape" && ref.current?.open) {
        close();
        ref.current?.querySelector("summary")?.focus();
      }
    };
    document.addEventListener("pointerdown", onPointer);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("pointerdown", onPointer);
      document.removeEventListener("keydown", onKey);
    };
  }, []);

  return (
    <details ref={ref} className="relative sm:hidden">
      <summary
        aria-label={label}
        className="tap-target inline-flex cursor-pointer list-none items-center justify-center rounded-[3px] border border-border px-2 text-foreground [&::-webkit-details-marker]:hidden"
      >
        <svg width="18" height="18" viewBox="0 0 18 18" aria-hidden="true" className="fill-none stroke-current">
          <path d="M2 5h14M2 9h14M2 13h14" strokeWidth="1.6" strokeLinecap="round" />
        </svg>
      </summary>
      {/* Any tap on a link inside closes the menu: home, pricing, docs, language, download. */}
      <div
        onClick={(event) => {
          if ((event.target as HTMLElement).closest("a")) close();
        }}
        className="absolute right-0 top-full z-20 mt-2 flex w-48 flex-col gap-1 rounded-[3px] border border-border bg-surface p-2 text-sm shadow-lg"
      >
        {children}
      </div>
    </details>
  );
}
