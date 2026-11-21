/**
 * Hooks de `support.*` (docs/support.md): os canais de apoio (embutidos no build) e o lembrete
 * da Visão geral. Nada aqui faz rede: os links só abrem no navegador quando clicados.
 */
import { useEffect } from 'react';

import { useUi } from '@/state/ui';

import type { SupportChoice } from '../generated/SupportChoice';
import type { SupportInfo } from '../generated/SupportInfo';
import type { SupportPrompt } from '../generated/SupportPrompt';
import { call } from '../client';
import { useCore, useCoreMutation } from './coreQuery';

export const useSupportInfo = () =>
  useCore<SupportInfo>('support', 'info', undefined, { staleTime: Infinity });

/** O cartão da Visão geral (só aparece depois de um evento de valor). */
export const useSupportPrompt = (viewAs: boolean) =>
  useCore<SupportPrompt>('support', 'prompt', viewAs ? { view_as: true } : {}, {
    staleTime: 0,
    refetchOnMount: 'always',
  });

export const useSupportRespond = () =>
  useCoreMutation<{ choice: SupportChoice }, null>('support.respond', { affects: ['support'] });

export const useSetSupportOptOut = () =>
  useCoreMutation<{ opted_out: boolean }, null>('support.set_opted_out', {
    affects: ['support'],
  });

export type SupportEvent = 'budget_month' | 'import' | 'tax_report';

/**
 * Avisa o núcleo de um evento de valor (mês fechado, importação terminada, relatório do IR).
 * Se as regras deixam, o cartão espera a próxima visita à Visão geral. Falha em silêncio.
 */
export function supportEvent(event: SupportEvent): void {
  const viewAs = Boolean(useUi.getState().viewAs);
  void call('support.event', { event, ...(viewAs ? { view_as: true } : {}) }).catch(
    () => undefined,
  );
}

/** Dispara o evento uma vez para cada `key` enquanto `active` (ex.: o mês fechado visto). */
export function useSupportEventWhen(active: boolean, event: SupportEvent, key: string): void {
  useEffect(() => {
    if (active) supportEvent(event);
  }, [active, event, key]);
}

export const markSupportSeen = () => call('support.seen', {}).catch(() => undefined);
