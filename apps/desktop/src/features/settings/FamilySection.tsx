import { useState } from 'react';
import { Link } from 'react-router';

import type { CostMethod } from '@/api/generated/CostMethod';
import type { CountryChange } from '@/api/generated/CountryChange';
import type { Institution } from '@/api/generated/Institution';
import type { InstitutionKind } from '@/api/generated/InstitutionKind';
import {
  useArchiveMember,
  useCreateInstitution,
  useCreateMember,
  useDeleteInstitution,
  useHousehold,
  useInstitutions,
  useMembers,
  useUpdateHousehold,
  useUpdateInstitution,
  useUpdateMember,
} from '@/api/hooks/household';
import { useCountry, useSetCountry } from '@/api/hooks/country';
import { useCostMethod, useSetCostMethod } from '@/api/hooks/portfolio';
import {
  Button,
  Dialog,
  Field,
  Icon,
  InstrumentCard,
  Input,
  Select,
  toast,
} from '@/design/components';
import { CountryPicker, supportText } from '@/features/country/CountryPicker';
import { Checkbox, InlineError, QueryState } from '@/features/transactions/ui/controls';
import { errorText } from '@/features/transactions/ui/errors';
import { INSTITUTION_KIND } from '@/features/transactions/ui/labels';
import { t } from '@/i18n';

import { tRich } from './richText';

const KINDS: InstitutionKind[] = ['bank', 'broker', 'issuer', 'other'];

function InlineName({
  value,
  label,
  onSave,
}: {
  value: string;
  label: string;
  onSave: (v: string) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [text, setText] = useState(value);
  if (!editing) {
    return (
      <button
        type="button"
        className="min-w-0 truncate text-left text-[13.5px] text-fg hover:text-accent"
        onClick={() => (setText(value), setEditing(true))}
        aria-label={t('ajustes-familia-renomear', { nome: label })}
      >
        {value}
      </button>
    );
  }
  const commit = () => {
    setEditing(false);
    if (text.trim() && text.trim() !== value) onSave(text.trim());
  };
  return (
    <Input
      aria-label={t('ajustes-familia-novo-nome', { nome: label })}
      value={text}
      autoFocus
      onChange={(e) => setText(e.target.value)}
      onBlur={commit}
      onKeyDown={(e) => {
        if (e.key === 'Enter') commit();
        if (e.key === 'Escape') {
          e.stopPropagation();
          setEditing(false);
        }
      }}
    />
  );
}

function HouseholdCard() {
  const { data } = useHousehold();
  const update = useUpdateHousehold();
  return (
    <InstrumentCard index="12.17" title={t('ajustes-secao-familia')}>
      {data ? (
        <div className="flex flex-col gap-2">
          <Field label={t('ajustes-familia-nome')} hint={t('ajustes-familia-nome-dica')}>
            <InlineName
              value={data.name}
              label={t('ajustes-familia-a-familia')}
              onSave={(name) =>
                update.mutate(
                  { name },
                  {
                    onSuccess: () =>
                      toast({ tone: 'success', title: t('ajustes-familia-nome-alterado') }),
                  },
                )
              }
            />
          </Field>
          <p className="label-caps">{t('ajustes-familia-moeda', { moeda: data.base_currency })}</p>
          <InlineError error={update.error} />
        </div>
      ) : null}
    </InstrumentCard>
  );
}

function changeText(c: CountryChange): string {
  const parts = [];
  if (c.disabled.length)
    parts.push(t('ajustes-familia-fontes-desligadas', { n: c.disabled.length }));
  if (c.enabled.length) parts.push(t('ajustes-familia-fontes-religadas', { n: c.enabled.length }));
  if (c.currency_changed)
    parts.push(t('ajustes-familia-moeda-agora', { moeda: c.state.base_currency }));
  return parts.length ? `${parts.join(', ')}.` : t('ajustes-familia-fontes-certas');
}

/** País onde a família mora (docs/countries.md): o app se adapta a ele. */
function CountryCard() {
  const { data: st, isLoading, error } = useCountry();
  const set = useSetCountry();
  const [pick, setPick] = useState<string | null>(null);
  const [currency, setCurrency] = useState(false);
  const target = st && pick ? st.choices.find((c) => c.code === pick) : undefined;
  return (
    <InstrumentCard index="12.20" title={t('ajustes-pais-titulo')}>
      <QueryState loading={isLoading} error={error}>
        {st ? (
          <div className="flex flex-col gap-3">
            <Field label={t('ajustes-pais-onde-mora')} hint={supportText(st.info.support)}>
              <CountryPicker
                choices={st.choices}
                value={st.info.code}
                disabled={set.isPending}
                onChange={(c) => {
                  if (c === st.info.code) return;
                  setPick(c);
                  setCurrency(!st.has_data);
                }}
              />
            </Field>
            <p className="label-caps">
              {t('ajustes-pais-resumo', {
                moeda: st.base_currency,
                inflacao: st.info.inflation_label,
                juro: st.info.rate_label,
              })}
            </p>
            {st.info.limits.length ? (
              <ul className="flex list-disc flex-col gap-1 pl-4 text-[12.5px] text-dim">
                {st.info.limits.map((l) => (
                  <li key={l}>{l}</li>
                ))}
              </ul>
            ) : null}
            <InlineError error={set.error} />
          </div>
        ) : null}
      </QueryState>
      {st && target ? (
        <Dialog
          open
          onOpenChange={(o) => !o && setPick(null)}
          title={t('ajustes-pais-mudar-titulo', { pais: target.name })}
          description={supportText(target.support)}
          size="md"
        >
          <div className="flex flex-col gap-3 text-[13px]">
            <ul className="flex list-disc flex-col gap-1.5 pl-4 text-fg">
              <li>{t('ajustes-pais-mudar-fontes', { pais: target.name })}</li>
              <li>{t('ajustes-pais-mudar-referencia', { pais: target.name })}</li>
              <li>{t('ajustes-pais-mudar-mantem')}</li>
            </ul>
            {target.currency !== st.base_currency ? (
              <div className="flex flex-col gap-1">
                <Checkbox
                  checked={currency}
                  onChange={setCurrency}
                  label={t('ajustes-pais-trocar-moeda', {
                    de: st.base_currency,
                    para: target.currency,
                  })}
                />
                {st.has_data ? (
                  <p className="pl-6 text-[12px] text-dim">
                    {t('ajustes-pais-trocar-moeda-dica', { moeda: target.currency })}
                  </p>
                ) : null}
              </div>
            ) : null}
            <div className="flex justify-end gap-2 border-t border-line pt-3">
              <Button variant="ghost" onClick={() => setPick(null)}>
                {t('comum-cancelar')}
              </Button>
              <Button
                variant="primary"
                loading={set.isPending}
                onClick={() =>
                  set.mutate(
                    { country: target.code, change_currency: currency },
                    {
                      onSuccess: (c) => {
                        setPick(null);
                        toast({
                          tone: 'success',
                          title: t('ajustes-pais-alterado', { pais: c.state.info.name }),
                          description: changeText(c),
                        });
                      },
                    },
                  )
                }
              >
                {t('ajustes-pais-mudar', { pais: target.name })}
              </Button>
            </div>
          </div>
        </Dialog>
      ) : null}
    </InstrumentCard>
  );
}

const METHOD_LABEL: Record<CostMethod, () => string> = {
  average: () => t('ajustes-metodo-custo-average'),
  fifo: () => t('ajustes-metodo-custo-fifo'),
  uk_pool: () => t('ajustes-metodo-custo-uk_pool'),
};

/** Método de custo das vendas (docs/cost-method.md). */
function CostMethodCard() {
  const { data: st, isLoading, error } = useCostMethod();
  const set = useSetCostMethod();
  const AUTO = 'auto';
  return (
    <InstrumentCard index="12.21" title={t('ajustes-metodo-custo-titulo')}>
      <QueryState loading={isLoading} error={error}>
        {st ? (
          <div className="flex flex-col gap-3">
            <p className="text-[12.5px] text-dim">{t('ajustes-metodo-custo-dica')}</p>
            <Field label={t('ajustes-metodo-custo-campo')}>
              <Select
                value={st.automatic ? AUTO : st.method}
                disabled={set.isPending}
                onValueChange={(v) =>
                  set.mutate(
                    { method: v === AUTO ? undefined : (v as CostMethod) },
                    {
                      onSuccess: () =>
                        toast({ tone: 'success', title: t('ajustes-metodo-custo-salvo') }),
                    },
                  )
                }
                options={[
                  {
                    value: AUTO,
                    label: t('ajustes-metodo-custo-auto', {
                      metodo: METHOD_LABEL[st.country_method](),
                    }),
                  },
                  ...(['average', 'fifo', 'uk_pool'] as const).map((m) => ({
                    value: m,
                    label: METHOD_LABEL[m](),
                  })),
                ]}
              />
            </Field>
            <p className="text-[12px] text-dim">{t('ajustes-metodo-custo-renda-fixa')}</p>
            <InlineError error={set.error} />
          </div>
        ) : null}
      </QueryState>
    </InstrumentCard>
  );
}

function MembersCard() {
  const [showArchived, setShowArchived] = useState(false);
  const { data = [], isLoading, error } = useMembers(true);
  const create = useCreateMember();
  const update = useUpdateMember();
  const archive = useArchiveMember();
  const [name, setName] = useState('');
  const fail = (title: string) => (e: unknown) =>
    toast({ tone: 'error', title, description: errorText(e) });
  const visible = data.filter((m) => showArchived || !m.archived);
  return (
    <InstrumentCard
      index="12.18"
      title={t('ajustes-membros-titulo')}
      actions={
        <button
          type="button"
          className="label-caps hover:text-fg"
          onClick={() => setShowArchived(!showArchived)}
          aria-pressed={showArchived}
        >
          {showArchived ? t('ajustes-membros-esconder') : t('ajustes-membros-mostrar')}
        </button>
      }
    >
      <QueryState loading={isLoading} error={error}>
        <ul
          className="flex flex-col divide-y divide-line/60"
          aria-label={t('ajustes-membros-lista')}
        >
          {visible.map((m) => (
            <li key={m.id} className="flex items-center gap-3 py-1.5">
              <span className="min-w-0 flex-1">
                <InlineName
                  value={m.name}
                  label={m.name}
                  onSave={(n) =>
                    update.mutate(
                      { id: m.id, name: n },
                      { onError: fail(t('ajustes-membros-nao-renomeou')) },
                    )
                  }
                />
              </span>
              {m.is_primary ? (
                <span className="label-caps rounded-control border border-accent/40 px-1.5 text-accent">
                  {t('ajustes-membros-principal')}
                </span>
              ) : null}
              {m.archived ? (
                <span className="label-caps">{t('ajustes-membros-arquivado')}</span>
              ) : null}
              {!m.is_primary && !m.archived ? (
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() =>
                    update.mutate(
                      { id: m.id, is_primary: true },
                      { onError: fail(t('ajustes-nao-alterou')) },
                    )
                  }
                >
                  {t('ajustes-membros-tornar-principal')}
                </Button>
              ) : null}
              {!m.is_primary ? (
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() =>
                    archive.mutate(
                      { id: m.id, archived: !m.archived },
                      { onError: fail(t('ajustes-membros-nao-arquivou')) },
                    )
                  }
                >
                  {m.archived ? t('ajustes-reativar') : t('ajustes-arquivar')}
                </Button>
              ) : null}
            </li>
          ))}
        </ul>
        <form
          className="mt-3 flex gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            if (!name.trim()) return;
            create.mutate(
              { name: name.trim() },
              {
                onSuccess: () => (
                  toast({
                    tone: 'success',
                    title: t('ajustes-membros-adicionado'),
                    description: name.trim(),
                  }),
                  setName('')
                ),
              },
            );
          }}
        >
          <Input
            aria-label={t('ajustes-membros-novo')}
            placeholder={t('ajustes-membros-novo-placeholder')}
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
          <Button
            type="submit"
            variant="secondary"
            loading={create.isPending}
            disabled={!name.trim()}
            icon={<Icon name="plus" size={12} />}
          >
            {t('ajustes-adicionar')}
          </Button>
        </form>
        <InlineError error={create.error} />
        <p className="mt-2 text-[12.5px] text-dim">
          {tRich('ajustes-membros-familia-tela', {
            link: (
              <Link to="/familia" className="text-accent hover:underline">
                {t('casca-tela-familia')}
              </Link>
            ),
          })}
        </p>
      </QueryState>
    </InstrumentCard>
  );
}

function InstitutionRow({ inst }: { inst: Institution }) {
  const update = useUpdateInstitution();
  const remove = useDeleteInstitution();
  return (
    <li className="grid grid-cols-[minmax(0,1fr)_150px_90px_auto] items-center gap-3 py-1.5">
      <InlineName
        value={inst.name}
        label={inst.name}
        onSave={(name) => update.mutate({ id: inst.id, name })}
      />
      <Select
        aria-label={t('ajustes-inst-tipo-de', { nome: inst.name })}
        value={inst.kind}
        onValueChange={(v) => update.mutate({ id: inst.id, kind: v as InstitutionKind })}
        options={KINDS.map((k) => ({ value: k, label: INSTITUTION_KIND[k] }))}
      />
      <span className="num text-[12px] text-dim">{inst.code ?? '—'}</span>
      <Button
        size="sm"
        variant="ghost"
        onClick={() =>
          remove.mutate(
            { id: inst.id },
            {
              onSuccess: () =>
                toast({
                  tone: 'success',
                  title: t('ajustes-inst-apagada'),
                  description: inst.name,
                }),
              onError: (e) =>
                toast({
                  tone: 'error',
                  title: t('ajustes-inst-nao-apagou'),
                  description: errorText(e),
                }),
            },
          )
        }
      >
        {t('comum-apagar')}
      </Button>
    </li>
  );
}

function InstitutionsCard() {
  const { data = [], isLoading, error } = useInstitutions();
  const create = useCreateInstitution();
  const [name, setName] = useState('');
  const [kind, setKind] = useState<InstitutionKind>('bank');
  const [code, setCode] = useState('');
  return (
    <InstrumentCard index="12.19" title={t('ajustes-inst-titulo')}>
      <QueryState loading={isLoading} error={error}>
        <ul className="flex flex-col divide-y divide-line/60" aria-label={t('ajustes-inst-titulo')}>
          {data.map((i) => (
            <InstitutionRow key={i.id} inst={i} />
          ))}
          {data.length === 0 ? (
            <li className="py-2 text-[13px] text-dim">{t('ajustes-inst-nenhuma')}</li>
          ) : null}
        </ul>
        <form
          className="mt-3 grid grid-cols-[minmax(0,1fr)_150px_90px_auto] gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            if (!name.trim()) return;
            create.mutate(
              { name: name.trim(), kind, ...(code.trim() ? { code: code.trim() } : {}) },
              {
                onSuccess: () => (
                  toast({
                    tone: 'success',
                    title: t('ajustes-inst-criada'),
                    description: name.trim(),
                  }),
                  setName(''),
                  setCode('')
                ),
              },
            );
          }}
        >
          <Input
            aria-label={t('ajustes-inst-nome')}
            placeholder={t('ajustes-inst-nome-placeholder')}
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
          <Select
            aria-label={t('ajustes-inst-tipo-nova')}
            value={kind}
            onValueChange={(v) => setKind(v as InstitutionKind)}
            options={KINDS.map((k) => ({ value: k, label: INSTITUTION_KIND[k] }))}
          />
          <Input
            aria-label={t('ajustes-inst-codigo')}
            placeholder={t('ajustes-inst-codigo-placeholder')}
            numeric
            value={code}
            onChange={(e) => setCode(e.target.value)}
          />
          <Button
            type="submit"
            variant="secondary"
            loading={create.isPending}
            disabled={!name.trim()}
          >
            {t('ajustes-inst-criar')}
          </Button>
        </form>
        <InlineError error={create.error} />
      </QueryState>
    </InstrumentCard>
  );
}

/** 12.17 · Família: nome, país, membros e instituições financeiras. */
export function FamilySection() {
  return (
    <div className="grid gap-5 @min-[1040px]:grid-cols-2">
      <div className="flex flex-col gap-5">
        <HouseholdCard />
        <CountryCard />
        <CostMethodCard />
        <MembersCard />
      </div>
      <InstitutionsCard />
    </div>
  );
}
