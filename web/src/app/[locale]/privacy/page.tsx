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

  const title = `${dict.privacy.title} — CloakView`;
  const description = dict.privacy.description;

  return {
    title,
    description,
    alternates: {
      languages: {
        en: '/en/privacy',
        es: '/es/privacy',
      },
    },
    openGraph: {
      title,
      description,
      url: `${siteConfig.url}/${params.locale}/privacy`,
      type: 'website',
    },
  };
}

export default async function PrivacyPage(props: Props) {
  const params = await props.params;
  const dict = await getDictionary(params.locale);
  const p = dict.privacy;

  return (
    <LegalLayout
      title={p.title}
      updated={p.updated}
      jurisdiction={p.jurisdiction}
      locale={params.locale}
    >
      <section className="space-y-8">
        {/* Overview */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{p.sections.overview.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {p.sections.overview.content}
          </p>
        </div>

        {/* Data Collection */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{p.sections.dataCollection.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {p.sections.dataCollection.content}
          </p>
        </div>

        {/* OpenAI Processing */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{p.sections.openaiProcessing.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {p.sections.openaiProcessing.content}
          </p>
        </div>

        {/* Your Key */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{p.sections.yourKey.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {p.sections.yourKey.content}
          </p>
        </div>

        {/* Tracking */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{p.sections.tracking.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {p.sections.tracking.content}
          </p>
        </div>

        {/* Storage */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{p.sections.storage.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {p.sections.storage.content}
          </p>
        </div>

        {/* Third Parties */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{p.sections.thirdParties.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {p.sections.thirdParties.content}
          </p>
        </div>

        {/* Data Rights */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{p.sections.dataRights.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {p.sections.dataRights.content}
          </p>
        </div>

        {/* App Updates */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{p.sections.appUpdates.title}</h2>
          <p className="text-slate-300 whitespace-pre-wrap leading-relaxed">
            {p.sections.appUpdates.content}
          </p>
        </div>

        {/* Contact */}
        <div>
          <h2 className="text-2xl font-bold mb-4">{p.sections.contacts.title}</h2>
          <LegalText className="text-slate-300" text={p.sections.contacts.content} />
        </div>
      </section>
    </LegalLayout>
  );
}
