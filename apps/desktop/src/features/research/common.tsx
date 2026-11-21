/**
 * Peças comuns da análise de ativos: link para o ativo (página do ativo cadastrado ou
 * análise por ticker), lista do universo para busca, estado vazio sem coleta e o erro de
 * "papel fora do universo".
 */
import { Link } from 'react-router';

import { isApiError } from '@/api/client';
import type { AssetClass } from '@/api/generated/AssetClass';
import { useScreen } from '@/api/hooks/research';
import { EmptyState, Icon } from '@/design/components';
import { t } from '@/i18n';

import { ErrorBlock, Notice } from '../portfolio/components/States';
import { defaultCurrency } from '../portfolio/labels';

/** Página do ativo cadastrado; sem cadastro, a análise pelo ticker. */
/** A página dedicada do ativo (docs/asset-page.md): pelo id do cadastro ou pelo ticker. */
export function subjectHref(assetId: string | null | undefined, ticker: string | null): string {
  return `/ativo/${encodeURIComponent(assetId || (ticker ?? '').toUpperCase())}`;
}

export function TickerLink({
  assetId,
  ticker,
  className,
}: {
  assetId: string | null | undefined;
  ticker: string;
  className?: string;
}) {
  return (
    <Link
      to={subjectHref(assetId, ticker)}
      className={
        className ?? 'font-medium text-fg underline-offset-2 hover:text-accent hover:underline'
      }
    >
      {ticker}
    </Link>
  );
}

/** Moeda dos papéis do universo pela classe (EUA em dólar, B3 em real). */
export const currencyOfClass = (cls: AssetClass) => defaultCurrency(cls);

const UNIVERSE_ARGS = { limit: 500, fields: [] as string[] };

/** Papéis do universo (ticker, nome, cadastro) para buscas e sugestões. */
export function useUniverseList() {
  return useScreen(UNIVERSE_ARGS);
}

/** Quem roda sobre o universo (muda a frase do estado vazio). */
export type UniverseUser = 'geral' | 'ticker' | 'screener' | 'rankings';

/** Universo vazio: explica a coleta e leva à aba de dados. */
export function NoUniverse({ what = 'geral' }: { what?: UniverseUser }) {
  return (
    <EmptyState
      kicker={t('pesquisa-sem-coleta')}
      title={t('pesquisa-sem-universo')}
      description={t('pesquisa-sem-universo-texto', { quem: what })}
      actions={
        <Link
          to="/ativos?aba=dados"
          className="inline-flex h-9 items-center gap-2 rounded-control bg-accent px-3.5 font-mono text-[11.5px] font-medium tracking-[0.08em] text-accent-ink uppercase"
        >
          <Icon name="refresh" size={12} /> {t('pesquisa-ir-dados')}
        </Link>
      }
    />
  );
}

/** Erro de uma consulta de ativo: fora do universo vira aviso com o caminho da coleta. */
export function SubjectError({ error, onRetry }: { error: unknown; onRetry?: () => void }) {
  if (isApiError(error) && error.code === 'not_found') {
    return (
      <Notice
        tone="warning"
        title={t('pesquisa-sem-dados-ativo')}
        actions={
          <Link className="text-[12.5px] text-accent underline" to="/ativos?aba=dados">
            {t('pesquisa-dados-analise')}
          </Link>
        }
      >
        {t('pesquisa-sem-dados-ativo-texto', { erro: error.message })}
      </Notice>
    );
  }
  return <ErrorBlock error={error} onRetry={onRetry} />;
}
