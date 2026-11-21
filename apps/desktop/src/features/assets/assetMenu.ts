/**
 * Menu de contexto de um ativo (linhas de 08 · Carteira › Posições e de 09 · Ativos): os
 * mesmos caminhos dos botões da página do ativo e da carteira, pela URL que essas telas já
 * leem (`?nova=operacao&ativo=`, `?ativo=&secao=`, `?ativo=&dialogo=`, `?comparar=`).
 */
import type { AssetClass } from '@/api/generated/AssetClass';
import type { ContextMenuEntry } from '@/design/components';
import { hasFundamentals } from '@/features/research/labels';
import { t } from '@/i18n';

import { assetSections } from './assetSections';
import { FUNDAMENTUS_CLASSES } from './FundamentusPanel';

export interface AssetRef {
  id: string;
  ticker: string | null;
  name: string;
  class: AssetClass;
}

const q = encodeURIComponent;

export function assetMenuItems(
  a: AssetRef,
  go: (path: string) => void,
  opts: { watched?: boolean } = {},
): ContextMenuEntry[] {
  const page = (extra = '') => `/ativos?ativo=${q(a.id)}${extra}`;
  const fundamentals =
    Boolean(a.ticker) &&
    (hasFundamentals(a.class) || (FUNDAMENTUS_CLASSES as readonly string[]).includes(a.class));
  const sections = assetSections(a.class);
  return [
    { heading: a.ticker ? `${a.ticker} · ${a.name}` : a.name },
    { label: t('ativos-menu-abrir'), icon: 'arrowRight', onSelect: () => go(page()) },
    { separator: true },
    {
      label: t('ativos-menu-nova-operacao'),
      icon: 'plus',
      onSelect: () => go(`/carteira?nova=operacao&ativo=${q(a.id)}`),
    },
    sections.income && {
      label: t('ativos-menu-registrar-provento'),
      icon: 'plus',
      onSelect: () => go(`/carteira?novo=provento&ativo=${q(a.id)}`),
    },
    sections.actions && {
      label: t('ativos-menu-evento-corporativo'),
      icon: 'plus',
      onSelect: () => go(`/carteira?novo=evento&ativo=${q(a.id)}`),
    },
    { separator: true },
    {
      label: t('ativos-menu-ver-extrato'),
      icon: 'history',
      onSelect: () => go(page('&secao=extrato')),
    },
    {
      label: t('ativos-menu-operacoes'),
      icon: 'sort',
      onSelect: () => go(page('&secao=operacoes')),
    },
    sections.income && {
      label: t('ativos-menu-proventos'),
      icon: 'sort',
      onSelect: () => go(page('&secao=proventos')),
    },
    {
      label: t('ativos-menu-grafico'),
      icon: 'reticle',
      onSelect: () => go(page('&secao=grafico')),
    },
    fundamentals && {
      label: t('ativos-menu-fundamentos'),
      icon: 'sliders',
      onSelect: () => go(page('&secao=fundamentos')),
    },
    { separator: true },
    !opts.watched && {
      label: t('ativos-menu-acompanhar'),
      icon: 'eye',
      onSelect: () => go(page('&dialogo=acompanhar')),
    },
    {
      label: t('ativos-menu-informar-cotacao'),
      icon: 'refresh',
      onSelect: () => go(page('&dialogo=cotacao')),
    },
    a.ticker
      ? {
          label: t('ativos-menu-comparar'),
          icon: 'sliders',
          onSelect: () => go(`/ativos?aba=comparador&comparar=${q(a.ticker!)}`),
        }
      : null,
    {
      label: t('ativos-menu-editar'),
      icon: 'sliders',
      onSelect: () => go(page('&dialogo=editar')),
    },
  ];
}
