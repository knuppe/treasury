/**
 * Extrato do contrato: o usuário cola o histórico do banco (planilha ou internet banking:
 * data, prestação, amortização, juros, seguros, taxas, saldo devedor, correção e valor
 * pago). A prévia mostra o período, as amortizações extraordinárias achadas pelo saldo, o
 * prazo que falta e a taxa implícita nos juros; ao importar, as parcelas ficam com os
 * valores do banco e o cronograma futuro parte do último saldo.
 */
import { useState } from 'react';

import type { LoanView } from '@/api/generated/LoanView';
import type { StatementPreview } from '@/api/generated/StatementPreview';
import { previewLoanStatement, useImportLoanStatement, useUpdateLoan } from '@/api/hooks/debt';
import { Button, Dialog, Field, Input, KeyValue, Money, toast } from '@/design/components';
import { InlineError, Switch } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { decimalString } from './transform';

export function StatementDialog({
  loan,
  open,
  onOpenChange,
}: {
  loan: LoanView;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [text, setText] = useState('');
  const [term, setTerm] = useState('');
  const [markPrevious, setMarkPrevious] = useState(true);
  const [preview, setPreview] = useState<StatementPreview | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const save = useImportLoanStatement();
  const update = useUpdateLoan();

  const args = () => ({
    id: loan.id,
    text,
    ...(/^\d+$/.test(term.trim()) ? { remaining_term: Number(term.trim()) } : {}),
    mark_previous_paid: markPrevious,
  });
  const check = () => {
    setBusy(true);
    setError(null);
    previewLoanStatement(args())
      .then((p) => {
        setPreview(p);
        if (!term && p.remaining_term != null) setTerm(String(p.remaining_term));
      })
      .catch(setError)
      .finally(() => setBusy(false));
  };
  const commit = () =>
    save.mutate(args(), {
      onSuccess: (r) => {
        toast({
          tone: 'success',
          title: t('dividas-extrato-importadas', { n: r.imported }),
          description: t('dividas-extrato-projecao', {
            data: formatDate(r.preview.last_date ?? ''),
            n: r.preview.remaining_term ?? '',
          }),
        });
        onOpenChange(false);
      },
    });
  const rateOff = preview?.implied_effective_annual_pct && preview.rate_mismatch;

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      resizable
      resizeId="loan-statement"
      title={t('dividas-extrato-titulo')}
      description={t('dividas-extrato-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('comum-cancelar')}
          </Button>
          <Button variant="secondary" loading={busy} disabled={!text.trim()} onClick={check}>
            {t('dividas-extrato-conferir')}
          </Button>
          <Button
            variant="primary"
            loading={save.isPending}
            disabled={!preview || preview.remaining_term == null}
            onClick={commit}
          >
            {preview
              ? t('dividas-extrato-importar-n', { n: preview.rows.length })
              : t('dividas-extrato-importar')}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        <Field label={t('dividas-extrato-colado')} hint={t('dividas-extrato-colado-dica')}>
          <textarea
            aria-label={t('dividas-extrato-colado')}
            value={text}
            onChange={(e) => {
              setText(e.target.value);
              setPreview(null);
            }}
            rows={8}
            spellCheck={false}
            data-selectable
            className="w-full resize-y rounded-control border border-line-strong bg-bg px-2.5 py-2 font-mono text-[12px] text-fg outline-none focus:border-accent"
          />
        </Field>
        <div className="flex flex-wrap items-end gap-4">
          <Field
            label={t('dividas-parcelas-que-faltam')}
            hint={t('dividas-extrato-faltam-dica')}
            className="w-56"
          >
            <Input
              inputMode="numeric"
              numeric
              value={term}
              onChange={(e) => {
                setTerm(e.target.value.replace(/\D/g, ''));
                setPreview(null);
              }}
            />
          </Field>
          <Switch
            checked={markPrevious}
            onChange={setMarkPrevious}
            label={t('dividas-extrato-marcar-anteriores')}
          />
        </div>
        {preview ? (
          <section
            aria-label={t('dividas-extrato-previa')}
            className="flex flex-col gap-3 rounded-card border border-line p-3"
          >
            <KeyValue
              items={[
                { label: t('dividas-parcelas'), value: preview.rows.length },
                {
                  label: t('dividas-extrato-periodo'),
                  value: t('dividas-extrato-periodo-valor', {
                    de: formatDate(preview.first_date ?? ''),
                    ate: formatDate(preview.last_date ?? ''),
                  }),
                },
                {
                  label: t('dividas-extrato-ultimo-saldo'),
                  value:
                    preview.last_balance != null ? <Money cents={preview.last_balance} /> : '—',
                },
                {
                  label: t('dividas-extrato-prazo-restante'),
                  value: preview.remaining_term ?? '—',
                },
                {
                  label: t('dividas-extrato-juros'),
                  value: preview.implied_monthly_pct
                    ? t('dividas-extrato-juros-valor', {
                        mensal: decimalString(preview.implied_monthly_pct),
                        nominal: decimalString(preview.implied_nominal_annual_pct ?? ''),
                        efetiva: decimalString(preview.implied_effective_annual_pct ?? ''),
                      })
                    : '—',
                },
              ]}
            />
            {preview.extras.length ? (
              <div>
                <p className="label-caps mb-1">{t('dividas-extrato-extras')}</p>
                <ul className="text-[12.5px] text-fg" aria-label={t('dividas-extrato-extras-aria')}>
                  {preview.extras.map((e) => (
                    <li key={e.date}>
                      {formatDate(e.date)}: <Money cents={e.amount} />
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
            {preview.warnings.length ? (
              <ul
                className="flex flex-col gap-1 text-[12.5px] text-warning"
                aria-label={t('dividas-extrato-avisos')}
              >
                {preview.warnings.map((w) => (
                  <li key={w} className="first-letter:uppercase">
                    {w}.
                  </li>
                ))}
              </ul>
            ) : null}
            {rateOff ? (
              <div>
                <Button
                  size="sm"
                  variant="secondary"
                  loading={update.isPending}
                  onClick={() =>
                    update.mutate(
                      { id: loan.id, annual_rate: preview.implied_effective_annual_pct! },
                      {
                        onSuccess: () => {
                          toast({
                            tone: 'success',
                            title: t('dividas-extrato-taxa-ajustada', {
                              taxa: decimalString(preview.implied_effective_annual_pct!),
                            }),
                          });
                          check();
                        },
                      },
                    )
                  }
                >
                  {t('dividas-extrato-usar-taxa', {
                    taxa: decimalString(preview.implied_effective_annual_pct!),
                  })}
                </Button>
              </div>
            ) : null}
          </section>
        ) : null}
        <InlineError error={error ?? save.error ?? update.error} />
      </div>
    </Dialog>
  );
}
