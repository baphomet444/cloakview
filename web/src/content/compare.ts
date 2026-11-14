import type { Locale } from "@/site.config";

export const compareSlugs = ["cluely", "smog-ai", "final-round-ai"] as const;
export type CompareSlug = (typeof compareSlugs)[number];

export function isCompareSlug(value: string): value is CompareSlug {
  return (compareSlugs as readonly string[]).includes(value);
}

export const comparePath = (locale: Locale, slug?: CompareSlug) =>
  `${locale === "en" ? "" : "/es"}/vs${slug ? `/${slug}` : ""}`;

export type CompareRow = { label: string; we: string; they: string };
export type ComparePoint = { title: string; body: string };

export type CompareDoc = {
  /** The other product's name, as it writes it. */
  name: string;
  /** Where the facts came from: shown on the page so the claims are traceable. */
  vendorUrl: string;
  /** Metadata */
  title: string;
  description: string;
  /** Visible copy */
  h1: string;
  standfirst: string;
  intro: string[];
  rows: CompareRow[];
  theyWin: ComparePoint[];
  weWin: ComparePoint[];
  verdict: string;
  faq: { q: string; a: string }[];
};

export type CompareIndex = {
  title: string;
  description: string;
  h1: string;
  intro: string;
  note: string;
  checkedLabel: string;
};

export const CHECKED_AT: Record<Locale, string> = {
  en: "November 2026",
  es: "noviembre de 2026",
};

export const compareIndex: Record<Locale, CompareIndex> = {
  en: {
    title: "Compare CloakView with other AI copilots",
    description:
      "How CloakView compares with Cluely, Smog AI and Final Round AI on price, screen-share stealth, whose models answer and where transcripts end up.",
    h1: "How CloakView compares",
    intro:
      "Three comparisons, same structure each time: what the tool costs, whether the overlay is actually hidden in the tier you would buy, whose models answer, and where the transcript ends up. Every claim about a rival comes from that vendor's own pricing or product pages.",
    note: "CloakView is a Mac-only app with no subscription, no account and no backend: you bring an OpenAI key and pay OpenAI for what you use. Where a rival does something better, the page says so.",
    checkedLabel: "Prices checked",
  },
  es: {
    title: "Compara CloakView con otros copilotos",
    description:
      "Cómo se compara CloakView con Cluely, Smog AI y Final Round AI en precio, invisibilidad, modelos y dónde acaban tus transcripciones.",
    h1: "Cómo se compara CloakView",
    intro:
      "Tres comparativas con la misma estructura: cuánto cuesta la herramienta, si la ventana está realmente oculta en el plan que comprarías, qué modelos responden y dónde acaba la transcripción. Todo lo que decimos de cada rival sale de sus propias páginas de precios o de producto.",
    note: "CloakView es una app solo para Mac, sin suscripción, sin cuenta y sin backend: traes tu clave de OpenAI y le pagas a OpenAI por lo que uses. Cuando un rival hace algo mejor, la página lo dice.",
    checkedLabel: "Precios revisados en",
  },
};

export type CompareLabels = {
  tableFeature: string;
  theyWinHeading: string;
  weWinHeading: string;
  verdictHeading: string;
  faqHeading: string;
  ctaTitle: string;
  ctaBody: string;
  ctaButton: string;
  ctaNote: string;
  sourceLabel: string;
  othersHeading: string;
  readLabel: string;
};

export const compareLabels: Record<Locale, CompareLabels> = {
  en: {
    tableFeature: "Feature",
    theyWinHeading: "Where {name} is the better choice",
    weWinHeading: "Where CloakView is the better choice",
    verdictHeading: "Which one to buy",
    faqHeading: "Common questions",
    ctaTitle: "Try it before you decide",
    ctaBody:
      "The app is free and needs no account. Download it, paste an OpenAI key, and run it on your next call — the overlay stays out of screen shares from the first launch.",
    ctaButton: "Download for macOS",
    ctaNote: "macOS 14.2 or later · Apple Silicon · signed and notarized",
    sourceLabel: "Figures from",
    othersHeading: "Other comparisons",
    readLabel: "Read the comparison",
  },
  es: {
    tableFeature: "Apartado",
    theyWinHeading: "Dónde {name} es mejor opción",
    weWinHeading: "Dónde CloakView es mejor opción",
    verdictHeading: "Cuál comprar",
    faqHeading: "Preguntas frecuentes",
    ctaTitle: "Pruébalo antes de decidir",
    ctaBody:
      "La app es gratis y no pide cuenta. Descárgala, pega una clave de OpenAI y úsala en tu próxima llamada: la ventana se queda fuera de la pantalla compartida desde el primer arranque.",
    ctaButton: "Descargar para macOS",
    ctaNote: "macOS 14.2 o posterior · Apple Silicon · firmada y notarizada",
    sourceLabel: "Datos de",
    othersHeading: "Otras comparativas",
    readLabel: "Leer la comparativa",
  },
};

export const compareDocs: Record<Locale, Record<CompareSlug, CompareDoc>> = {
  en: {
    cluely: {
      name: "Cluely",
      vendorUrl: "https://cluely.com/pricing",
      title: "CloakView vs Cluely: price, stealth and privacy",
      description:
        "Cluely sells screen-share undetectability as a separate top tier. CloakView hides the overlay in the free app, and you pay OpenAI for your own usage.",
      h1: "CloakView vs Cluely",
      standfirst:
        "Cluely sells undetectability as its most expensive tier. CloakView hides the overlay in the free app.",
      intro: [
        "Both products do the same thing in the same place: an AI that listens to a live call and puts an answer on your screen while the other side sees nothing. The difference is the business model, and it shows up in three places — what the stealth costs, whose models answer, and whether your transcripts leave your Mac.",
        "Everything below about Cluely comes from its public pricing page in November 2026: a free Starter tier, a Pro subscription billed monthly or annually (from $11.99/month on annual billing), and a separate Pro + Undetectability tier that adds hiding from screen sharing. CloakView's side describes what the shipped app does today.",
      ],
      rows: [
        {
          label: "Cost to start",
          we: "Free app. No account, no card, no trial clock.",
          they: "Free Starter tier, with a limited number of AI responses and limited meeting notes.",
        },
        {
          label: "Ongoing cost",
          we: "No subscription. You pay OpenAI for the usage you generate, at their public rates. CloakView Pro is a single $99 payment.",
          they: "A Pro subscription, monthly or annual. Their pricing page advertises Pro from $11.99/month billed annually.",
        },
        {
          label: "Hiding the overlay from a screen share",
          we: "Included in every tier, free one included. It is the reason the app exists.",
          they: "A separate top tier: Pro + Undetectability. On Starter and Pro, undetectability is not in the plan.",
        },
        {
          label: "Whose models answer",
          we: "Yours. You paste your own OpenAI key and the app calls OpenAI directly.",
          they: "Theirs. Cluely's models and infrastructure, bundled into the subscription.",
        },
        {
          label: "Account",
          we: "None. There is nothing to sign up for and no dashboard to log into.",
          they: "A Cluely account tied to the subscription.",
        },
        {
          label: "Where your data goes",
          we: "Audio and transcripts stay on your Mac. There is no CloakView backend and no telemetry.",
          they: "Processed on Cluely's service, under Cluely's privacy policy.",
        },
        {
          label: "Platforms",
          we: "macOS 14.2 or later, Apple Silicon.",
          they: "A macOS desktop app, plus a mobile app on their site.",
        },
        {
          label: "Transcription",
          we: "Live transcription through OpenAI's realtime speech models; live translation is part of Pro.",
          they: "Live transcription and unlimited AI responses on Pro.",
        },
        {
          label: "Reading your screen",
          we: "Pro reads a question or a task straight off your screen.",
          they: "Yes — vision features on the paid tiers.",
        },
        {
          label: "Notes and history",
          we: "Automatic and manual notes; searchable history on Pro.",
          they: "Meeting notetaking, unlimited on Pro.",
        },
        {
          label: "Best fit",
          we: "Mac users who want a private tool they pay for once and control themselves.",
          they: "People who want a hosted product across devices and prefer a subscription to managing an API key.",
        },
      ],
      theyWin: [
        {
          title: "Nothing to set up",
          body: "Subscribe, download, sign in. There is no OpenAI dashboard to visit, no key to paste, no spending limit to configure — which for many people is the whole point of paying for software.",
        },
        {
          title: "It runs on your phone too",
          body: "CloakView is a Mac app, full stop. If you want the same assistant on a phone, Cluely is the one that has it.",
        },
        {
          title: "Model upkeep is their job",
          body: "Cluely swaps models on their side as new ones ship. With CloakView you choose the model and you carry the cost of that choice.",
        },
      ],
      weWin: [
        {
          title: "Stealth is not an upsell",
          body: "Hiding the overlay from a screen share is what CloakView is built around, and it works on the free app. With Cluely that behaviour is the most expensive tier.",
        },
        {
          title: "No subscription to remember",
          body: "The app is free and Pro is one $99 payment. The only recurring cost is OpenAI usage, billed to you by OpenAI at the rates published on their pricing page.",
        },
        {
          title: "Your interviews stay on your disk",
          body: "There is no CloakView server holding recordings or transcripts. If you uninstall the app, what is on your Mac is everything that exists.",
        },
        {
          title: "You own the key and the model",
          body: "Revoke the key and the app simply stops. Change models when a better one ships. No cancellation flow, because there is nothing to cancel.",
        },
      ],
      verdict:
        "If you want a hosted product with a phone app and zero setup, Cluely is a reasonable buy — just check which tier actually hides the overlay before an interview. If what you want is an invisible copilot on your Mac that you pay for once and whose transcripts never leave the machine, CloakView is cheaper and more private, and the stealth is not an upgrade.",
      faq: [
        {
          q: "Is CloakView a Cluely clone?",
          a: "No. Both put an AI answer on screen during a live call, but CloakView is macOS-only, has no backend and no account, and runs on your own OpenAI key. Hiding the overlay is built in rather than sold separately.",
        },
        {
          q: "Is CloakView cheaper than Cluely?",
          a: "For most people, yes. The CloakView app is free and Pro is a one-time $99; the only recurring cost is the OpenAI usage you actually generate. Cluely charges a subscription, and undetectability sits in its highest tier.",
        },
        {
          q: "Can CloakView use Claude or Gemini instead of OpenAI?",
          a: "Not today. CloakView talks to OpenAI and you supply the key, so the model list is OpenAI's. Cluely chooses the models for you on its own infrastructure.",
        },
        {
          q: "Does CloakView work on Windows?",
          a: "No. CloakView requires macOS 14.2 or later on Apple Silicon. Cluely has a macOS app and a mobile app.",
        },
      ],
    },
    "smog-ai": {
      name: "Smog AI",
      vendorUrl: "https://smog-ai.com/pricing",
      title: "CloakView vs Smog AI: price, stealth and privacy",
      description:
        "Smog AI includes stealth in its free tier and sells a $199 lifetime licence. CloakView is free, with a $99 one-time Pro. Cost, quotas and data compared.",
      h1: "CloakView vs Smog AI",
      standfirst:
        "The closest pair on this site: both are native Mac overlays that stay invisible to a screen share.",
      intro: [
        "Smog and CloakView are the nearest neighbours here. Both are native Mac overlays that hide from screen sharing, both keep transcripts on your machine, and both can run on your own OpenAI key instead of a bundled subscription. The differences are in the price shape and the ceilings.",
        "Smog's plans, from its own pricing page in November 2026: a free tier with the undetectable overlay and a limited token quota, Pro at $24.99/month (less on annual billing) with their models included, and a $199 one-time Lifetime licence, listed at $299 and capped at 50 seats, which lets you bring your own OpenAI key. CloakView has no quota and no account at all.",
      ],
      rows: [
        {
          label: "Cost to start",
          we: "Free app. No account, no card.",
          they: "Free tier, with the undetectable overlay included and a limited token quota.",
        },
        {
          label: "Ongoing cost",
          we: "No subscription. You pay OpenAI for your own usage. CloakView Pro is a single $99 payment.",
          they: "Pro at $24.99/month, or a $199 one-time Lifetime licence (listed at $299, 50 seats).",
        },
        {
          label: "Hiding the overlay from a screen share",
          we: "Included in every tier, free app included.",
          they: "Included from the free tier up. Their strongest point, and it is a fair one.",
        },
        {
          label: "Your own API key",
          we: "Yes, for every feature. That is the only way it works.",
          they: "On the Lifetime plan. Pro bundles access to their models instead.",
        },
        {
          label: "Usage quota",
          we: "None. Your only ceiling is the spending limit you set in your own OpenAI account.",
          they: "The free tier is metered by a token quota; paid tiers are unlimited under a fair-use policy.",
        },
        {
          label: "Where your data goes",
          we: "Audio and transcripts on your Mac. No CloakView backend, no telemetry.",
          they: "Interface, transcripts and notes stored locally; AI requests send the minimum text needed, which they say is not stored.",
        },
        {
          label: "Platforms",
          we: "macOS 14.2 or later, Apple Silicon.",
          they: "macOS 14.2 or later (Apple Silicon; Intel if the Mac runs 14.2), plus a Windows 10/11 x64 beta.",
        },
        {
          label: "Languages",
          we: "OpenAI realtime transcription; live translation on Pro.",
          they: "Automatic language detection and real-time translation.",
        },
        {
          label: "Reading your screen",
          we: "Pro reads a question or task straight off your screen.",
          they: "Vision — screen analysis, capped by the token quota on the free tier.",
        },
        {
          label: "Notes and export",
          we: "Automatic and manual notes; searchable history on Pro.",
          they: "Automatic and manual notes, with PDF, Markdown and email export on paid tiers.",
        },
        {
          label: "Best fit",
          we: "Mac-only users who want one payment, no account and their own key from the first launch.",
          they: "People who want a Windows beta or a lifetime licence that includes the models.",
        },
      ],
      theyWin: [
        {
          title: "A lifetime plan with models included",
          body: "Smog's $199 Lifetime licence is one payment for the app and the AI behind it. If you would rather never open an OpenAI dashboard, that is a coherent offer — CloakView always needs a key.",
        },
        {
          title: "Windows, in beta",
          body: "Smog runs a Windows 10/11 x64 beta. CloakView is macOS on Apple Silicon only, so if your work laptop is a PC, this comparison ends there.",
        },
        {
          title: "More export formats",
          body: "PDF, Markdown and email export on the paid tiers are documented on their pricing page. CloakView's history is searchable but the export surface is narrower.",
        },
      ],
      weWin: [
        {
          title: "Half the price of the lifetime licence",
          body: "Pro is $99 once, against $199 for Smog's lifetime plan — and you keep the key, so the model and the per-token cost stay yours.",
        },
        {
          title: "No token quota anywhere",
          body: "The free CloakView app has no metered tier and no fair-use throttle. The only limit is your own OpenAI spending limit, which you set and can change.",
        },
        {
          title: "No account, no sign-in",
          body: "There is no CloakView account to create, verify or lose. Paste the key and the app works; revoke the key and it stops.",
        },
        {
          title: "Nothing to unlearn for macOS users",
          body: "One DMG, Apple Silicon only, no Intel fallback to reason about — the same machine that runs your interview runs the app at native speed.",
        },
      ],
      verdict:
        "These two are genuinely close, so buy on the detail that matters to you. Smog wins if you need Windows, want the models included in one lifetime payment, or export a lot. CloakView wins if you want the cheapest one-time price, no quota and no account, and you are happy to own the OpenAI key yourself.",
      faq: [
        {
          q: "Is Smog cheaper than CloakView?",
          a: "It depends on the shape you buy. Both have free tiers with the stealth overlay. Beyond that, Smog sells $24.99/month with models included or a $199 lifetime licence; CloakView gives you the app free and Pro for a one-time $99, with OpenAI billing your usage separately.",
        },
        {
          q: "Does CloakView have a token quota?",
          a: "No. There is no metered tier and no fair-use throttle in the app; the only limit is the spending limit you set inside your OpenAI account.",
        },
        {
          q: "Which one runs on Windows?",
          a: "Smog has a Windows 10/11 x64 beta. CloakView is macOS 14.2+ on Apple Silicon only, with no Windows build.",
        },
        {
          q: "Can I bring my own key to both?",
          a: "Yes, but on different plans. Smog offers it on the Lifetime licence, while Pro bundles their models. CloakView only works with your own OpenAI key, in every tier.",
        },
      ],
    },
    "final-round-ai": {
      name: "Final Round AI",
      vendorUrl: "https://www.finalroundai.com/blog/final-round-ai-pricing",
      title: "CloakView vs Final Round AI: cost and stealth",
      description:
        "Final Round AI charges $25 to $90 a month for live Copilot sessions and has no free plan. CloakView is free, with a one-time $99 Pro.",
      h1: "CloakView vs Final Round AI",
      standfirst:
        "One is an interview-prep suite with a live copilot attached. The other is only the live copilot.",
      intro: [
        "Final Round AI is a preparation platform: goals, mock interviews, résumé work, debriefs and performance scores, with a live Interview Copilot in the middle of it. CloakView is deliberately smaller — a Mac overlay that transcribes the call, answers questions and never appears in a screen share.",
        "The pricing quoted here is Final Round's own, published in November 2026: there is no permanent free plan, only a 10-minute trial session; Pro costs $25/month billed annually ($300/year), $60/month billed quarterly ($180 per quarter), or $90/month billed monthly, which is non-refundable. That is the number to weigh against a free app plus $99 once for Pro.",
      ],
      rows: [
        {
          label: "Cost to start",
          we: "Free app. No account, no card.",
          they: "No permanent free plan: a 10-minute trial session.",
        },
        {
          label: "Ongoing cost",
          we: "No subscription. OpenAI usage at public rates, billed by OpenAI. Pro is $99 once.",
          they: "$25/month billed annually ($300/year), $60/month quarterly, or $90/month monthly and non-refundable.",
        },
        {
          label: "Hiding the overlay from a screen share",
          we: "Included in every tier, free app included.",
          they: "Stealth Mode belongs to their paid tiers rather than the trial.",
        },
        {
          label: "Whose models answer",
          we: "Yours. Paste your own OpenAI key.",
          they: "Theirs, bundled into the subscription.",
        },
        {
          label: "Interview preparation",
          we: "None. CloakView does not run mock interviews or score you.",
          they: "Mock interviews, goals, résumé preparation, automatic debriefs and performance scores.",
        },
        {
          label: "Languages",
          we: "OpenAI realtime transcription; live translation on Pro.",
          they: "91 languages and regional accents, on every plan.",
        },
        {
          label: "Help with coding questions",
          we: "Pro reads the problem off your screen and answers from it.",
          they: "Screen Help for coding questions, part of the desktop app.",
        },
        {
          label: "Where your data goes",
          we: "Audio and transcripts stay on your Mac. No backend, no telemetry.",
          they: "Held in your Final Round account, with transcripts and scores.",
        },
        {
          label: "Platforms",
          we: "macOS 14.2 or later, Apple Silicon.",
          they: "Desktop app plus the web.",
        },
        {
          label: "Best fit",
          we: "Mac users who want the live copilot without a prep suite or a monthly bill.",
          they: "Candidates who want the whole interview-prep package in one subscription.",
        },
      ],
      theyWin: [
        {
          title: "Practice, not just assistance",
          body: "Mock interviews, goals and scored debriefs are things CloakView does not attempt. If you want to rehearse before the real call, Final Round is the tool for that.",
        },
        {
          title: "A record of how you performed",
          body: "Full transcripts, performance scores and speech-clarity feedback after the session are part of their plans. CloakView keeps a searchable history on Pro; it does not rate you.",
        },
        {
          title: "Broad language coverage",
          body: "91 languages and regional accents across every plan is a wider claim than CloakView makes, and it does not depend on which model your key can reach.",
        },
      ],
      weWin: [
        {
          title: "No subscription for a one-off season",
          body: "A single month at their monthly rate is non-refundable; the annual plan is $300. CloakView is free, Pro is $99 once, and the only other cost is the OpenAI usage you generate.",
        },
        {
          title: "Stealth is not a plan feature",
          body: "Hiding the overlay from a screen share works in the free CloakView app. With Final Round it is tied to paid tiers, so confirm your plan includes it before an interview.",
        },
        {
          title: "No account holding your interviews",
          body: "Final Round stores transcripts and scores in your account. CloakView has no account and no server: what you said stays in files on your Mac.",
        },
        {
          title: "Smaller surface, fewer distractions",
          body: "CloakView is one overlay with two keys to remember. No prep dashboard, no scoring, nothing to browse mid-call.",
        },
      ],
      verdict:
        "If you want to practise, be scored and work through a structured job-search programme, Final Round AI is the fuller product and the subscription buys you real features. If you want the live copilot only — invisible, on your Mac, paid for once — CloakView costs a fraction of a year of Final Round and keeps your transcripts local.",
      faq: [
        {
          q: "Is Final Round AI free?",
          a: "No. Their pricing page states there is no permanent free plan; you get a 10-minute trial session, after which live Copilot sessions need a paid plan.",
        },
        {
          q: "What does a job search cost with each?",
          a: "Final Round starts at $25/month billed annually ($300/year), or $90 for a single month, which is non-refundable. CloakView is free, or $99 once for Pro, plus whatever OpenAI usage you generate.",
        },
        {
          q: "Does CloakView do mock interviews?",
          a: "No. CloakView is a live copilot: it transcribes, answers and helps while the call happens. If you want practice sessions and feedback, Final Round AI is built for exactly that.",
        },
        {
          q: "Does Final Round AI hide during a screen share?",
          a: "They offer stealth mode, tied to their paid tiers rather than the trial. CloakView includes stealth in the free app — but whichever tool you pick, test it in your own meeting setup before a real interview.",
        },
      ],
    },
  },
  es: {
    cluely: {
      name: "Cluely",
      vendorUrl: "https://cluely.com/pricing",
      title: "CloakView vs Cluely: precio y privacidad",
      description:
        "Cluely vende la invisibilidad como plan superior aparte. CloakView oculta la ventana en la app gratuita y tú le pagas a OpenAI tu propio consumo.",
      h1: "CloakView vs Cluely",
      standfirst:
        "Cluely vende la invisibilidad como su plan más caro. CloakView oculta la ventana en la app gratuita.",
      intro: [
        "Los dos productos hacen lo mismo en el mismo sitio: una IA que escucha una llamada en directo y pone una respuesta en tu pantalla mientras la otra parte no ve nada. La diferencia está en el modelo de negocio, y se nota en tres puntos: qué cuesta la invisibilidad, qué modelos responden y si tus transcripciones salen de tu Mac.",
        "Todo lo que decimos de Cluely viene de su página de precios pública en noviembre de 2026: un plan Starter gratuito, una suscripción Pro mensual o anual (desde 11,99 USD/mes con facturación anual) y un plan Pro + Undetectability aparte que añade el ocultamiento durante la pantalla compartida. El lado de CloakView describe lo que la app hace hoy tal como se distribuye.",
      ],
      rows: [
        {
          label: "Coste inicial",
          we: "App gratuita. Sin cuenta, sin tarjeta, sin reloj de prueba.",
          they: "Plan Starter gratuito, con un número limitado de respuestas de IA y notas limitadas.",
        },
        {
          label: "Coste recurrente",
          we: "Sin suscripción. Le pagas a OpenAI el consumo que generes, a sus tarifas públicas. CloakView Pro es un único pago de 99 USD.",
          they: "Suscripción Pro, mensual o anual. Su página anuncia Pro desde 11,99 USD/mes con facturación anual.",
        },
        {
          label: "Ocultar la ventana en una pantalla compartida",
          we: "Incluido en todos los planes, también en el gratuito. Es la razón de ser de la app.",
          they: "Un plan superior aparte: Pro + Undetectability. En Starter y Pro la invisibilidad no está incluida.",
        },
        {
          label: "Qué modelos responden",
          we: "Los tuyos. Pegas tu propia clave de OpenAI y la app llama a OpenAI directamente.",
          they: "Los suyos. Modelos e infraestructura de Cluely, incluidos en la suscripción.",
        },
        {
          label: "Cuenta",
          we: "Ninguna. No hay nada que registrar ni panel al que entrar.",
          they: "Una cuenta de Cluely vinculada a la suscripción.",
        },
        {
          label: "Dónde van tus datos",
          we: "El audio y las transcripciones se quedan en tu Mac. No hay backend de CloakView ni telemetría.",
          they: "Se procesan en el servicio de Cluely, según su política de privacidad.",
        },
        {
          label: "Plataformas",
          we: "macOS 14.2 o posterior, Apple Silicon.",
          they: "App de escritorio para macOS, y una app móvil en su web.",
        },
        {
          label: "Transcripción",
          we: "Transcripción en vivo con los modelos de voz de OpenAI en tiempo real; la traducción simultánea es parte de Pro.",
          they: "Transcripción en vivo y respuestas de IA ilimitadas en Pro.",
        },
        {
          label: "Leer tu pantalla",
          we: "Pro lee una pregunta o una tarea directamente de tu pantalla.",
          they: "Sí: funciones de visión en los planes de pago.",
        },
        {
          label: "Notas e historial",
          we: "Notas automáticas y manuales; historial con búsqueda en Pro.",
          they: "Notas de reunión, ilimitadas en Pro.",
        },
        {
          label: "Para quién es",
          we: "Usuarios de Mac que quieren una herramienta privada, pagada una vez y bajo su control.",
          they: "Quien prefiere un producto alojado, en varios dispositivos, y una suscripción antes que gestionar una clave de API.",
        },
      ],
      theyWin: [
        {
          title: "Nada que configurar",
          body: "Te suscribes, descargas e inicias sesión. No hay panel de OpenAI que visitar, ni clave que pegar, ni límite de gasto que ajustar; para mucha gente, eso es justo por lo que paga software.",
        },
        {
          title: "También funciona en el móvil",
          body: "CloakView es una app de Mac y nada más. Si quieres el mismo asistente en tu teléfono, Cluely es el que lo tiene.",
        },
        {
          title: "Ellos mantienen los modelos",
          body: "Cluely cambia de modelo en su lado según salen nuevos. En CloakView eliges tú el modelo y asumes tú el coste de esa elección.",
        },
      ],
      weWin: [
        {
          title: "La invisibilidad no es un extra",
          body: "Ocultar la ventana a una pantalla compartida es aquello para lo que está construido CloakView, y funciona en la app gratuita. En Cluely ese comportamiento es el plan más caro.",
        },
        {
          title: "Sin suscripción que recordar",
          body: "La app es gratis y Pro es un único pago de 99 USD. El único coste recurrente es el consumo de OpenAI, facturado por OpenAI a las tarifas que publica.",
        },
        {
          title: "Tus entrevistas se quedan en tu disco",
          body: "No hay servidor de CloakView guardando grabaciones ni transcripciones. Si desinstalas la app, lo que hay en tu Mac es todo lo que existe.",
        },
        {
          title: "La clave y el modelo son tuyos",
          body: "Revocas la clave y la app simplemente se detiene. Cambias de modelo cuando aparezca uno mejor. No hay proceso de cancelación porque no hay nada que cancelar.",
        },
      ],
      verdict:
        "Si quieres un producto alojado, con app móvil y cero configuración, Cluely es una compra razonable; eso sí, comprueba qué plan oculta de verdad la ventana antes de una entrevista. Si lo que quieres es un copiloto invisible en tu Mac, pagado una vez y cuyas transcripciones no salen de la máquina, CloakView es más barato y más privado, y la invisibilidad no es una mejora de plan.",
      faq: [
        {
          q: "¿CloakView es un clon de Cluely?",
          a: "No. Los dos ponen una respuesta de IA en pantalla durante una llamada, pero CloakView es solo para macOS, no tiene backend ni cuenta y funciona con tu propia clave de OpenAI. El ocultamiento viene de serie en vez de venderse aparte.",
        },
        {
          q: "¿CloakView es más barato que Cluely?",
          a: "Para la mayoría, sí. La app de CloakView es gratis y Pro es un pago único de 99 USD; el único coste recurrente es el consumo de OpenAI que generes. Cluely cobra suscripción y la invisibilidad está en su plan más alto.",
        },
        {
          q: "¿Puede CloakView usar Claude o Gemini en vez de OpenAI?",
          a: "Hoy no. CloakView habla con OpenAI y la clave la pones tú, así que la lista de modelos es la de OpenAI. Cluely elige los modelos en su propia infraestructura.",
        },
        {
          q: "¿CloakView funciona en Windows?",
          a: "No. CloakView requiere macOS 14.2 o posterior en Apple Silicon. Cluely tiene app para macOS y app móvil.",
        },
      ],
    },
    "smog-ai": {
      name: "Smog AI",
      vendorUrl: "https://smog-ai.com/pricing",
      title: "CloakView vs Smog AI: precio e invisibilidad",
      description:
        "Smog AI incluye la invisibilidad en su plan gratuito y vende una licencia de 199 USD. CloakView es gratis, con Pro de 99 USD una sola vez.",
      h1: "CloakView vs Smog AI",
      standfirst:
        "El par más cercano del sitio: las dos son ventanas nativas de Mac invisibles a la pantalla compartida.",
      intro: [
        "Smog y CloakView son vecinos cercanos. Las dos son ventanas nativas de Mac que se ocultan al compartir pantalla, las dos guardan las transcripciones en tu máquina y las dos pueden funcionar con tu propia clave de OpenAI en lugar de una suscripción con modelos incluidos. Las diferencias están en la forma del precio y en los techos.",
        "Los planes de Smog, según su página de precios en noviembre de 2026: un plan gratuito con la ventana invisible y una cuota limitada de tokens, Pro a 24,99 USD/mes (menos con facturación anual) con sus modelos incluidos, y una licencia de por vida de 199 USD, listada a 299 USD y limitada a 50 plazas, que sí permite traer tu clave de OpenAI. CloakView no tiene cuota ni cuenta de ningún tipo.",
      ],
      rows: [
        {
          label: "Coste inicial",
          we: "App gratuita. Sin cuenta, sin tarjeta.",
          they: "Plan gratuito, con la ventana invisible incluida y una cuota limitada de tokens.",
        },
        {
          label: "Coste recurrente",
          we: "Sin suscripción. Le pagas a OpenAI tu propio consumo. CloakView Pro es un único pago de 99 USD.",
          they: "Pro a 24,99 USD/mes, o licencia de por vida de 199 USD (listada a 299 USD, 50 plazas).",
        },
        {
          label: "Ocultar la ventana en una pantalla compartida",
          we: "Incluido en todos los planes, también en el gratuito.",
          they: "Incluido desde el plan gratuito. Es su punto más fuerte, y es justo reconocerlo.",
        },
        {
          label: "Tu propia clave de API",
          we: "Sí, para todas las funciones. Es la única forma en que funciona.",
          they: "En el plan de por vida. Pro incluye acceso a sus modelos en su lugar.",
        },
        {
          label: "Cuota de uso",
          we: "Ninguna. Tu único techo es el límite de gasto que fijes en tu cuenta de OpenAI.",
          they: "El plan gratuito tiene una cuota de tokens; los de pago son ilimitados bajo política de uso razonable.",
        },
        {
          label: "Dónde van tus datos",
          we: "Audio y transcripciones en tu Mac. Sin backend de CloakView, sin telemetría.",
          they: "Interfaz, transcripciones y notas guardadas en local; las peticiones a la IA envían el texto mínimo necesario, que dicen no almacenar.",
        },
        {
          label: "Plataformas",
          we: "macOS 14.2 o posterior, Apple Silicon.",
          they: "macOS 14.2 o posterior (Apple Silicon; Intel si el Mac corre 14.2), más una beta de Windows 10/11 x64.",
        },
        {
          label: "Idiomas",
          we: "Transcripción en tiempo real de OpenAI; traducción simultánea en Pro.",
          they: "Detección automática de idioma y traducción en tiempo real.",
        },
        {
          label: "Leer tu pantalla",
          we: "Pro lee una pregunta o tarea directamente de tu pantalla.",
          they: "Vision: análisis de pantalla, limitado por la cuota de tokens en el plan gratuito.",
        },
        {
          label: "Notas y exportación",
          we: "Notas automáticas y manuales; historial con búsqueda en Pro.",
          they: "Notas automáticas y manuales, con exportación a PDF, Markdown y correo en los planes de pago.",
        },
        {
          label: "Para quién es",
          we: "Usuarios solo de Mac que quieren un pago, ninguna cuenta y su propia clave desde el primer arranque.",
          they: "Quien necesita una beta de Windows o una licencia de por vida con los modelos incluidos.",
        },
      ],
      theyWin: [
        {
          title: "Licencia de por vida con modelos incluidos",
          body: "Los 199 USD de Smog son un pago por la app y por la IA que hay detrás. Si prefieres no abrir nunca un panel de OpenAI, es una oferta coherente; CloakView siempre necesita una clave.",
        },
        {
          title: "Windows, en beta",
          body: "Smog tiene una beta para Windows 10/11 x64. CloakView es solo macOS en Apple Silicon, así que si tu portátil de trabajo es un PC, la comparación termina ahí.",
        },
        {
          title: "Más formatos de exportación",
          body: "La exportación a PDF, Markdown y correo en los planes de pago está documentada en su página de precios. El historial de CloakView se puede buscar, pero su superficie de exportación es más estrecha.",
        },
      ],
      weWin: [
        {
          title: "La mitad del precio de la licencia de por vida",
          body: "Pro son 99 USD una vez frente a los 199 USD del plan de por vida de Smog, y te quedas con la clave: el modelo y el coste por token siguen siendo tuyos.",
        },
        {
          title: "Ninguna cuota de tokens",
          body: "La app gratuita de CloakView no tiene plan medido ni freno por uso razonable. El único límite es el de gasto que fijes en OpenAI, que puedes cambiar cuando quieras.",
        },
        {
          title: "Sin cuenta ni inicio de sesión",
          body: "No hay cuenta de CloakView que crear, verificar o perder. Pegas la clave y funciona; la revocas y se detiene.",
        },
        {
          title: "Nada que desaprender en Mac",
          body: "Un DMG, solo Apple Silicon, sin respaldo Intel del que preocuparse: la misma máquina que hace la entrevista mueve la app a velocidad nativa.",
        },
      ],
      verdict:
        "Estos dos están de verdad cerca, así que compra por el detalle que te importe. Smog gana si necesitas Windows, quieres los modelos incluidos en un pago único o exportas mucho. CloakView gana si quieres el precio único más bajo, sin cuota y sin cuenta, y no te importa ser tú quien lleva la clave de OpenAI.",
      faq: [
        {
          q: "¿Smog es más barato que CloakView?",
          a: "Depende de la forma que compres. Los dos tienen plan gratuito con la ventana invisible. A partir de ahí, Smog vende 24,99 USD/mes con modelos incluidos o 199 USD de por vida; CloakView te da la app gratis y Pro por 99 USD una vez, con OpenAI facturándote el consumo aparte.",
        },
        {
          q: "¿CloakView tiene cuota de tokens?",
          a: "No. No hay plan medido ni freno por uso razonable en la app; el único límite es el de gasto que configures en tu cuenta de OpenAI.",
        },
        {
          q: "¿Cuál funciona en Windows?",
          a: "Smog tiene una beta para Windows 10/11 x64. CloakView es solo macOS 14.2+ en Apple Silicon, sin versión para Windows.",
        },
        {
          q: "¿Puedo usar mi propia clave en los dos?",
          a: "Sí, pero en planes distintos. Smog lo permite en la licencia de por vida, mientras que Pro incluye sus modelos. CloakView solo funciona con tu clave de OpenAI, en todos los planes.",
        },
      ],
    },
    "final-round-ai": {
      name: "Final Round AI",
      vendorUrl: "https://www.finalroundai.com/blog/final-round-ai-pricing",
      title: "CloakView vs Final Round AI: coste y datos",
      description:
        "Final Round AI cobra de 25 a 90 USD al mes por sesiones en vivo y no tiene plan gratis. CloakView es gratis, con Pro de 99 USD una vez.",
      h1: "CloakView vs Final Round AI",
      standfirst:
        "Una es una suite de preparación de entrevistas con copiloto en vivo. La otra es solo el copiloto en vivo.",
      intro: [
        "Final Round AI es una plataforma de preparación: objetivos, entrevistas simuladas, trabajo de currículum, informes y puntuaciones, con un Interview Copilot en vivo en el centro. CloakView es a propósito más pequeño: una ventana de Mac que transcribe la llamada, responde preguntas y nunca aparece en una pantalla compartida.",
        "Los precios que citamos son los de Final Round, publicados en noviembre de 2026: no hay plan gratuito permanente, solo una sesión de prueba de 10 minutos; Pro cuesta 25 USD/mes con facturación anual (300 USD al año), 60 USD/mes trimestral (180 USD por trimestre) o 90 USD/mes mensual, que no es reembolsable. Ese es el número que hay que comparar con una app gratis más 99 USD una vez por Pro.",
      ],
      rows: [
        {
          label: "Coste inicial",
          we: "App gratuita. Sin cuenta, sin tarjeta.",
          they: "Sin plan gratuito permanente: una sesión de prueba de 10 minutos.",
        },
        {
          label: "Coste recurrente",
          we: "Sin suscripción. Consumo de OpenAI a tarifas públicas, facturado por OpenAI. Pro son 99 USD una vez.",
          they: "25 USD/mes anual (300 USD al año), 60 USD/mes trimestral, o 90 USD/mes mensual y no reembolsable.",
        },
        {
          label: "Ocultar la ventana en una pantalla compartida",
          we: "Incluido en todos los planes, también en el gratuito.",
          they: "El modo invisible pertenece a sus planes de pago, no a la prueba.",
        },
        {
          label: "Qué modelos responden",
          we: "Los tuyos. Pegas tu propia clave de OpenAI.",
          they: "Los suyos, incluidos en la suscripción.",
        },
        {
          label: "Preparación de entrevistas",
          we: "Ninguna. CloakView no hace entrevistas simuladas ni te puntúa.",
          they: "Entrevistas simuladas, objetivos, preparación de currículum, informes automáticos y puntuaciones.",
        },
        {
          label: "Idiomas",
          we: "Transcripción en tiempo real de OpenAI; traducción simultánea en Pro.",
          they: "91 idiomas y acentos regionales, en todos los planes.",
        },
        {
          label: "Ayuda con preguntas de código",
          we: "Pro lee el problema desde tu pantalla y responde a partir de él.",
          they: "Screen Help para preguntas de código, dentro de su app de escritorio.",
        },
        {
          label: "Dónde van tus datos",
          we: "Audio y transcripciones se quedan en tu Mac. Sin backend, sin telemetría.",
          they: "En tu cuenta de Final Round, con transcripciones y puntuaciones.",
        },
        {
          label: "Plataformas",
          we: "macOS 14.2 o posterior, Apple Silicon.",
          they: "App de escritorio y web.",
        },
        {
          label: "Para quién es",
          we: "Usuarios de Mac que quieren el copiloto en vivo sin suite de preparación ni factura mensual.",
          they: "Candidatos que quieren todo el paquete de preparación en una sola suscripción.",
        },
      ],
      theyWin: [
        {
          title: "Practicar, no solo asistir",
          body: "Las entrevistas simuladas, los objetivos y los informes puntuados son cosas que CloakView no intenta. Si quieres ensayar antes de la llamada real, Final Round es la herramienta para eso.",
        },
        {
          title: "Un registro de cómo lo hiciste",
          body: "Transcripciones completas, puntuaciones de desempeño y claridad del habla después de la sesión forman parte de sus planes. CloakView guarda un historial con búsqueda en Pro; no te puntúa.",
        },
        {
          title: "Cobertura amplia de idiomas",
          body: "91 idiomas y acentos regionales en todos los planes es una promesa más amplia que la de CloakView, y no depende del modelo al que llegue tu clave.",
        },
      ],
      weWin: [
        {
          title: "Sin suscripción para una temporada corta",
          body: "Un solo mes a su tarifa mensual no es reembolsable, y el plan anual son 300 USD. CloakView es gratis, Pro son 99 USD una vez, y el único otro coste es el consumo de OpenAI que generes.",
        },
        {
          title: "La invisibilidad no es una función de plan",
          body: "Ocultar la ventana en una pantalla compartida funciona en la app gratuita de CloakView. En Final Round depende de planes de pago, así que confirma que el tuyo la incluye antes de una entrevista.",
        },
        {
          title: "Ninguna cuenta guardando tus entrevistas",
          body: "Final Round almacena transcripciones y puntuaciones en tu cuenta. CloakView no tiene cuenta ni servidor: lo que dijiste se queda en archivos de tu Mac.",
        },
        {
          title: "Menos superficie, menos distracciones",
          body: "CloakView es una ventana con dos atajos que recordar. Sin panel de preparación, sin puntuaciones, nada que explorar en mitad de la llamada.",
        },
      ],
      verdict:
        "Si quieres practicar, recibir puntuaciones y seguir un programa estructurado de búsqueda de empleo, Final Round AI es el producto más completo y la suscripción te compra funciones reales. Si solo quieres el copiloto en vivo —invisible, en tu Mac y pagado una vez— CloakView cuesta una fracción de un año de Final Round y mantiene tus transcripciones en local.",
      faq: [
        {
          q: "¿Final Round AI es gratis?",
          a: "No. Su página de precios dice que no hay plan gratuito permanente: hay una sesión de prueba de 10 minutos y, después, las sesiones en vivo requieren un plan de pago.",
        },
        {
          q: "¿Cuánto cuesta una búsqueda de empleo con cada uno?",
          a: "Final Round empieza en 25 USD/mes con facturación anual (300 USD al año), o 90 USD por un solo mes, que no es reembolsable. CloakView es gratis, o 99 USD una vez por Pro, más el consumo de OpenAI que generes.",
        },
        {
          q: "¿CloakView hace entrevistas simuladas?",
          a: "No. CloakView es un copiloto en vivo: transcribe, responde y ayuda mientras ocurre la llamada. Si quieres sesiones de práctica con retroalimentación, Final Round AI está hecho exactamente para eso.",
        },
        {
          q: "¿Final Round AI se oculta al compartir pantalla?",
          a: "Ofrecen modo invisible, vinculado a sus planes de pago y no a la prueba. CloakView incluye la invisibilidad en la app gratuita; en cualquier caso, prueba la herramienta en tu propia configuración de reunión antes de una entrevista real.",
        },
      ],
    },
  },
};
