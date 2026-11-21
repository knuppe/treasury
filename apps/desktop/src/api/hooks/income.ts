/** Hooks de `income.*`: rendas dos membros, holerites e a previsão dos próximos meses. */
import type { IncomeSource } from '../generated/IncomeSource';
import type { IncomeSourceInput } from '../generated/IncomeSourceInput';
import type { IncomeSourcePatch } from '../generated/IncomeSourcePatch';
import type { IncomeSummary } from '../generated/IncomeSummary';
import type { Payslip } from '../generated/Payslip';
import type { PayslipArgs } from '../generated/PayslipArgs';
import type { PayslipPreview } from '../generated/PayslipPreview';
import type { PayslipResult } from '../generated/PayslipResult';
import { call } from '../client';
import { useCore, useCoreMutation } from './coreQuery';

export const useIncomeSummary = (months = 12) =>
  useCore<IncomeSummary>('income', 'summary', { months });

export const usePayslips = (memberId?: string) =>
  useCore<Payslip[]>('income', 'list_payslips', memberId ? { member_id: memberId } : {});

export const useCreateIncomeSource = () =>
  useCoreMutation<IncomeSourceInput, IncomeSource>('income.create_source', {
    affects: ['income'],
  });

export const useUpdateIncomeSource = () =>
  useCoreMutation<IncomeSourcePatch, IncomeSource>('income.update_source', {
    affects: ['income'],
  });

export const useDeleteIncomeSource = () =>
  useCoreMutation<{ id: string }, unknown>('income.delete_source', { affects: ['income'] });

export const previewPayslip = (args: PayslipArgs) =>
  call<PayslipPreview>('income.payslip_preview', args);

export const useImportPayslip = () =>
  useCoreMutation<PayslipArgs, PayslipResult>('income.payslip_import', {
    affects: ['income', 'tax_burden'],
  });

export const useDeletePayslip = () =>
  useCoreMutation<{ id: string }, unknown>('income.delete_payslip', {
    affects: ['income', 'tax_burden'],
  });
