/**
 * Montador de condições do screener: `campo op valor`, combinadas com E/OU no nível de
 * cima e em grupos (`(A e B) ou (C e D)`). Edita o rascunho (`DraftQuery`); a conversão
 * para o núcleo fica em `query.ts`.
 */
import type { CmpOp } from '@/api/generated/CmpOp';
import type { IndicatorInfo } from '@/api/generated/IndicatorInfo';
import type { Logic } from '@/api/generated/Logic';
import { Button, Icon, IconButton, Input, Select, type SelectOption } from '@/design/components';
import { t } from '@/i18n';
import { currencyPrefix } from '@/lib/format';

import { Segmented } from '../portfolio/components/Segmented';
import { CLASS_LABEL } from '../portfolio/labels';
import {
  BOOL_FIELDS,
  FUNDAMENTAL_CLASSES,
  GROUP_LABEL,
  LOGIC_LABEL,
  OP_LABEL,
  TEXT_FIELDS,
} from './labels';
import {
  arity,
  changeField,
  fieldKind,
  newCondition,
  newGroup,
  opsFor,
  type DraftCondition,
  type DraftGroup,
  type DraftQuery,
} from './query';

export function fieldOptions(catalog: readonly IndicatorInfo[]): SelectOption[] {
  return [
    ...catalog.map((i) => ({ value: i.key, label: i.label, hint: GROUP_LABEL[i.group] })),
    ...TEXT_FIELDS.map((f) => ({ value: f.key, label: f.label, hint: t('pesquisa-tipo-texto') })),
    ...BOOL_FIELDS.map((f) => ({ value: f.key, label: f.label, hint: t('pesquisa-tipo-sim-nao') })),
  ];
}

function unitSuffix(field: string, catalog: readonly IndicatorInfo[]): string | undefined {
  const u = catalog.find((i) => i.key === field)?.unit;
  if (u === 'percent') return '%';
  if (u === 'multiple') return '×';
  // Os fundamentos do filtro são os da CVM, em reais.
  if (u === 'money') return currencyPrefix('BRL');
  return undefined;
}

function LogicSwitch({
  value,
  onChange,
  label,
}: {
  value: Logic;
  onChange: (l: Logic) => void;
  label: string;
}) {
  return (
    <Segmented<Logic>
      size="sm"
      label={label}
      value={value}
      onChange={onChange}
      options={[
        {
          value: 'and',
          label: t('pesquisa-logica-todas'),
          hint: t('pesquisa-logica-todas-dica'),
        },
        {
          value: 'or',
          label: t('pesquisa-logica-qualquer'),
          hint: t('pesquisa-logica-qualquer-dica'),
        },
      ]}
    />
  );
}

export function ConditionRow({
  c,
  index,
  catalog,
  onChange,
  onRemove,
  error,
  joiner,
  group,
}: {
  /** Número do grupo (1, 2…) quando a condição está dentro de um grupo. */
  group?: number;
  c: DraftCondition;
  index: number;
  catalog: readonly IndicatorInfo[];
  onChange: (c: DraftCondition) => void;
  onRemove: () => void;
  error?: string;
  joiner?: string;
}) {
  const kind = fieldKind(c.field);
  const a = arity(c.op);
  const suffix = unitSuffix(c.field, catalog);
  const n = group ? t('pesquisa-cond-n-grupo', { n: index + 1, grupo: group }) : String(index + 1);
  return (
    <li className="flex flex-col gap-1" data-condition={c.id}>
      <div className="flex flex-wrap items-center gap-2">
        <span className="label-caps w-8 text-right">{joiner ?? ''}</span>
        <div className="w-60">
          <Select
            aria-label={t('pesquisa-cond-campo', { n })}
            value={c.field}
            onValueChange={(f) => onChange(changeField(c, f))}
            options={fieldOptions(catalog)}
          />
        </div>
        <div className="w-36">
          <Select
            aria-label={t('pesquisa-cond-operador', { n })}
            value={c.op}
            onValueChange={(op) => onChange({ ...c, op: op as CmpOp })}
            options={opsFor(kind).map((op) => ({ value: op, label: OP_LABEL[op] }))}
          />
        </div>
        {a === 'none' ? null : kind === 'bool' ? (
          <div className="w-28">
            <Select
              aria-label={t('pesquisa-cond-valor', { n })}
              value={c.value === 'false' ? 'false' : 'true'}
              onValueChange={(v) => onChange({ ...c, value: v })}
              options={[
                { value: 'true', label: t('comum-sim') },
                { value: 'false', label: t('comum-nao') },
              ]}
            />
          </div>
        ) : c.field === 'class' && a === 'one' ? (
          <div className="w-44">
            <Select
              aria-label={t('pesquisa-cond-valor', { n })}
              value={c.value}
              placeholder={t('pesquisa-cond-classe')}
              onValueChange={(v) => onChange({ ...c, value: v })}
              options={FUNDAMENTAL_CLASSES.map((k) => ({ value: k, label: CLASS_LABEL[k] }))}
            />
          </div>
        ) : (
          <>
            <div className={kind === 'numeric' ? 'w-32' : 'w-56'}>
              <Input
                aria-label={
                  a === 'two' ? t('pesquisa-cond-minimo', { n }) : t('pesquisa-cond-valor', { n })
                }
                numeric={kind === 'numeric'}
                inputMode={kind === 'numeric' ? 'decimal' : undefined}
                placeholder={
                  kind === 'numeric'
                    ? a === 'two'
                      ? t('pesquisa-cond-min')
                      : '0'
                    : a === 'list'
                      ? t('pesquisa-cond-lista')
                      : t('pesquisa-cond-texto')
                }
                value={c.value}
                invalid={Boolean(error)}
                suffix={kind === 'numeric' ? suffix : undefined}
                onChange={(e) => onChange({ ...c, value: e.target.value })}
              />
            </div>
            {a === 'two' ? (
              <>
                <span className="text-[12px] text-dim">{t('pesquisa-cond-e')}</span>
                <div className="w-32">
                  <Input
                    aria-label={t('pesquisa-cond-maximo', { n })}
                    numeric
                    inputMode="decimal"
                    placeholder={t('pesquisa-cond-max')}
                    value={c.value2}
                    invalid={Boolean(error)}
                    suffix={suffix}
                    onChange={(e) => onChange({ ...c, value2: e.target.value })}
                  />
                </div>
              </>
            ) : null}
          </>
        )}
        <IconButton
          label={t('pesquisa-cond-remover', { n })}
          size="sm"
          icon={<Icon name="x" size={12} />}
          onClick={onRemove}
        />
      </div>
      {error ? (
        <p role="alert" className="ml-10 text-[12px] text-negative">
          {error}
        </p>
      ) : null}
    </li>
  );
}

function ConditionList({
  conditions,
  logic,
  catalog,
  errors,
  onChange,
  label,
  group,
}: {
  group?: number;
  conditions: DraftCondition[];
  logic: Logic;
  catalog: readonly IndicatorInfo[];
  errors: Record<string, string>;
  onChange: (list: DraftCondition[]) => void;
  label: string;
}) {
  return (
    <ul className="flex flex-col gap-2" aria-label={label}>
      {conditions.map((c, i) => (
        <ConditionRow
          key={c.id}
          c={c}
          index={i}
          group={group}
          catalog={catalog}
          joiner={i === 0 ? undefined : LOGIC_LABEL[logic]}
          error={errors[c.id]}
          onChange={(next) => onChange(conditions.map((x) => (x.id === c.id ? next : x)))}
          onRemove={() => onChange(conditions.filter((x) => x.id !== c.id))}
        />
      ))}
    </ul>
  );
}

export function ConditionBuilder({
  draft,
  onChange,
  catalog,
  errors,
}: {
  draft: DraftQuery;
  onChange: (d: DraftQuery) => void;
  catalog: readonly IndicatorInfo[];
  errors: Record<string, string>;
}) {
  const setGroup = (g: DraftGroup) =>
    onChange({ ...draft, groups: draft.groups.map((x) => (x.id === g.id ? g : x)) });
  const parts = draft.conditions.length + draft.groups.length;
  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-3">
        <span className="label-caps">{t('pesquisa-combinar')}</span>
        <LogicSwitch
          label={t('pesquisa-combinar-tudo')}
          value={draft.logic}
          onChange={(logic) => onChange({ ...draft, logic })}
        />
        <span className="text-[12px] text-dim">
          {parts === 0 ? t('pesquisa-sem-condicoes') : null}
        </span>
      </div>
      <ConditionList
        label={t('pesquisa-condicoes')}
        conditions={draft.conditions}
        logic={draft.logic}
        catalog={catalog}
        errors={errors}
        onChange={(conditions) => onChange({ ...draft, conditions })}
      />
      {draft.groups.map((g, gi) => (
        <section
          key={g.id}
          aria-label={t('pesquisa-grupo-n', { n: gi + 1 })}
          className="flex flex-col gap-2 rounded-card border border-dashed border-line-strong/70 px-3 py-2.5"
        >
          <div className="flex flex-wrap items-center gap-3">
            <span className="label-caps text-fg">
              {draft.conditions.length || gi > 0 ? `${LOGIC_LABEL[draft.logic]} · ` : ''}
              {t('pesquisa-grupo-n', { n: gi + 1 })}
            </span>
            <LogicSwitch
              label={t('pesquisa-combinar-grupo', { n: gi + 1 })}
              value={g.logic}
              onChange={(logic) => setGroup({ ...g, logic })}
            />
            <Button
              size="sm"
              variant="ghost"
              className="ml-auto"
              onClick={() =>
                onChange({ ...draft, groups: draft.groups.filter((x) => x.id !== g.id) })
              }
            >
              {t('pesquisa-remover-grupo')}
            </Button>
          </div>
          <ConditionList
            label={t('pesquisa-condicoes-grupo', { n: gi + 1 })}
            group={gi + 1}
            conditions={g.conditions}
            logic={g.logic}
            catalog={catalog}
            errors={errors}
            onChange={(conditions) => setGroup({ ...g, conditions })}
          />
          {errors[g.id] ? (
            <p role="alert" className="text-[12px] text-negative">
              {errors[g.id]}
            </p>
          ) : null}
          <Button
            size="sm"
            variant="ghost"
            className="self-start"
            icon={<Icon name="plus" size={11} />}
            onClick={() =>
              setGroup({ ...g, conditions: [...g.conditions, newCondition('dy', 'gte')] })
            }
          >
            {t('pesquisa-condicao-no-grupo')}
          </Button>
        </section>
      ))}
      <div className="flex flex-wrap gap-2">
        <Button
          size="sm"
          icon={<Icon name="plus" size={11} />}
          onClick={() => onChange({ ...draft, conditions: [...draft.conditions, newCondition()] })}
        >
          {t('pesquisa-adicionar-condicao')}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          icon={<Icon name="plus" size={11} />}
          onClick={() => onChange({ ...draft, groups: [...draft.groups, newGroup()] })}
        >
          {t('pesquisa-adicionar-grupo')}
        </Button>
      </div>
    </div>
  );
}
