/**
 * Informes de rendimentos do ano (`tax_burden.*_income_report`): com o informe de uma fonte
 * pagadora, a renda do trabalho do membro usa os valores dele (bruto, INSS e IR retido) no
 * lugar da estimativa a partir do salário líquido.
 */
import { useState } from 'react';

import type { IncomeReport } from '@/api/generated/IncomeReport';
import type { IncomeReportInput } from '@/api/generated/IncomeReportInput';
import { useMembers } from '@/api/hooks/household';
import {
  useDeleteIncomeReport,
  useIncomeReports,
  useSaveIncomeReport,
} from '@/api/hooks/taxBurden';
import {
  Button,
  Dialog,
  DialogClose,
  Field,
  Icon,
  Input,
  Money,
  MoneyInput,
  Select,
  TechTable,
  toast,
} from '@/design/components';
import { ConfirmButton, errorMessage } from '@/features/portfolio/components/States';
import { t } from '@/i18n';

const FAMILY = '__familia';

interface Draft {
  id?: string;
  payer: string;
  member: string;
  gross: number | null;
  social_security: number | null;
  income_tax: number | null;
  thirteenth_gross: number | null;
  thirteenth_tax: number | null;
  notes: string;
}

const emptyDraft = (): Draft => ({
  payer: '',
  member: FAMILY,
  gross: null,
  social_security: null,
  income_tax: null,
  thirteenth_gross: null,
  thirteenth_tax: null,
  notes: '',
});

const toDraft = (r: IncomeReport): Draft => ({
  id: r.id,
  payer: r.payer,
  member: r.member_id ?? FAMILY,
  gross: r.gross,
  social_security: r.social_security,
  income_tax: r.income_tax,
  thirteenth_gross: r.thirteenth_gross,
  thirteenth_tax: r.thirteenth_tax,
  notes: r.notes ?? '',
});

export function IncomeReportDialog({
  year,
  currency,
  draft: initial,
  onClose,
}: {
  year: number;
  currency: string;
  draft: Draft;
  onClose: () => void;
}) {
  const [d, setD] = useState<Draft>(initial);
  const members = useMembers().data ?? [];
  const save = useSaveIncomeReport();
  const set = <K extends keyof Draft>(k: K, v: Draft[K]) => setD((x) => ({ ...x, [k]: v }));
  const valid = d.payer.trim() !== '' && d.gross != null && d.gross >= 0;
  const submit = () => {
    if (!valid) return;
    const input: IncomeReportInput = {
      ...(d.id ? { id: d.id } : {}),
      year,
      payer: d.payer.trim(),
      gross: d.gross ?? 0,
      social_security: d.social_security ?? 0,
      income_tax: d.income_tax ?? 0,
      thirteenth_gross: d.thirteenth_gross ?? 0,
      thirteenth_tax: d.thirteenth_tax ?? 0,
      ...(d.member !== FAMILY ? { member_id: d.member } : {}),
      ...(d.notes.trim() ? { notes: d.notes.trim() } : {}),
    };
    save.mutate(input, {
      onSuccess: () => {
        toast({
          tone: 'success',
          title: d.id ? t('impostos-informe-alterado') : t('impostos-informe-lancado'),
          description: d.payer.trim(),
        });
        onClose();
      },
      onError: (e) =>
        toast({ tone: 'error', title: t('impostos-erro-salvar'), description: errorMessage(e) }),
    });
  };
  const money = (k: keyof Draft, label: string, hint?: string) => (
    <Field label={label} hint={hint}>
      <MoneyInput
        value={d[k] as number | null}
        onValueChange={(v) => set(k, v as never)}
        currency={currency}
      />
    </Field>
  );
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      title={
        d.id
          ? t('impostos-informe-editar-titulo', { ano: String(year) })
          : t('impostos-informe-lancar-titulo', { ano: String(year) })
      }
      description={t('impostos-informe-descricao')}
      size="md"
      footer={
        <>
          <DialogClose asChild>
            <Button variant="ghost">{t('comum-cancelar')}</Button>
          </DialogClose>
          <Button variant="primary" loading={save.isPending} disabled={!valid} onClick={submit}>
            {t('comum-salvar')}
          </Button>
        </>
      }
    >
      <form
        className="grid gap-3 @container sm:grid-cols-2"
        onSubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <Field label={t('impostos-fonte-pagadora')} className="sm:col-span-2">
          <Input value={d.payer} onChange={(e) => set('payer', e.target.value)} autoFocus />
        </Field>
        <Field label={t('impostos-membro')} className="sm:col-span-2">
          <Select
            aria-label={t('impostos-membro')}
            value={d.member}
            onValueChange={(v) => set('member', v)}
            options={[
              { value: FAMILY, label: t('impostos-sem-membro-casa') },
              ...members.map((m) => ({ value: m.id, label: m.name })),
            ]}
          />
        </Field>
        {money('gross', t('impostos-rendimentos-tributaveis'), t('impostos-sem-13'))}
        {money('social_security', t('impostos-previdencia-oficial'))}
        {money('income_tax', t('impostos-ir-retido-fonte'), t('impostos-sem-ir-13'))}
        {money('thirteenth_gross', t('impostos-13-salario'))}
        {money('thirteenth_tax', t('impostos-ir-13'))}
        <Field label={t('impostos-observacoes')} className="sm:col-span-2">
          <Input value={d.notes} onChange={(e) => set('notes', e.target.value)} />
        </Field>
        <button type="submit" hidden aria-hidden tabIndex={-1} />
      </form>
    </Dialog>
  );
}

export function IncomeReportsPanel({ year, currency }: { year: number; currency: string }) {
  const q = useIncomeReports(year);
  const members = useMembers(true).data ?? [];
  const del = useDeleteIncomeReport();
  const [draft, setDraft] = useState<Draft | null>(null);
  const rows = q.data ?? [];
  const memberName = (id: string | null) =>
    id ? (members.find((m) => m.id === id)?.name ?? '—') : t('impostos-da-casa');
  const ano = String(year);
  return (
    <section aria-label={t('impostos-informes-de', { ano })} className="flex flex-col gap-3">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex min-w-0 flex-col gap-0.5">
          <h3 className="label-caps text-fg">{t('impostos-informes-titulo', { ano })}</h3>
          <p className="text-[12px] text-dim">{t('impostos-informes-texto')}</p>
        </div>
        <Button
          size="sm"
          icon={<Icon name="plus" size={12} />}
          onClick={() => setDraft(emptyDraft())}
        >
          {t('impostos-lancar-informe')}
        </Button>
      </header>
      {rows.length > 0 ? (
        <TechTable<IncomeReport>
          label={t('impostos-informes-de', { ano })}
          prefsId="taxburden.reports"
          dense
          rows={rows}
          getRowId={(r) => r.id}
          minWidth="760px"
          customizable={false}
          onActivate={(r) => setDraft(toDraft(r))}
          rowMenu={(r) => [
            { label: t('impostos-editar-menu'), onSelect: () => setDraft(toDraft(r)) },
            { separator: true },
            {
              label: t('impostos-excluir'),
              danger: true,
              onSelect: () =>
                del.mutate(
                  { id: r.id },
                  { onError: (e) => toast({ tone: 'error', title: errorMessage(e) }) },
                ),
            },
          ]}
          rowMenuLabel={(r) => t('impostos-acoes-informe', { fonte: r.payer })}
          columns={[
            { key: 'payer', header: t('impostos-fonte-pagadora'), hideable: false },
            {
              key: 'member',
              header: t('impostos-membro'),
              sortValue: (r) => memberName(r.member_id),
              cell: (r) => memberName(r.member_id),
            },
            {
              key: 'gross',
              header: t('impostos-bruto'),
              numeric: true,
              cell: (r) => <Money cents={r.gross} currency={currency} />,
            },
            {
              key: 'social_security',
              header: t('impostos-informe-inss'),
              numeric: true,
              cell: (r) => <Money cents={r.social_security} currency={currency} />,
            },
            {
              key: 'income_tax',
              header: t('impostos-ir-retido'),
              numeric: true,
              cell: (r) => <Money cents={r.income_tax} currency={currency} />,
            },
            {
              key: 'thirteenth_gross',
              header: t('impostos-13-bruto'),
              numeric: true,
              cell: (r) => <Money cents={r.thirteenth_gross} currency={currency} />,
            },
            {
              key: 'thirteenth_tax',
              header: t('impostos-ir-do-13'),
              numeric: true,
              cell: (r) => <Money cents={r.thirteenth_tax} currency={currency} />,
            },
            {
              key: 'actions',
              header: '',
              title: t('impostos-acoes'),
              sortable: false,
              align: 'right',
              width: '150px',
              cell: (r) => (
                <span className="inline-flex items-center gap-1">
                  <Button
                    size="sm"
                    variant="ghost"
                    aria-label={t('impostos-editar-informe', { fonte: r.payer })}
                    onClick={() => setDraft(toDraft(r))}
                  >
                    {t('comum-editar')}
                  </Button>
                  <ConfirmButton
                    label={t('impostos-excluir')}
                    title={t('impostos-excluir-informe')}
                    description={t('impostos-excluir-informe-texto', { fonte: r.payer, ano })}
                    pending={del.isPending}
                    onConfirm={() =>
                      del.mutateAsync({ id: r.id }).catch((e: unknown) => {
                        toast({ tone: 'error', title: errorMessage(e) });
                      })
                    }
                  />
                </span>
              ),
            },
          ]}
        />
      ) : (
        <p className="text-[12.5px] text-dim">{t('impostos-informes-vazio', { ano })}</p>
      )}
      {draft ? (
        <IncomeReportDialog
          year={year}
          currency={currency}
          draft={draft}
          onClose={() => setDraft(null)}
        />
      ) : null}
    </section>
  );
}
