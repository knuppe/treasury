/**
 * Mock do Estudo com IA (`assets.study_*`): aceite do aviso, contexto por ativo, pacote de
 * exemplo e um estudo pronto na hora (o job termina no primeiro poll), com um número que
 * confere e outro que não existe nos dados. Espelho simplificado de `assets/study.rs`.
 */
import type { Study } from '../generated/Study';
import type { StudyPackage } from '../generated/StudyPackage';
import type { StudyState } from '../generated/StudyState';
import { mockFail, onMockReset, registerMockHandlers, requireUnlocked } from '../mockRegistry';
import { mockFinishedJob } from './ai';
import { getAsset } from './assets';

interface State {
  accepted: boolean;
  context: Map<string, { text: string; at: string }>;
  studies: Study[];
  seq: number;
}

let st: State = fresh();
function fresh(): State {
  return { accepted: false, context: new Map(), studies: [], seq: 0 };
}
onMockReset(() => {
  st = fresh();
});

const str = (v: unknown) => (typeof v === 'string' ? v : '');

const ENGINE = {
  id: 'eng-claude',
  name: 'Claude',
  kind: 'api',
  provider: 'anthropic',
  provider_label: 'Anthropic',
  model: 'claude-sonnet',
  local: false,
  lean: false,
} as const;

function pkg(assetId: string, a: Record<string, unknown>) {
  const asset = getAsset(assetId);
  const p: Record<string, unknown> = {
    record: {
      ticker: asset.ticker,
      name: asset.name,
      class: asset.class_label,
      sector: asset.sector,
    },
    score: { score: 71, band_label: 'Saudável', confidence: 0.8 },
    price: { last_close: 36.1, change_1y: 0.084 },
  };
  const ctx = st.context.get(assetId);
  if (ctx) p.user_context = ctx.text;
  if (str(a.question)) p.question = str(a.question);
  if (a.include_position === true)
    p.my_position = { portfolio_weight: 0.052, unrealized_gain: 0.13 };
  return p;
}

function summaryOf(s: Study) {
  const {
    include_position,
    include_values,
    context_snapshot,
    output,
    checks,
    disclaimer_version,
    ...head
  } = s;
  void include_position;
  void include_values;
  void context_snapshot;
  void output;
  void checks;
  void disclaimer_version;
  return head;
}

registerMockHandlers({
  'assets.study_state': (a): StudyState => {
    requireUnlocked();
    const id = str(a.asset_id);
    getAsset(id);
    const ctx = st.context.get(id);
    return {
      asset_id: id,
      context: ctx?.text ?? null,
      context_updated_at: ctx?.at ?? null,
      accepted: st.accepted,
      disclaimer_version: '1',
      engine: ENGINE,
      studies: st.studies.filter((s) => s.asset_id === id).map(summaryOf),
    };
  },
  'assets.study_accept': () => {
    requireUnlocked();
    st.accepted = true;
    return null;
  },
  'assets.study_context_set': (a) => {
    requireUnlocked();
    const id = str(a.asset_id);
    const text = str(a.text).trim();
    if (text.length > 2000)
      mockFail('invalid', 'o contexto para a IA pode ter no máximo 2000 caracteres');
    if (text) st.context.set(id, { text, at: new Date().toISOString() });
    else st.context.delete(id);
    return null;
  },
  'assets.study_package': (a): StudyPackage => {
    requireUnlocked();
    const p = pkg(str(a.asset_id), a);
    return {
      asset_id: str(a.asset_id),
      package: p,
      chars: JSON.stringify(p).length,
      engine: ENGINE,
    };
  },
  'assets.study_start': (a) => {
    requireUnlocked();
    const id = str(a.asset_id);
    const asset = getAsset(id);
    if (!st.accepted)
      mockFail(
        'invalid',
        'Antes do primeiro estudo, leia e aceite o aviso na aba Estudo (IA) do ativo.',
      );
    const job = mockFinishedJob('Estudo com IA', 'study', () => {
      st.seq += 1;
      const study: Study = {
        id: `study-${st.seq}`,
        asset_id: id,
        created_at: new Date().toISOString(),
        engine_name: ENGINE.name,
        provider: ENGINE.provider_label,
        model: ENGINE.model,
        question: str(a.question) || null,
        confidence: 'medium',
        unchecked: 1,
        summary: `${asset.ticker ?? asset.name}: receita regulada e dividendos regulares, com dívida a acompanhar.`,
        include_position: a.include_position === true,
        include_values: a.include_values === true,
        context_snapshot: st.context.get(id)?.text ?? null,
        disclaimer_version: '1',
        output: {
          summary: `${asset.ticker ?? asset.name}: receita regulada e dividendos regulares, com dívida a acompanhar.`,
          business: null,
          strengths: [{ text: 'Score de saúde na faixa Saudável', data: 'score.score' }],
          risks: [{ text: 'Juros altos encarecem a dívida', data: null }],
          key_numbers: [
            { label: 'Score', value: 71, unit: null, ref: 'score.score' },
            {
              label: 'Dívida líquida / EBITDA',
              value: 3.4,
              unit: 'x',
              ref: 'fundamentals.net_debt_ebitda',
            },
          ],
          changes: null,
          context_check: null,
          manual_checks: [{ item: 'Dívida líquida', where: 'DFP 2025 na CVM' }],
          sources: [{ name: 'CVM', date: null }],
          confidence: 'medium',
          missing: null,
        },
        checks: [
          { label: 'Score', status: 'ok', expected: 71 },
          { label: 'Dívida líquida / EBITDA', status: 'not_found', expected: null },
        ],
      };
      st.studies.unshift(study);
      return { screen: 'asset_study', asset_id: id, id: study.id };
    });
    return { job_id: job };
  },
  'assets.study_list': (a) => {
    requireUnlocked();
    return st.studies.filter((s) => s.asset_id === str(a.asset_id)).map(summaryOf);
  },
  'assets.study_get': (a): Study => {
    requireUnlocked();
    const s = st.studies.find((x) => x.id === str(a.id));
    if (!s) mockFail('not_found', `estudo ${str(a.id)}`);
    return s;
  },
  'assets.study_delete': (a) => {
    requireUnlocked();
    st.studies = st.studies.filter((x) => x.id !== str(a.id));
    return null;
  },
});
