/**
 * Ajustes › Sobre e apoio (docs/support.md): por que apoiar, como (GitHub Sponsors, o único
 * canal), como ajudar sem dinheiro, para onde vai o dinheiro e os compromissos. Sempre
 * disponível, também pelo menu do macOS (Treasury › Apoiar o projeto…). Sem rede: o link só
 * abre no navegador quando clicado; sem endereço, aparece como "em breve".
 */
import { useState } from 'react';

import { useSources } from '@/api/hooks/dataSources';
import { useProfile } from '@/api/hooks/profile';
import { useAppInfo } from '@/api/hooks/settings';
import { useSetSupportOptOut, useSupportInfo } from '@/api/hooks/support';
import { openExternal } from '@/app/externalLinks';
import { Button, Icon, InstrumentCard, toast } from '@/design/components';
import { TermsCard } from '@/features/terms/TermsCard';
import { QueryState, Switch } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { useCompanion } from '@/state/companion';

import { askText, bugReport } from './texts';

function Soon() {
  return <span className="label-caps text-dim">{t('apoio-em-breve')}</span>;
}

/** Copia e avisa: `done` é o título do aviso de sucesso, já traduzido. */
function copy(text: string, done: string) {
  void navigator.clipboard
    ?.writeText(text)
    .then(() => toast({ tone: 'success', title: done }))
    .catch(() => toast({ tone: 'error', title: t('apoio-copiar-erro') }));
}

/** Link pedido pela licença do Lightweight Charts (nome do site, não traduz). */
/** O nome do canal (marca, não traduz). */
const SPONSORS = 'GitHub Sponsors';
const TRADINGVIEW_HOST = 'tradingview.com';
const COINGECKO_HOST = 'coingecko.com';

export function SupportPage() {
  const q = useSupportInfo();
  const sources = useSources();
  const level = useProfile().data?.level;
  const opt = useSetSupportOptOut();
  const [shared, setShared] = useState(false);
  const version = useAppInfo().data?.version ?? '';
  return (
    <QueryState loading={q.isPending} error={q.error} label={t('comum-carregando')}>
      {q.data ? (
        <div className="flex flex-col gap-5">
          <InstrumentCard index="12.S1" title={t('apoio-por-que')}>
            <div className="flex flex-col gap-2 text-[13.5px] text-fg">
              <p>{askText(level)}</p>
              <p className="text-dim">— {q.data.channels.signature}</p>
              <p className="text-[12.5px] text-dim">{t('apoio-sem-obrigacao')}</p>
            </div>
          </InstrumentCard>

          <InstrumentCard index="12.S2" title={t('apoio-com-dinheiro')}>
            <section
              aria-label={SPONSORS}
              className="flex max-w-md flex-col gap-2 rounded-card border border-line px-4 py-3.5"
            >
              <h3 className="text-[15px] font-medium text-fg">{SPONSORS}</h3>
              <p className="text-[12px] text-dim">{t('apoio-github-taxa')}</p>
              {q.data.channels.github_sponsors_url ? (
                <Button
                  size="sm"
                  variant="secondary"
                  className="self-start"
                  onClick={() => void openExternal(q.data.channels.github_sponsors_url ?? '')}
                >
                  {t('apoio-github-abrir')}
                </Button>
              ) : (
                <Soon />
              )}
            </section>
          </InstrumentCard>

          <InstrumentCard index="12.S3" title={t('apoio-sem-dinheiro')}>
            <ul className="flex flex-col gap-2.5 text-[13px] text-fg">
              <li className="flex flex-wrap items-center gap-2">
                <span className="flex-1">{t('apoio-erro-texto')}</span>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => useCompanion.getState().ask(bugReport(version))}
                >
                  {t('apoio-erro-botao')}
                </Button>
              </li>
              <li className="flex flex-wrap items-center gap-2">
                <span className="flex-1">{t('apoio-banco-texto')}</span>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => useCompanion.getState().ask(t('apoio-banco-pedido'))}
                >
                  {t('apoio-banco-botao')}
                </Button>
              </li>
              <li className="flex flex-wrap items-center gap-2">
                <span className="flex-1">{t('apoio-repositorio-texto')}</span>
                {q.data.channels.repository_url ? (
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => void openExternal(q.data.channels.repository_url ?? '')}
                  >
                    {t('apoio-repositorio-botao')}
                  </Button>
                ) : (
                  <Soon />
                )}
              </li>
              <li className="flex flex-wrap items-center gap-2">
                <span className="flex-1">{t('apoio-indicar-texto')}</span>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => {
                    copy(t('apoio-indicacao'), t('apoio-texto-copiado'));
                    setShared(true);
                  }}
                >
                  {shared ? t('apoio-indicar-copiado') : t('apoio-indicar-botao')}
                </Button>
              </li>
            </ul>
          </InstrumentCard>

          <InstrumentCard index="12.S4" title={t('apoio-destino')}>
            <div className="flex flex-col gap-2 text-[13px] text-fg">
              <p>{t('apoio-destino-texto')}</p>
              {q.data.channels.annual_summary ? (
                <p className="text-dim">
                  {t('apoio-resumo-anual', {
                    data: q.data.channels.annual_summary.date,
                    texto: q.data.channels.annual_summary.text,
                  })}
                </p>
              ) : null}
            </div>
          </InstrumentCard>

          <InstrumentCard index="12.S5" title={t('apoio-compromissos')}>
            <div className="grid gap-4 text-[13px] text-fg @min-[820px]:grid-cols-2">
              <div>
                <h3 className="label-caps mb-1.5 text-dim">{t('apoio-compromissos-app')}</h3>
                <ul className="flex flex-col gap-1">
                  {[
                    t('apoio-compromisso-sem-anuncios'),
                    t('apoio-compromisso-sem-venda'),
                    t('apoio-compromisso-sem-rastreamento'),
                    t('apoio-compromisso-sem-afiliados'),
                    t('apoio-compromisso-nada-vira-pago'),
                  ].map((item) => (
                    <li key={item} className="flex gap-2">
                      <Icon name="check" size={12} className="mt-1 shrink-0 text-positive" />
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
              <div>
                <h3 className="label-caps mb-1.5 text-dim">{t('apoio-compromissos-autor')}</h3>
                <ul className="flex flex-col gap-1">
                  {[
                    t('apoio-compromisso-seguro'),
                    t('apoio-compromisso-falhas'),
                    t('apoio-compromisso-atualizacoes'),
                    t('apoio-compromisso-aviso'),
                  ].map((item) => (
                    <li key={item} className="flex gap-2">
                      <Icon name="check" size={12} className="mt-1 shrink-0 text-positive" />
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </InstrumentCard>

          <TermsCard />

          <InstrumentCard index="12.S6" title={t('apoio-sobre')}>
            <div className="flex flex-col gap-3">
              <p className="text-[13px] text-dim">{t('apoio-sobre-texto', { versao: version })}</p>
              <Switch
                checked={!q.data.opted_out}
                onChange={(on) => opt.mutate({ opted_out: !on })}
                label={t('apoio-lembrete')}
                description={t('apoio-lembrete-descricao')}
              />
              <div className="flex flex-col gap-1 border-t border-line pt-3 text-[12px] text-dim">
                <h3 className="label-caps">{t('apoio-dados-terceiros')}</h3>
                <p>{t('nucleo-longevidade-atribuicao-onu')}</p>
                <p>{t('nucleo-longevidade-atribuicao-ibge')}</p>
                <p>{t('apoio-atribuicao-bandeiras')}</p>
                {/* Pedido da licença do Lightweight Charts: o aviso do NOTICE e o link. */}
                <p>
                  {t('apoio-atribuicao-graficos')}{' '}
                  <a
                    href="https://www.tradingview.com/"
                    className="text-accent underline-offset-2 hover:underline"
                  >
                    {TRADINGVIEW_HOST}
                  </a>
                </p>
                {/* Pedido da API pública da CoinGecko: atribuição com link onde os dados aparecem. */}
                <p>
                  {t('apoio-atribuicao-cripto')}{' '}
                  <a
                    href="https://www.coingecko.com/"
                    className="text-accent underline-offset-2 hover:underline"
                  >
                    {COINGECKO_HOST}
                  </a>
                </p>
                {/* MIT, BSD, Apache e OFL pedem os avisos junto do binário: THIRD_PARTY_LICENSES.md
                    vai nos instaladores (bundle.resources) e na raiz do repositório. */}
                <p>{t('apoio-atribuicao-outros')}</p>
                {/* O texto que cada fonte de dados pede (licença, atribuição ou aviso), o mesmo de
                    Ajustes › Fontes de dados, reunido aqui. */}
                {sources.data?.length ? (
                  <details>
                    <summary className="cursor-pointer text-fg">
                      {t('apoio-fontes-atribuicoes')}
                    </summary>
                    <ul
                      aria-label={t('apoio-fontes-atribuicoes')}
                      className="mt-2 flex flex-col gap-1.5"
                    >
                      {sources.data.map((s) => (
                        <li key={s.id}>
                          <span className="text-fg">{s.name}:</span> {s.terms}
                        </li>
                      ))}
                    </ul>
                  </details>
                ) : null}
                {q.data.channels.repository_url ? (
                  <p>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() =>
                        void openExternal(
                          `${q.data.channels.repository_url}/blob/main/THIRD_PARTY_LICENSES.md`,
                        )
                      }
                    >
                      {t('apoio-licencas-terceiros-abrir')}
                    </Button>
                  </p>
                ) : null}
              </div>
            </div>
          </InstrumentCard>
        </div>
      ) : null}
    </QueryState>
  );
}
