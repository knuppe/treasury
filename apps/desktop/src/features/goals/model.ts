/** Metas (funções puras): rótulos, situação, formulário ↔ entrada/patch do núcleo. */
import type { EmergencyReserve } from '@/api/generated/EmergencyReserve';
import type { GoalInput } from '@/api/generated/GoalInput';
import type { GoalKind } from '@/api/generated/GoalKind';
import type { GoalPatch } from '@/api/generated/GoalPatch';
import type { GoalView } from '@/api/generated/GoalView';
import { t } from '@/i18n';

export const GOAL_KINDS: readonly { value: GoalKind; label: string }[] = [
  {
    value: 'emergency',
    get label() {
      return t('metas-tipo-emergencia');
    },
  },
  {
    value: 'travel',
    get label() {
      return t('metas-tipo-viagem');
    },
  },
  {
    value: 'debt_payoff',
    get label() {
      return t('metas-tipo-quitacao');
    },
  },
  {
    value: 'purchase',
    get label() {
      return t('metas-tipo-compra');
    },
  },
  {
    value: 'retirement',
    get label() {
      return t('metas-tipo-aposentadoria');
    },
  },
  {
    value: 'education',
    get label() {
      return t('metas-tipo-educacao');
    },
  },
  {
    value: 'other',
    get label() {
      return t('metas-tipo-outra');
    },
  },
];

export const goalKindLabel = (k: GoalKind) => GOAL_KINDS.find((x) => x.value === k)?.label ?? k;

export type GoalSituation = 'done' | 'on_track' | 'off_track' | 'no_pace' | 'no_date';

/** Situação da meta: atingida, no caminho, fora do ritmo, sem ritmo (sem aportes) ou sem data. */
export function goalSituation(g: GoalView): GoalSituation {
  if (g.remaining === 0) return 'done';
  if (!g.target_date) return g.projected_date ? 'no_date' : 'no_pace';
  if (g.on_track) return 'on_track';
  return g.projected_date ? 'off_track' : 'no_pace';
}

export const SITUATION_LABEL: Record<GoalSituation, { label: string; icon: string; tone: string }> =
  {
    done: {
      get label() {
        return t('metas-situacao-atingida');
      },
      icon: '✓',
      tone: 'text-positive',
    },
    on_track: {
      get label() {
        return t('metas-situacao-no-caminho');
      },
      icon: '●',
      tone: 'text-positive',
    },
    off_track: {
      get label() {
        return t('metas-situacao-fora-ritmo');
      },
      icon: '▲',
      tone: 'text-warning',
    },
    no_pace: {
      get label() {
        return t('metas-situacao-sem-aportes');
      },
      icon: '○',
      tone: 'text-dim',
    },
    no_date: {
      get label() {
        return t('metas-situacao-sem-data');
      },
      icon: '·',
      tone: 'text-dim',
    },
  };

/** Ritmo usado na projeção: a média real das contas ou, sem ela, o aporte planejado. */
export function paceSource(g: GoalView): 'real' | 'planned' | 'none' {
  if (g.avg_monthly_contribution > 0) return 'real';
  if ((g.monthly_contribution ?? 0) > 0) return 'planned';
  return 'none';
}

// ── Formulário ───────────────────────────────────────────────────

export interface GoalForm {
  name: string;
  kind: GoalKind;
  target: number | null;
  targetDate: string | null;
  initial: number | null;
  monthly: number | null;
  accountIds: string[];
  notes: string;
  /** Retorno real ao ano em %, acima da inflação (4 = 4%); vazio: conta linear. */
  realReturn: number | null;
}

export function emptyGoalForm(kind: GoalKind = 'other'): GoalForm {
  return {
    name: kind === 'emergency' ? t('metas-tipo-emergencia') : '',
    kind,
    target: null,
    targetDate: null,
    initial: null,
    monthly: null,
    accountIds: [],
    notes: '',
    realReturn: null,
  };
}

export function formFromGoal(g: GoalView): GoalForm {
  return {
    name: g.name,
    kind: g.kind,
    target: g.target_amount,
    targetDate: g.target_date,
    initial: g.initial_amount || null,
    monthly: g.monthly_contribution,
    accountIds: [...g.account_ids],
    notes: g.notes ?? '',
    realReturn: g.real_return == null ? null : Math.round(g.real_return * 10_000) / 100,
  };
}

export type GoalFormErrors = Partial<
  Record<'name' | 'target' | 'targetDate' | 'realReturn', string>
>;

export function validateGoal(f: GoalForm, today: string): GoalFormErrors {
  const e: GoalFormErrors = {};
  if (!f.name.trim()) e.name = t('metas-erro-nome');
  if (f.target == null || f.target <= 0) e.target = t('metas-erro-valor');
  if (f.targetDate && f.targetDate < today) e.targetDate = t('metas-erro-data-passou');
  if (f.realReturn != null && (f.realReturn < -5 || f.realReturn > 15))
    e.realReturn = t('metas-erro-retorno');
  return e;
}

export function goalInput(f: GoalForm): GoalInput {
  const input: GoalInput = { name: f.name.trim(), kind: f.kind, target_amount: f.target ?? 0 };
  if (f.targetDate) input.target_date = f.targetDate;
  if (f.initial) input.initial_amount = f.initial;
  if (f.monthly) input.monthly_contribution = f.monthly;
  if (f.accountIds.length) input.account_ids = [...f.accountIds].sort();
  if (f.notes.trim()) input.notes = f.notes.trim();
  if (f.realReturn != null) input.real_return = f.realReturn / 100;
  return input;
}

/** Só o que mudou (`null` limpa data, aporte e notas). */
export function goalPatch(original: GoalView, f: GoalForm): GoalPatch {
  const p: GoalPatch = { id: original.id };
  const before = formFromGoal(original);
  if (f.name.trim() !== before.name) p.name = f.name.trim();
  if (f.kind !== before.kind) p.kind = f.kind;
  if (f.target != null && f.target !== before.target) p.target_amount = f.target;
  if (f.targetDate !== before.targetDate) p.target_date = f.targetDate;
  if ((f.initial ?? 0) !== original.initial_amount) p.initial_amount = f.initial ?? 0;
  if ((f.monthly ?? null) !== (before.monthly ?? null)) p.monthly_contribution = f.monthly ?? null;
  const a = [...f.accountIds].sort().join(',');
  if (a !== [...before.accountIds].sort().join(',')) p.account_ids = [...f.accountIds].sort();
  if (f.notes.trim() !== before.notes.trim()) p.notes = f.notes.trim() || null;
  if (f.realReturn !== before.realReturn)
    p.real_return = f.realReturn == null ? null : f.realReturn / 100;
  return p;
}

export const hasChanges = (p: GoalPatch) => Object.keys(p).length > 1;

// ── Reserva de emergência ────────────────────────────────────────

export interface ReserveView {
  coverage: number | null;
  /** Fração da meta de reserva já guardada (0–1+). */
  progress: number;
  missing: number;
  status: 'empty' | 'low' | 'partial' | 'ok';
}

export function reserveView(r: EmergencyReserve): ReserveView {
  const progress = r.target_amount > 0 ? r.current_amount / r.target_amount : 0;
  const coverage = r.coverage_months;
  return {
    coverage,
    progress,
    missing: Math.max(0, r.target_amount - r.current_amount),
    status:
      r.avg_essential_monthly === 0
        ? 'empty'
        : progress >= 1
          ? 'ok'
          : coverage != null && coverage < 3
            ? 'low'
            : 'partial',
  };
}
