import { getDictionary } from '@/lib/get-dictionary';
import { LegalLayout } from '@/components/LegalLayout';
import { LegalText } from '@/components/legal-text';
import { Metadata } from 'next';
import { siteConfig } from '@/site.config';

interface Props {
  params: Promise<{ locale: 'en' | 'es' }>;
}

export async function generateMetadata(
  props: Props
): Promise<Metadata> {
  const params = await props.params;
  const dict = await getDictionary(params.locale);

  const title = `${dict.cookies.title} — CloakView`;
  const description = dict.cookies.description;

  return {
    title,
    description,
    alternates: {
      languages: {
        en: '/en/cookies',
        es: '/es/cookies',
      },
    },
    openGraph: {
      title,
      description,
      url: `${siteConfig.url}/${params.locale}/cookies`,
      type: 'website',
    },
  };
}

export default async function CookiesPage(props: Props) {
  const params = await props.params;
  const dict = await getDictionary(params.locale);
  const c = dict.cookies;

  return (
    <LegalLayout
      title={c.title}
      updated={c.updated}
      jurisdiction={c.jurisdiction}
      locale={params.locale}
    >
      <section className="space-y-8">
        {/* Overview */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{c.sections.overview.title}</h2>
          <p className="text-muted whitespace-pre-wrap leading-relaxed">
            {c.sections.overview.content}
          </p>
        </div>

        {/* Why No */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{c.sections.whyNo.title}</h2>
          <p className="text-muted whitespace-pre-wrap leading-relaxed">
            {c.sections.whyNo.content}
          </p>
        </div>

        {/* Website Cookies */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{c.sections.websiteCookies.title}</h2>
          <p className="text-muted whitespace-pre-wrap leading-relaxed">
            {c.sections.websiteCookies.content}
          </p>
        </div>

        {/* Third Party Scripts */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{c.sections.thirdPartyScripts.title}</h2>
          <p className="text-muted whitespace-pre-wrap leading-relaxed">
            {c.sections.thirdPartyScripts.content}
          </p>
        </div>

        {/* Local Storage */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{c.sections.localStorage.title}</h2>
          <p className="text-muted whitespace-pre-wrap leading-relaxed">
            {c.sections.localStorage.content}
          </p>
        </div>

        {/* If We Change */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{c.sections.ifWeChange.title}</h2>
          <p className="text-muted whitespace-pre-wrap leading-relaxed">
            {c.sections.ifWeChange.content}
          </p>
        </div>

        {/* Third Party Links */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{c.sections.thirdPartyLinks.title}</h2>
          <p className="text-muted whitespace-pre-wrap leading-relaxed">
            {c.sections.thirdPartyLinks.content}
          </p>
        </div>

        {/* Questions */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{c.sections.questions.title}</h2>
          <LegalText className="text-muted" text={c.sections.questions.content} />
        </div>
      </section>
    </LegalLayout>
  );
}
