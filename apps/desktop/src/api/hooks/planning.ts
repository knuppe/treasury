/**
 * Hooks do planejamento: previsão de saldo e "quanto posso gastar hoje" (`forecast.*`),
 * simulador de quitação (`debt.payoff_plan`), assinaturas detectadas (`subscriptions.*`) e
 * preferências de notificação (`notify.*`).
 */
import { keepPreviousData, useQuery } from '@tanstack/react-query';

import type { BalanceForecast } from '../generated/BalanceForecast';
import type { DailyAllowance } from '../generated/DailyAllowance';
import type { DetectedInstallments } from '../generated/DetectedInstallments';
import type { ForecastArgs } from '../generated/ForecastArgs';
import type { NotifyPrefs } from '../generated/NotifyPrefs';
import type { PayoffArgs } from '../generated/PayoffArgs';
import type { PayoffPlan } from '../generated/PayoffPlan';
import type { SavedScenario } from '../generated/SavedScenario';
import type { ScenarioArgs } from '../generated/ScenarioArgs';
import type { ScenarioResult } from '../generated/ScenarioResult';
import type { SubscriptionReport } from '../generated/SubscriptionReport';
import { call } from '../client';
import { useCore, useCoreMutation } from './coreQuery';

export const useBalanceForecast = (args: ForecastArgs, enabled = true) =>
  useQuery({
    queryKey: ['forecast', 'balance', args],
    queryFn: () => call<BalanceForecast>('forecast.balance', args),
    enabled,
    placeholderData: keepPreviousData,
  });

export const useDailyAllowance = (enabled = true) =>
  useCore<DailyAllowance>('forecast', 'daily_allowance', {}, { enabled });

/** Simulação (só lê): refeita a cada mudança do extra ou das dívidas informadas. */
export const useDebtPayoff = (args: PayoffArgs, enabled = true) =>
  useQuery({
    queryKey: ['debt', 'payoff_plan', args],
    queryFn: () => call<PayoffPlan>('debt.payoff_plan', args),
    enabled,
    placeholderData: keepPreviousData,
    retry: false,
  });

export const useSubscriptions = () => useCore<SubscriptionReport>('subscriptions', 'list');

/** "Não é assinatura" (ou desfazer): preferência do usuário, fora do histórico. */
export const useDismissSubscription = () =>
  useCoreMutation<{ key: string }, SubscriptionReport>('subscriptions.dismiss', {
    affects: ['subscriptions'],
  });

export const useRestoreSubscription = () =>
  useCoreMutation<{ key: string }, SubscriptionReport>('subscriptions.restore', {
    affects: ['subscriptions'],
  });

export const useNotifyPrefs = () => useCore<NotifyPrefs>('notify', 'get_prefs');

export const useSetNotifyPrefs = () =>
  useCoreMutation<NotifyPrefs, NotifyPrefs>('notify.set_prefs', { affects: ['notify'] });

/** Confere os lembretes agora (cria os alertas que faltam). */
export const runRemindersNow = () => call<{ created: number }>('notify.run_now', {});

/** Cenário "e se" (só lê): refeito a cada mudança dos eventos ou do horizonte. */
export const useScenario = (args: ScenarioArgs, enabled = true) =>
  useQuery({
    queryKey: ['scenarios', 'simulate', args],
    queryFn: () => call<ScenarioResult>('scenarios.simulate', args),
    enabled,
    placeholderData: keepPreviousData,
    retry: false,
  });

export const useSavedScenarios = () => useCore<SavedScenario[]>('scenarios', 'list');

export const useSaveScenario = () =>
  useCoreMutation<SavedScenario, SavedScenario[]>('scenarios.save', { affects: ['scenarios'] });

export const useDeleteScenario = () =>
  useCoreMutation<{ name: string }, SavedScenario[]>('scenarios.delete', {
    affects: ['scenarios'],
  });

/** Parcelas detectadas pela descrição (estimativa). */
export const useDetectedInstallments = () =>
  useCore<DetectedInstallments>('installments', 'detected');

export const useIgnoreInstallmentSeries = () =>
  useCoreMutation<{ key: string }, DetectedInstallments>('installments.ignore', {
    affects: ['installments'],
  });
