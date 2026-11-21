/** Hooks de `pension.*`: plano de previdência (VGBL/PGBL), aportes e simulação de resgate. */
import type { PensionPlanInput } from '../generated/PensionPlanInput';
import type { PensionStatementArgs } from '../generated/PensionStatementArgs';
import type { PensionStatementPreview } from '../generated/PensionStatementPreview';
import type { PensionStatementResult } from '../generated/PensionStatementResult';
import type { PensionView } from '../generated/PensionView';
import type { WithdrawalArgs } from '../generated/WithdrawalArgs';
import type { WithdrawalSim } from '../generated/WithdrawalSim';
import { call } from '../client';
import { useCore, useCoreMutation } from './coreQuery';

export const usePension = (assetId: string, enabled = true) =>
  useCore<PensionView>('pension', 'get', { asset_id: assetId }, { enabled });

export const useSetPensionPlan = () =>
  useCoreMutation<PensionPlanInput, PensionView>('pension.set_plan', { affects: ['market'] });

/** Saldo informado (planos sem cota pública): vira a cota do dia. */
export const useSetPensionBalance = () =>
  useCoreMutation<{ asset_id: string; date: string; balance: number }, { price: string }>(
    'pension.set_balance',
    { affects: ['market', 'ledger'] },
  );

export const simulateWithdrawal = (args: WithdrawalArgs) =>
  call<WithdrawalSim>('pension.simulate', args);

/** Extrato de previdência (docs/brazil/pension-statement.md): prévia sem gravar. */
export const previewPensionStatement = (args: PensionStatementArgs) =>
  call<PensionStatementPreview>('pension.statement_preview', args);

/** Grava o extrato (plano, cotas, cotações e, se pedido, os seguros de risco). */
export const useImportPensionStatement = () =>
  useCoreMutation<PensionStatementArgs, PensionStatementResult>('pension.statement_import', {
    affects: ['market', 'ledger'],
  });
