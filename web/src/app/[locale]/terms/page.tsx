import { getDictionary } from '@/lib/get-dictionary';
import { LegalLayout } from '@/components/LegalLayout';
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

  const title = `${dict.terms.title} — CloakView`;
  const description = dict.terms.description;

  return {
    title,
    description,
    alternates: {
      languages: {
        en: '/en/terms',
        es: '/es/terms',
      },
    },
    openGraph: {
      title,
      description,
      url: `${siteConfig.url}/${params.locale}/terms`,
      type: 'website',
    },
  };
}

export default async function TermsPage(props: Props) {
  const params = await props.params;
  const dict = await getDictionary(params.locale);
  const t = dict.terms;

  return (
    <LegalLayout
      title={t.title}
      updated={t.updated}
      jurisdiction={t.jurisdiction}
      locale={params.locale}
    >
      <section className="space-y-8">
        {/* Acceptance */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{t.sections.acceptance.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {t.sections.acceptance.content}
          </p>
        </div>

        {/* License */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{t.sections.license.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {t.sections.license.content}
          </p>
        </div>

        {/* Your API Key */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{t.sections.yourApiKey.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {t.sections.yourApiKey.content}
          </p>
        </div>

        {/* Liability */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{t.sections.liability.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {t.sections.liability.content}
          </p>
        </div>

        {/* Legal Entity */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{t.sections.legalEntity.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {t.sections.legalEntity.content}
          </p>
        </div>

        {/* Intellectual Property */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{t.sections.intellectualProperty.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {t.sections.intellectualProperty.content}
          </p>
        </div>

        {/* Indemnification */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{t.sections.indemnification.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {t.sections.indemnification.content}
          </p>
        </div>

        {/* Modifications */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{t.sections.modifications.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {t.sections.modifications.content}
          </p>
        </div>

        {/* Governing */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{t.sections.governing.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {t.sections.governing.content}
          </p>
        </div>

        {/* Severability */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{t.sections.severability.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {t.sections.severability.content}
          </p>
        </div>

        {/* Entire Agreement */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{t.sections.entireAgreement.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {t.sections.entireAgreement.content}
          </p>
        </div>
      </section>
    </LegalLayout>
  );
}
