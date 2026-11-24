/**
 * Proventos a confirmar dentro de "Registrar proventos": o que foi anunciado (agenda do ativo
 * e eventos oficiais da B3) com a família tendo o ativo na data-com e que ainda não está na
 * carteira (`portfolio.pending_income`). Vem tudo preenchido — data, conta, tipo, quantidade e
 * valor —; a família marca o que recebeu, ajusta o valor se o extrato diz outro e confirma.
 * Grava numa alteração só (`portfolio.create_incomes`, um "Desfazer" para todos).
 */
import { useState } from 'react';

import type { IncomeInput } from '@/api/generated/IncomeInput';
import type { PendingIncome } from '@/api/generated/PendingIncome';
import { useCreateIncomes, useInvestmentAccounts, usePendingIncome } from '@/api/hooks/portfolio';
import { useEngines } from '@/api/hooks/ai';
import { Button, DateInput, Money, toast, Tooltip } from '@/design/components';
import { cx } from '@/design/cx';
import { tRich } from '@/features/settings/richText';
import { Checkbox } from '@/features/transactions/ui/controls';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t } from '@/i18n';
import { formatDate, parseMoney } from '@/lib/format';
import { useCompanion } from '@/state/companion';
import { FormError } from '@/vault/FormError';

import { pendingKey, useDismissedIncome } from '../assets/PendingIncome';
import { formatQuantity, formatUnitPrice } from './decimal';
import { INCOME_KIND_LABEL, sourceName } from './labels';
import { maskAmount, typeAmount } from './incomeMasks';

/** Nome curto da fonte no selo da conferência. */
const shortSource = (s: string) =>
  ({ b3_listados: 'B3', fundos_net: 'Fundos.NET', yahoo: 'Yahoo', fundamentus: 'Fundamentus' })[
    s
  ] ?? sourceName(s);

const STATUS: Record<
  PendingIncome['check']['status'],
  { mark: string; readonly text: string; tone: string }
> = {
  confirmed: {
    mark: '✓',
    get text() {
      return t('carteira-conf-confere');
    },
    tone: 'text-positive',
  },
  divergent: {
    mark: '⚠',
    get text() {
      return t('carteira-conf-divergem');
    },
    tone: 'text-warning',
  },
  single: {
    mark: '·',
    get text() {
      return t('carteira-conf-uma-fonte');
    },
    tone: 'text-dim',
  },
};

/** Conferência entre as fontes, com o que cada uma diz no detalhe. */
function CheckBadge({ p }: { p: PendingIncome }) {
  const st = STATUS[p.check.status];
  const names = p.check.sources.map((x) => shortSource(x.source)).join(` ${t('carteira-e')} `);
  return (
    <Tooltip
      side="right"
      content={
        <div className="flex max-w-80 flex-col gap-1 text-[12px]">
          {p.check.sources.map((x) => {
            const fonte = <span className="font-medium">{shortSource(x.source)}</span>;
            const valor = formatUnitPrice(x.amount_per_share, p.currency);
            const data = formatDate(x.ex_date);
            return (
              <span key={x.source}>
                {x.pay_date
                  ? tRich('carteira-conf-fonte-pago', {
                      fonte,
                      valor,
                      data,
                      pago: formatDate(x.pay_date),
                    })
                  : tRich('carteira-conf-fonte', { fonte, valor, data })}
              </span>
            );
          })}
          {p.check.note ? <span className="text-dim">{p.check.note}</span> : null}
        </div>
      }
    >
      <span
        tabIndex={0}
        className={cx(
          'mt-0.5 inline-flex w-fit cursor-help items-center gap-1 text-[11.5px]',
          st.tone,
        )}
      >
        <span aria-hidden>{st.mark}</span>
        {p.check.status === 'divergent'
          ? t('carteira-conferencia-usado', {
              status: st.text,
              fontes: names,
              fonte: shortSource(p.check.reference),
            })
          : t('carteira-conferencia', { status: st.text, fontes: names })}
      </span>
    </Tooltip>
  );
}

/** Ativo sem cotação nova e sem negócios (deslistado ou incorporado?). */
export function AssetWarning({ p }: { p: PendingIncome }) {
  return p.asset_warning ? (
    <span className="mt-0.5 block text-[11.5px] text-warning first-letter:uppercase">
      {p.asset_warning}
    </span>
  ) : null;
}

/** Pedido ao agente: conferir os proventos em dúvida e propor a correção (o usuário aprova). */
export function agentPrompt(items: readonly PendingIncome[]): string {
  const lines = items.map((p) => {
    const srcs = p.check.sources
      .map((x) =>
        x.pay_date
          ? t('carteira-agente-fonte-pago', {
              fonte: shortSource(x.source),
              valor: x.amount_per_share,
              data: x.ex_date,
              pago: x.pay_date,
            })
          : t('carteira-agente-fonte', {
              fonte: shortSource(x.source),
              valor: x.amount_per_share,
              data: x.ex_date,
            }),
      )
      .join('; ');
    const item = t('carteira-agente-item', {
      ativo: p.asset.ticker ?? p.asset.name,
      tipo: INCOME_KIND_LABEL[p.kind],
      data: p.ex_date,
      fontes: srcs,
    });
    return `${item}${p.check.note ? ` — ${p.check.note}` : ''}${p.asset_warning ? ` — ${p.asset_warning}` : ''}`;
  });
  return [t('carteira-agente-pedido'), ...lines, t('carteira-agente-fecho')].join('\n');
}

/** Pendentes ainda não dispensados nesta sessão. */
export function usePendingToConfirm(): PendingIncome[] {
  const q = usePendingIncome();
  const dismissed = useDismissedIncome((s) => s.keys);
  return (q.data ?? []).filter((p) => !dismissed.has(pendingKey(p)));
}

/** Entrada do provento com o que a família ajustou (data quando a fonte não deu, valor). */
export function confirmedInput(p: PendingIncome, net: number, payDate: string): IncomeInput {
  const tax = p.tax_withheld;
  return { ...p.input, pay_date: payDate, gross: net + tax, tax_withheld: tax };
}

export function PendingIncomeConfirm({ onDone }: { onDone: () => void }) {
  const rows = usePendingToConfirm();
  const accounts = useInvestmentAccounts().data ?? [];
  const save = useCreateIncomes();
  const [off, setOff] = useState<ReadonlySet<string>>(new Set());
  const [amounts, setAmounts] = useState<Record<string, string>>({});
  const [payDates, setPayDates] = useState<Record<string, string>>({});
  const accountName = (id: string) =>
    accounts.find((a) => a.id === id)?.name ?? t('carteira-conta');
  // Com um motor de IA ligado, o agente confere o que diverge ou tem uma fonte só.
  const hasAgent = (useEngines().data ?? []).some((e) => e.enabled);
  const doubtful = rows.filter((p) => p.check.status !== 'confirmed');

  const netOf = (p: PendingIncome) => {
    const text = amounts[pendingKey(p)];
    return text == null ? p.net : (parseMoney(text) ?? 0);
  };
  const chosen = rows.filter((p) => !off.has(pendingKey(p)) && netOf(p) > 0);
  const total = chosen.reduce((s, p) => s + netOf(p), 0);
  const currencies = new Set(chosen.map((p) => p.currency));
  const currency = currencies.size === 1 ? [...currencies][0]! : null;
  const toggle = (k: string, on: boolean) =>
    setOff((prev) => {
      const next = new Set(prev);
      if (on) next.delete(k);
      else next.add(k);
      return next;
    });

  const submit = () =>
    save.mutate(
      {
        items: chosen.map((p) =>
          confirmedInput(p, netOf(p), payDates[pendingKey(p)] ?? p.pay_date),
        ),
      },
      {
        onSuccess: (r) => {
          toast({ tone: 'success', title: t('carteira-proventos-registrados', { n: r.created }) });
          onDone();
        },
      },
    );

  if (!rows.length) {
    return <p className="text-[13px] text-dim">{t('carteira-pend-vazio')}</p>;
  }

  const allOn = off.size === 0;
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[12.5px] text-dim">{t('carteira-pend-texto')}</p>
      <table className="w-full border-collapse text-[13px]" aria-label={t('carteira-pend-tabela')}>
        <thead>
          <tr className="label-caps text-left text-dim">
            <th className="w-8 pb-1.5">
              <Checkbox
                label={t('carteira-pend-marcar-todos')}
                hideLabel
                checked={allOn}
                indeterminate={!allOn && off.size < rows.length}
                onChange={(on) => setOff(on ? new Set() : new Set(rows.map(pendingKey)))}
              />
            </th>
            <th className="w-[130px] pb-1.5 pr-2 font-normal">{t('carteira-pagamento')}</th>
            <th className="pb-1.5 pr-2 font-normal">{t('carteira-ativo')}</th>
            <th className="w-[170px] pb-1.5 pr-2 text-right font-normal">
              {t('carteira-pend-qtd-cota')}
            </th>
            <th className="w-[140px] pb-1.5 text-right font-normal">
              {t('carteira-grade-valor-recebido')}
            </th>
          </tr>
        </thead>
        <tbody>
          {rows.map((p) => {
            const k = pendingKey(p);
            const on = !off.has(k);
            const label = p.asset.ticker ?? p.asset.name;
            return (
              <tr key={k} className={cx('border-t border-line align-top', !on && 'opacity-50')}>
                <td className="py-1.5">
                  <Checkbox
                    label={t('carteira-pend-recebi', {
                      ativo: label,
                      data: formatDate(p.pay_date),
                    })}
                    hideLabel
                    checked={on}
                    onChange={(v) => toggle(k, v)}
                  />
                </td>
                <td className="py-1.5 pr-2">
                  {p.pay_date_guessed ? (
                    <span className="flex flex-col gap-0.5">
                      <DateInput
                        aria-label={t('carteira-pend-data-de', { ativo: label })}
                        value={payDates[k] ?? p.pay_date}
                        onValueChange={(d) => d && setPayDates((s) => ({ ...s, [k]: d }))}
                      />
                      <span className="text-[11px] text-warning italic">
                        {t('carteira-pend-sem-data')}
                      </span>
                    </span>
                  ) : (
                    <span className="num">{formatDate(p.pay_date)}</span>
                  )}
                </td>
                <td className="py-1.5 pr-2">
                  <span className="flex flex-col leading-tight">
                    <span className="font-medium text-fg">{label}</span>
                    <span className="text-[11.5px] text-dim">
                      {t('carteira-pend-detalhe', {
                        tipo: INCOME_KIND_LABEL[p.kind],
                        conta: accountName(p.account_id),
                        data: formatDate(p.ex_date),
                      })}
                    </span>
                    <CheckBadge p={p} />
                    <AssetWarning p={p} />
                  </span>
                </td>
                <td className="num py-1.5 pr-2 text-right text-dim">
                  {formatQuantity(p.quantity)} × {formatUnitPrice(p.amount_per_share, p.currency)}
                  {p.tax_withheld ? (
                    <span className="block text-[11px]">
                      {withSlots(t('carteira-ir-valor', { valor: slot(0) }), [
                        <Money key="ir" cents={p.tax_withheld} currency={p.currency} />,
                      ])}
                    </span>
                  ) : null}
                </td>
                <td className="py-1.5 text-right">
                  <input
                    aria-label={t('carteira-pend-valor-de', { ativo: label })}
                    inputMode="decimal"
                    disabled={!on}
                    className="num h-8 w-full rounded-control border border-line bg-bg px-2 text-right text-[13px] outline-none focus:border-accent"
                    value={amounts[k] ?? maskAmount(p.net)}
                    onChange={(e) => setAmounts((s) => ({ ...s, [k]: typeAmount(e.target.value) }))}
                  />
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      <FormError error={save.error} />
      {doubtful.length && hasAgent ? (
        <div className="flex flex-wrap items-center gap-2 rounded-control border border-line bg-bg px-3 py-2 text-[12.5px]">
          <span className="text-dim">{t('carteira-pend-duvidosos', { n: doubtful.length })}</span>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => {
              useCompanion.getState().ask(agentPrompt(doubtful));
              onDone();
            }}
          >
            {t('carteira-pend-pedir-agente')}
          </Button>
        </div>
      ) : null}
      <div className="flex flex-wrap items-center justify-end gap-3 border-t border-line pt-3">
        <span role="status" className="mr-auto text-[12px] text-dim">
          {t('carteira-pend-marcados', { n: chosen.length, total: rows.length })}
          {currency ? (
            <>
              {' · '}
              <Money cents={total} currency={currency} />
            </>
          ) : null}
        </span>
        <Button
          variant="primary"
          disabled={!chosen.length}
          loading={save.isPending}
          onClick={submit}
        >
          {chosen.length
            ? t('carteira-confirmar-n-proventos', { n: chosen.length })
            : t('carteira-confirmar')}
        </Button>
      </div>
    </div>
  );
}
