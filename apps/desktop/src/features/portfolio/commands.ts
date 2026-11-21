/**
 * Comandos globais da carteira e dos ativos na barra ⌘K: nova operação, provento,
 * cadastrar ativo, calcular renda fixa, atualizar cotações e "abrir ativo" por ticker ou
 * nome (a lista de ativos só é lida quando a barra abre).
 */
import { useMemo } from 'react';
import { useNavigate } from 'react-router';

import { useAssets } from '@/api/hooks/assets';
import { useCountryFeatures } from '@/api/hooks/country';
import { useCommandBar, useRegisterCommands, type AppCommand } from '@/commands/registry';
import { t } from '@/i18n';

import { classLabel } from './labels';

export function usePortfolioCommands(): void {
  const navigate = useNavigate();
  const barOpen = useCommandBar((s) => s.open);
  const assets = useAssets({}, { enabled: barOpen });
  const brMarket = useCountryFeatures().br_market;

  const commands = useMemo<AppCommand[]>(() => {
    const go = (path: string) => () => void navigate(path);
    // B3, notas de corretagem e as análises com dados da CVM só no Brasil.
    const base: (AppCommand & { brOnly?: boolean })[] = [
      {
        id: 'investimentos.operacao',
        title: t('carteira-cmd-operacao'),
        group: t('casca-grupo-investimentos'),
        icon: 'plus',
        order: 0,
        keywords: t('carteira-cmd-operacao-busca').split(','),
        perform: go('/carteira?nova=operacao'),
      },
      {
        id: 'investimentos.provento',
        title: t('carteira-cmd-provento'),
        group: t('casca-grupo-investimentos'),
        order: 1,
        keywords: t('carteira-cmd-provento-busca').split(','),
        perform: go('/carteira?novo=provento'),
      },
      {
        id: 'investimentos.proventos',
        title: t('carteira-cmd-proventos'),
        group: t('casca-grupo-investimentos'),
        order: 1.05,
        keywords: t('carteira-cmd-proventos-busca').split(','),
        perform: go('/carteira?novo=proventos'),
      },
      {
        brOnly: true,
        id: 'investimentos.importar_b3',
        title: t('carteira-cmd-importar-b3'),
        group: t('casca-grupo-investimentos'),
        order: 1.1,
        keywords: t('carteira-cmd-importar-b3-busca').split(','),
        perform: go('/carteira?aba=operacoes&importar=b3'),
      },
      {
        brOnly: true,
        id: 'investimentos.importar_notas',
        title: t('carteira-cmd-importar-notas'),
        group: t('casca-grupo-investimentos'),
        order: 1.2,
        keywords: t('carteira-cmd-importar-notas-busca').split(','),
        perform: go('/carteira?aba=operacoes&importar=notas'),
      },
      {
        id: 'investimentos.ativo',
        title: t('carteira-cadastrar-ativo'),
        group: t('casca-grupo-investimentos'),
        order: 2,
        keywords: t('carteira-cmd-ativo-busca').split(','),
        perform: go('/ativos?novo=ativo'),
      },
      {
        id: 'investimentos.cotacoes',
        title: t('carteira-cmd-cotacoes'),
        group: t('casca-grupo-investimentos'),
        icon: 'refresh',
        order: 3,
        keywords: t('carteira-cmd-cotacoes-busca').split(','),
        perform: go('/ativos?aba=indices'),
      },
      {
        id: 'investimentos.agenda',
        title: t('carteira-cmd-agenda'),
        group: t('casca-grupo-investimentos'),
        order: 4,
        keywords: t('carteira-cmd-agenda-busca').split(','),
        perform: go('/ativos?aba=agenda'),
      },
      {
        id: 'investimentos.watchlist',
        title: t('carteira-cmd-watchlist'),
        group: t('casca-grupo-investimentos'),
        order: 5,
        keywords: t('carteira-cmd-watchlist-busca').split(','),
        perform: go('/ativos?aba=watchlist'),
      },
      {
        brOnly: true,
        id: 'investimentos.rankings',
        title: t('carteira-cmd-rankings'),
        group: t('casca-grupo-investimentos'),
        order: 6,
        keywords: t('carteira-cmd-rankings-busca').split(','),
        perform: go('/mercado?aba=rankings'),
      },
      {
        brOnly: true,
        id: 'investimentos.screener',
        title: t('carteira-cmd-screener'),
        group: t('casca-grupo-investimentos'),
        order: 7,
        keywords: t('carteira-cmd-screener-busca').split(','),
        perform: go('/mercado?aba=explorador'),
      },
      {
        brOnly: true,
        id: 'investimentos.comparador',
        title: t('carteira-cmd-comparador'),
        group: t('casca-grupo-investimentos'),
        order: 8,
        keywords: t('carteira-cmd-comparador-busca').split(','),
        perform: go('/ativos?aba=comparador'),
      },
      {
        brOnly: true,
        id: 'investimentos.renda_fixa',
        title: t('carteira-rf-cmd'),
        group: t('casca-grupo-investimentos'),
        order: 8.5,
        keywords: t('carteira-rf-cmd-busca').split(','),
        perform: go('/carteira?aba=renda-fixa'),
      },
      {
        brOnly: true,
        id: 'investimentos.analise',
        title: t('carteira-cmd-analise'),
        group: t('casca-grupo-investimentos'),
        icon: 'refresh',
        order: 9,
        keywords: t('carteira-cmd-analise-busca').split(','),
        perform: go('/ativos?aba=dados'),
      },
    ];
    const open = (assets.data ?? []).map((a): AppCommand => ({
      id: `ativo.${a.id}`,
      title: a.ticker ? `${a.ticker} — ${a.name}` : a.name,
      group: t('carteira-cmd-grupo-ativos'),
      hint: classLabel(a.class, a.class_label),
      keywords: [...t('carteira-cmd-abrir-ativo-busca').split(','), a.ticker ?? '', a.name],
      searchOnly: true,
      perform: go(`/ativos?ativo=${encodeURIComponent(a.id)}`),
    }));
    return [...base.filter((c) => brMarket || !c.brOnly), ...open];
  }, [navigate, assets.data, brMarket]);

  useRegisterCommands(commands);
}
