import Image from "next/image";

/**
 * The one way this site shows a picture inside an article: a bordered frame, a
 * descriptive alt and an optional caption. `width`/`height` are required so the
 * layout never jumps while the image loads (CLS stays at zero).
 */
export function MediaFigure({
  src,
  width,
  height,
  alt,
  caption,
  priority = false,
  className = "",
  maxWidthClass = "max-w-4xl",
}: {
  src: string;
  width: number;
  height: number;
  alt: string;
  caption?: string;
  priority?: boolean;
  className?: string;
  maxWidthClass?: string;
}) {
  return (
    <figure className={`mx-auto ${maxWidthClass} ${className}`}>
      <div className="overflow-hidden rounded-2xl border border-border bg-surface">
        <Image
          src={src}
          width={width}
          height={height}
          alt={alt}
          sizes="(min-width: 1024px) 896px, 100vw"
          priority={priority}
          className="h-auto w-full"
        />
      </div>
      {caption ? (
        <figcaption className="mt-3 text-sm leading-relaxed text-muted">{caption}</figcaption>
      ) : null}
    </figure>
  );
}
