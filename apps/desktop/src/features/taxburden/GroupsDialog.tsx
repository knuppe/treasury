/**
 * "Grupos das categorias": todas as categorias de despesa com o grupo de imposto que vale
 * (escolha, padrão pelo nome ou herdado do pai) e a alíquota própria. Para usuários
 * avançados; cada mudança grava na hora (`tax_burden.set_category`) e dá para desfazer no
 * histórico.
 */
import { useMemo, useState, type KeyboardEvent } from 'react';

import type { TaxCategoryView } from '@/api/generated/TaxCategoryView';
import type { TaxDataset } from '@/api/generated/TaxDataset';
import { useSetTaxCategory, useTaxCategories } from '@/api/hooks/taxBurden';
import { Button, Dialog, DialogClose, Icon, Input, Select, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { errorMessage, LoadingBlock } from '@/features/portfolio/components/States';
import { t } from '@/i18n';

import { groupOptions, parseRate } from './CategoryDialogs';
import { groupLabel, ORIGIN_LABEL, rateInputText } from './model';

const DEFAULT = '__padrao';

/** Busca sem acentos nem maiúsculas. */
const searchFold = (s: string) => s.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase();

export function GroupsDialog({
  dataset,
  open,
  onOpenChange,
}: {
  dataset: TaxDataset | undefined;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const q = useTaxCategories(open);
  const [filter, setFilter] = useState('');
  const rows = useMemo(() => {
    const f = searchFold(filter.trim());
    return (q.data ?? []).filter(
      (c) =>
        !f ||
        searchFold(c.path).includes(f) ||
        searchFold(groupLabel(c.group_key, dataset)).includes(f),
    );
  }, [q.data, filter, dataset]);
  const options = useMemo(() => groupOptions(dataset, t('impostos-padrao-app')), [dataset]);
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title={t('impostos-grupos-categorias')}
      description={t('impostos-grupos-descricao')}
      size="xl"
      resizable
      resizeId="taxburden.groups"
      footer={
        <DialogClose asChild>
          <Button>{t('comum-fechar')}</Button>
        </DialogClose>
      }
    >
      <Input
        aria-label={t('impostos-filtrar-categorias')}
        placeholder={t('impostos-filtrar-placeholder')}
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
        prefix={<Icon name="search" size={13} />}
        wrapperClassName="shrink-0"
      />
      {q.isPending ? (
        <LoadingBlock label={t('impostos-carregando-categorias')} />
      ) : (
        <div className="shrink-0 overflow-x-auto">
          <table
            className="w-full min-w-[720px] text-[12.5px]"
            aria-label={t('impostos-grupos-categorias')}
          >
            <thead>
              <tr className="label-caps text-left">
                <th className="py-1.5 pr-3 font-normal">{t('impostos-categoria')}</th>
                <th className="py-1.5 pr-3 font-normal">{t('impostos-grupo')}</th>
                <th className="w-36 py-1.5 pr-3 font-normal">{t('impostos-aliquota-propria')}</th>
                <th className="w-32 py-1.5 font-normal">
                  <span className="sr-only">{t('impostos-acoes')}</span>
                </th>
              </tr>
            </thead>
            <tbody>
              {rows.map((c) => (
                <GroupRow
                  key={`${c.category_id}:${c.rate ?? ''}`}
                  row={c}
                  dataset={dataset}
                  options={
                    c.default_group_key
                      ? groupOptions(
                          dataset,
                          t('impostos-padrao-app-grupo', {
                            grupo: groupLabel(c.default_group_key, dataset),
                          }),
                        )
                      : options
                  }
                />
              ))}
            </tbody>
          </table>
          {rows.length === 0 ? (
            <p className="py-4 text-[13px] text-dim">{t('impostos-filtro-vazio')}</p>
          ) : null}
        </div>
      )}
    </Dialog>
  );
}

function GroupRow({
  row: c,
  dataset,
  options,
}: {
  row: TaxCategoryView;
  dataset: TaxDataset | undefined;
  options: ReturnType<typeof groupOptions>;
}) {
  const set = useSetTaxCategory();
  const [text, setText] = useState(rateInputText(c.rate));
  const save = (patch: { group_key?: string | null; rate?: number | null }) =>
    set.mutate(
      { category_id: c.category_id, ...patch },
      {
        onError: (e) =>
          toast({
            tone: 'error',
            title: t('impostos-erro-salvar'),
            description: errorMessage(e),
          }),
      },
    );
  const commitRate = () => {
    const r = parseRate(text);
    if (text.trim() && (r == null || r < 0 || r >= 100)) {
      toast({
        tone: 'error',
        title: t('impostos-aliquota-invalida'),
        description: t('impostos-aliquota-faixa'),
      });
      return;
    }
    if (r !== c.rate) save({ rate: r });
  };
  const onKey = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      commitRate();
    }
  };
  const custom = c.origin === 'user' || c.rate != null;
  return (
    <tr className="border-t border-line/70 align-middle">
      <td className="py-1.5 pr-3">
        <span className="block truncate text-fg">{c.path}</span>
        <span className={cx('text-[11px]', c.group_key ? 'text-dim' : 'text-warning')}>
          {c.group_key
            ? `${groupLabel(c.group_key, dataset)} · ${ORIGIN_LABEL[c.origin ?? ''] ?? ''}`
            : t('impostos-sem-grupo-fora')}
        </span>
      </td>
      <td className="py-1.5 pr-3">
        <Select
          aria-label={t('impostos-grupo-de', { categoria: c.path })}
          value={c.origin === 'user' && c.group_key ? c.group_key : DEFAULT}
          onValueChange={(v) => save({ group_key: v === DEFAULT ? null : v })}
          options={options}
          disabled={set.isPending}
        />
      </td>
      <td className="py-1.5 pr-3">
        <Input
          aria-label={t('impostos-aliquota-propria-de', { categoria: c.path })}
          numeric
          inputMode="decimal"
          suffix="%"
          placeholder={c.rate == null ? t('impostos-do-pais') : undefined}
          value={text}
          onChange={(e) => setText(e.target.value)}
          onBlur={commitRate}
          onKeyDown={onKey}
        />
      </td>
      <td className="py-1.5 text-right">
        {custom ? (
          <Button
            size="sm"
            variant="ghost"
            aria-label={t('impostos-voltar-padrao-de', { categoria: c.path })}
            disabled={set.isPending}
            onClick={() => {
              setText('');
              save({ group_key: null, rate: null });
            }}
          >
            {t('impostos-voltar-padrao')}
          </Button>
        ) : null}
      </td>
    </tr>
  );
}
