/**
 * Guia da chave gratuita do Groq no cadastro do motor (docs/ai-agent.md › Groq): os três
 * passos no console do Groq, os limites do plano gratuito, o agente enxuto e o que a página
 * de dados do Groq diz sobre privacidade, com a data em que foi conferido.
 */
import { formatDate } from '@/lib/format';
import { t } from '@/i18n';

/** Quando as fontes do Groq foram conferidas (limites, modelos, dados). */
export const GROQ_CHECKED = '2026-10-20';

const CONSOLE = 'https://console.groq.com/keys';
const DATA_POLICY = 'https://console.groq.com/docs/your-data';

const link = 'text-accent underline-offset-2 hover:underline';

export function GroqGuide() {
  return (
    <section
      aria-label={t('groq-guia-titulo')}
      className="flex flex-col gap-3 rounded-card border border-line bg-surface-2 p-3 text-[12.5px] leading-relaxed"
    >
      <h3 className="text-[13px] font-medium text-fg">{t('groq-guia-titulo')}</h3>
      <ol className="flex list-decimal flex-col gap-1 pl-5 text-fg">
        <li>{t('groq-guia-passo-1')}</li>
        <li>{t('groq-guia-passo-2')}</li>
        <li>{t('groq-guia-passo-3')}</li>
      </ol>
      <a className={link} href={CONSOLE} target="_blank" rel="noreferrer">
        {t('groq-guia-abrir')}
      </a>
      <div>
        <p className="label-caps text-[10.5px]">{t('groq-guia-limites-titulo')}</p>
        <p className="text-dim">{t('groq-guia-limites')}</p>
      </div>
      <div>
        <p className="label-caps text-[10.5px]">{t('groq-guia-enxuto-titulo')}</p>
        <p className="text-dim">{t('groq-guia-enxuto')}</p>
      </div>
      <div>
        <p className="label-caps text-[10.5px]">
          {t('groq-guia-privacidade-titulo', { data: formatDate(GROQ_CHECKED) })}
        </p>
        <p className="text-dim">{t('groq-guia-privacidade')}</p>
        <a className={link} href={DATA_POLICY} target="_blank" rel="noreferrer">
          {t('groq-guia-fonte')}
        </a>
      </div>
    </section>
  );
}
