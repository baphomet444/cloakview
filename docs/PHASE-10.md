### Código exacto

En `globals.css`:

```css
@keyframes cloak {
  0%, 100% { opacity: 1; }
  45% { opacity: 0.12; }
}

.cloak-letter {
  animation: cloak 6s ease-in-out infinite;
}

.cloak-wordmark:hover .cloak-letter {
  animation-duration: 1.6s;
}

@media (prefers-reduced-motion: reduce) {
  .cloak-letter { animation: none; opacity: 1; }
}
```

En `nav-bar.tsx`, el logo:

```tsx
<Link
  href={withLocale("/", locale)}
  className="cloak-wordmark font-display text-xl font-semibold tracking-tight text-foreground"
  aria-label="CloakView"
>
  {"CloakView".split("").map((letter, i) => (
    <span
      key={i}
      aria-hidden="true"
      className="cloak-letter inline-block"
      style={{ animationDelay: `${i * 90}ms` }}
    >
      {letter}
    </span>
  ))}
</Link>
```

`inline-block` es obligatorio: sobre un inline puro la opacidad no se aplica por letra. El `aria-label` evita que el lector deletree.