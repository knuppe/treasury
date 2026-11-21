/**
 * Detalhe da meta de faculdade (docs/design/college-goal.md §9.3): o resultado de hoje, a
 * tabela ano a ano, o que está guardado (posições ligadas) e o histórico dos recálculos.
 */
import { useState } from 'react';

import type { ReviewReason } from '@/api/generated/ReviewReason';
import { useEducationPlan } from '@/api/hooks/goals';
import {
  Button,
  Dialog,
  Money,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '@/design/components';
import { cx } from '@/design/cx';
import { QueryView } from '@/features/reports/controls';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { EducationDialog } from './EducationDialog';
import { EducationResultPanel, spreadLabel } from './EducationResultPanel';
import { HoldingsEditor } from './HoldingsEditor';
import { institutionLabel } from './model';

const REASON: Record<ReviewReason, () => string> = {
  annual: () => t('educacao-revisao-anual'),
  edit: () => t('educacao-revisao-edicao'),
  fee_update: () => t('educacao-revisao-mensalidade'),
  created: () => t('educacao-revisao-criada'),
};

export function EducationDetail({
  goalId,
  open,
  onOpenChange,
}: {
  goalId: string;
  open: boolean;
  onOpenChange: (o: boolean) => void;
}) {
  const plan = useEducationPlan(open ? goalId : null);
  const [editing, setEditing] = useState<null | 0 | 1>(null);
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="xl"
      title={plan.data?.goal.name ?? t('educacao-plano')}
      description={
        plan.data
          ? [
              plan.data.child_name,
              institutionLabel(plan.data.plan.institution_type),
              plan.data.plan.course,
              t('educacao-periodo', {
                inicio: formatDate(plan.data.result.start_date),
                fim: formatDate(plan.data.result.end_date),
              }),
            ]
              .filter(Boolean)
              .join(' · ')
          : undefined
      }
    >
      <QueryView query={plan}>
        {(v) => (
          <div className="flex flex-col gap-4">
            <EducationResultPanel result={v.result} />
            <div className="flex flex-wrap gap-2">
              <Button variant="secondary" size="sm" onClick={() => setEditing(1)}>
                {t('educacao-atualizar-mensalidade')}
              </Button>
              <Button variant="ghost" size="sm" onClick={() => setEditing(0)}>
                {t('educacao-editar-plano')}
              </Button>
            </div>
            <Tabs defaultValue="anos">
              <TabsList aria-label={t('educacao-plano')}>
                <TabsTrigger value="anos">{t('educacao-aba-anos')}</TabsTrigger>
                <TabsTrigger value="guardado">{t('educacao-aba-guardado')}</TabsTrigger>
                <TabsTrigger value="historico">{t('educacao-aba-historico')}</TabsTrigger>
              </TabsList>
              <TabsContent value="anos">
                <div className="overflow-x-auto">
                  <table className="w-full text-[12.5px]">
                    <caption className="sr-only">{t('educacao-aba-anos')}</caption>
                    <thead>
                      <tr className="label-caps text-left">
                        <th scope="col" className="py-1.5 pr-3">
                          {t('educacao-col-ano')}
                        </th>
                        <th scope="col" className="py-1.5 pr-3 text-right">
                          {t('educacao-col-mensalidade-hoje')}
                        </th>
                        <th scope="col" className="py-1.5 pr-3 text-right">
                          {t('educacao-col-mensalidade-epoca')}
                        </th>
                        <th scope="col" className="py-1.5 pr-3 text-right">
                          {t('educacao-col-pagamentos')}
                        </th>
                        <th scope="col" className="py-1.5 text-right">
                          {t('educacao-col-saldo')}
                        </th>
                      </tr>
                    </thead>
                    <tbody>
                      {v.result.years.map((y) => (
                        <tr
                          key={y.year}
                          className={cx('border-t border-line', y.in_course && 'bg-accent/5')}
                        >
                          <th scope="row" className="num py-1.5 pr-3 text-left font-normal text-fg">
                            {y.year}
                            {y.in_course ? (
                              <span className="label-caps ml-2 text-accent">
                                {t('educacao-em-curso')}
                              </span>
                            ) : null}
                          </th>
                          <td className="num py-1.5 pr-3 text-right">
                            <Money cents={y.fee_real} currency={v.goal.currency} />
                          </td>
                          <td className="num py-1.5 pr-3 text-right text-dim">
                            <Money cents={y.fee_nominal} currency={v.goal.currency} />
                          </td>
                          <td className="num py-1.5 pr-3 text-right">
                            {y.payments_real ? (
                              <Money cents={y.payments_real} currency={v.goal.currency} />
                            ) : (
                              '—'
                            )}
                          </td>
                          <td className="num py-1.5 text-right">
                            <Money cents={y.planned_balance} currency={v.goal.currency} />
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                <p className="mt-2 text-[12px] text-dim">{t('educacao-tabela-nota')}</p>
              </TabsContent>
              <TabsContent value="guardado">
                <div className="flex flex-col gap-3">
                  <p className="text-[12.5px] text-fg">
                    <Money cents={v.goal.current_amount} currency={v.goal.currency} />{' '}
                    <span className="text-dim">{t('educacao-guardado-hoje')}</span>
                  </p>
                  <HoldingsEditor goalId={v.goal.id} targetDate={v.result.start_date} />
                </div>
              </TabsContent>
              <TabsContent value="historico">
                {v.reviews.length ? (
                  <ul className="flex flex-col divide-y divide-line text-[12.5px]">
                    {v.reviews.map((r) => (
                      <li key={r.id} className="flex flex-wrap items-baseline gap-x-3 gap-y-1 py-2">
                        <span className="num text-fg">
                          {formatDate(r.reviewed_at.slice(0, 10))}
                        </span>
                        <span className="label-caps">{REASON[r.reason]()}</span>
                        <span className="num text-fg">
                          <Money cents={r.monthly_needed} currency={v.goal.currency} />
                          <span className="text-dim">{t('metas-por-mes')}</span>
                        </span>
                        <span className="text-dim">{spreadLabel(r.spread)}</span>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p className="text-[12.5px] text-dim">{t('educacao-historico-vazio')}</p>
                )}
              </TabsContent>
            </Tabs>
            {editing != null ? (
              <EducationDialog
                plan={v}
                open
                initialStep={editing}
                onOpenChange={(o) => {
                  if (!o) setEditing(null);
                }}
              />
            ) : null}
          </div>
        )}
      </QueryView>
    </Dialog>
  );
}
