/**
 * Criptoativos custodiados no Brasil na apuração do mês (`TaxMonth.crypto`): ganho de capital
 * com isenção de R$ 35 mil vendidos no mês e DARF próprio (código 4600), que pode ser gerado
 * e acompanhado na lista de DARFs. Embaixo, onde a cripto de cada conta está custodiada
 * (pela moeda da conta, ou como a família escolher): no exterior, vai para o resumo anual.
 */
import { useCryptoCustody, useGenerateDarf, useSetCryptoCustody } from '@/api/hooks/tax';
import { useUndoToast } from '@/api/hooks/undoToast';
import type { TaxMonth } from '@/api/generated/TaxMonth';
import { Button, KeyValue, Money } from '@/design/components';
import { InlineError, SectionTitle, Segmented } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';
import { formatDate, formatMonthLong } from '@/lib/format';

import { CopyButton } from './CopyValue';
import { copyText } from './transform';

type Place = 'auto' | 'brazil' | 'abroad';

export function CryptoMonthCard({
  month: m,
  memberId,
}: {
  month: TaxMonth;
  memberId: string | null;
}) {
  const generate = useGenerateDarf();
  const undoToast = useUndoToast();
  const custody = useCryptoCustody();
  const setCustody = useSetCryptoCustody();
  const c = m.crypto;
  if (!c) return null;
  const accounts = (custody.data ?? []).filter((a) => a.has_crypto);
  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <SectionTitle>{t('ir-cripto-titulo')}</SectionTitle>
        {c.darf_amount > 0 && !c.darf_id ? (
          <Button
            variant="primary"
            size="sm"
            loading={generate.isPending}
            onClick={() =>
              generate.mutate(
                {
                  month: m.month,
                  revenue_code: c.revenue_code,
                  ...(memberId ? { member_id: memberId } : {}),
                },
                {
                  onSuccess: (d) =>
                    undoToast({
                      tone: 'success',
                      title: t('ir-darf-gerado'),
                      description: t('ir-darf-gerado-texto', {
                        codigo: d.revenue_code,
                        vencimento: formatDate(d.due_date),
                      }),
                    }),
                },
              )
            }
          >
            {t('ir-gerar-darf', { valor: copyText(c.darf_amount) })}
          </Button>
        ) : c.darf_id ? (
          <span className="label-caps text-dim">
            {c.paid ? t('ir-cripto-darf-pago') : t('ir-cripto-darf-gerado')}
          </span>
        ) : null}
      </div>
      <InlineError error={generate.error} />
      <div className="rounded-card border border-line p-3.5">
        <KeyValue
          layout="stacked"
          items={[
            { label: t('ir-cripto-vendido'), value: <Money cents={c.sales} /> },
            ...(c.sales !== c.sales_brazil
              ? [{ label: t('ir-cripto-vendido-brasil'), value: <Money cents={c.sales_brazil} /> }]
              : []),
            { label: t('ir-cripto-ganho'), value: <Money cents={c.gain} /> },
            ...(c.loss > 0
              ? [{ label: t('ir-cripto-perda'), value: <Money cents={-c.loss} /> }]
              : []),
            {
              label: t('ir-cripto-situacao'),
              value: c.exempt ? t('ir-cripto-isento') : t('ir-cripto-tributado'),
            },
            { label: t('ir-imposto-mes'), value: <Money cents={c.tax} /> },
            ...(c.carried_in > 0
              ? [{ label: t('ir-acumulado'), value: <Money cents={c.carried_in} /> }]
              : []),
            {
              label: 'DARF',
              value:
                c.darf_amount > 0 ? (
                  <span className="inline-flex items-center gap-2">
                    <Money cents={c.darf_amount} className="font-medium" />
                    <CopyButton
                      text={copyText(c.darf_amount)}
                      label={t('ir-valor-darf-de', { periodo: formatMonthLong(m.month) })}
                    />
                  </span>
                ) : (
                  '—'
                ),
            },
            { label: t('ir-codigo-receita'), value: c.revenue_code },
            { label: t('ir-vencimento'), value: formatDate(c.due_date) },
            ...(c.carried_out > 0
              ? [{ label: t('ir-vai-mes-seguinte'), value: <Money cents={c.carried_out} /> }]
              : []),
          ]}
        />
        <p className="mt-3 text-[12px] text-dim">{t('ir-cripto-nota')}</p>
        {accounts.length > 0 ? (
          <div className="mt-3 flex flex-col gap-2 border-t border-line pt-3">
            <p className="text-[12px] text-dim">{t('ir-cripto-custodia-dica')}</p>
            {accounts.map((a) => {
              const value: Place = !a.chosen ? 'auto' : a.abroad ? 'abroad' : 'brazil';
              return (
                <div
                  key={a.account_id}
                  className="flex flex-wrap items-center justify-between gap-2"
                >
                  <span className="text-[13px]">
                    {a.name} <span className="text-dim">· {a.currency}</span>
                  </span>
                  <Segmented<Place>
                    size="sm"
                    label={t('ir-cripto-custodia-de', { conta: a.name })}
                    value={value}
                    onChange={(v) =>
                      setCustody.mutate({
                        account_id: a.account_id,
                        abroad: v === 'auto' ? null : v === 'abroad',
                      })
                    }
                    options={[
                      {
                        value: 'auto',
                        label: t('ir-cripto-custodia-auto', {
                          lugar:
                            a.currency === 'BRL' ? t('ir-cripto-brasil') : t('ir-cripto-exterior'),
                        }),
                      },
                      { value: 'brazil', label: t('ir-cripto-brasil') },
                      { value: 'abroad', label: t('ir-cripto-exterior') },
                    ]}
                  />
                </div>
              );
            })}
            <InlineError error={setCustody.error} />
          </div>
        ) : null}
      </div>
    </div>
  );
}
