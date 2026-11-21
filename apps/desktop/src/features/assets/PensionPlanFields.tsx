/**
 * Campos do plano de previdência (VGBL/PGBL): tipo, regime de tributação, seguradora, início
 * e taxas. Usados no cadastro do ativo (o plano grava junto, `assets.create` com `pension`) e
 * na aba Previdência (`pension.set_plan`).
 */
import type { PensionPlan } from '@/api/generated/PensionPlan';
import type { PensionPlanTerms } from '@/api/generated/PensionPlanTerms';
import type { PlanType } from '@/api/generated/PlanType';
import type { TaxRegime } from '@/api/generated/TaxRegime';
import { DateInput, Field, Input } from '@/design/components';
import { Segmented } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

export interface PlanDraft {
  type: PlanType;
  regime: TaxRegime;
  insurer: string;
  admin: string;
  loading: string;
  start: string | null;
}

/** Rascunho a partir do plano salvo (ou o padrão: VGBL regressivo). */
export function planDraft(p?: PensionPlan | null): PlanDraft {
  return {
    type: p?.plan_type ?? 'vgbl',
    regime: p?.tax_regime ?? 'regressive',
    insurer: p?.insurer ?? '',
    admin: p?.admin_fee_pct?.replace('.', ',') ?? '',
    loading: p?.loading_fee_pct?.replace('.', ',') ?? '',
    start: p?.start_date ?? null,
  };
}

/** Rascunho → termos do núcleo (campos vazios ficam de fora). */
export function planTerms(d: PlanDraft): PensionPlanTerms {
  return {
    plan_type: d.type,
    tax_regime: d.regime,
    ...(d.insurer.trim() ? { insurer: d.insurer.trim() } : {}),
    ...(d.admin.trim() ? { admin_fee_pct: d.admin.trim() } : {}),
    ...(d.loading.trim() ? { loading_fee_pct: d.loading.trim() } : {}),
    ...(d.start ? { start_date: d.start } : {}),
  };
}

export function PensionPlanFields({
  value,
  onChange,
}: {
  value: PlanDraft;
  onChange: (d: PlanDraft) => void;
}) {
  const set = <K extends keyof PlanDraft>(k: K, v: PlanDraft[K]) => onChange({ ...value, [k]: v });
  return (
    <div className="grid gap-4 @min-[640px]:grid-cols-2">
      <Field label={t('ativos-plano-tipo')} hint={t('ativos-plano-tipo-dica')}>
        <Segmented<PlanType>
          label={t('ativos-plano-tipo')}
          value={value.type}
          onChange={(v) => set('type', v)}
          options={[
            { value: 'vgbl', label: 'VGBL' },
            { value: 'pgbl', label: 'PGBL' },
            { value: 'efpc', label: t('ativos-plano-efpc') },
          ]}
        />
      </Field>
      <Field label={t('ativos-plano-regime')} hint={t('ativos-plano-regime-dica')}>
        <Segmented<TaxRegime>
          label={t('ativos-plano-regime')}
          value={value.regime}
          onChange={(v) => set('regime', v)}
          options={[
            { value: 'regressive', label: t('ativos-plano-regressivo') },
            { value: 'progressive', label: t('ativos-plano-progressivo') },
          ]}
        />
      </Field>
      <Field label={t('ativos-plano-seguradora')}>
        <Input
          value={value.insurer}
          onChange={(e) => set('insurer', e.target.value)}
          placeholder="Brasilprev, Icatu…"
        />
      </Field>
      <Field label={t('ativos-plano-inicio')}>
        <DateInput value={value.start} onValueChange={(v) => set('start', v)} />
      </Field>
      <Field label={t('ativos-plano-taxa-adm')}>
        <Input
          numeric
          inputMode="decimal"
          value={value.admin}
          onChange={(e) => set('admin', e.target.value)}
          suffix="%"
        />
      </Field>
      <Field label={t('ativos-plano-taxa-carregamento')} hint={t('ativos-plano-carregamento-dica')}>
        <Input
          numeric
          inputMode="decimal"
          value={value.loading}
          onChange={(e) => set('loading', e.target.value)}
          suffix="%"
        />
      </Field>
    </div>
  );
}
