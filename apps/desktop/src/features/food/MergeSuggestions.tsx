/**
 * "Talvez sejam o mesmo produto" (aba Produtos de 06 · Consumo): pares de produtos com nomes
 * parecidos, de `food.merge_suggestions`. Juntar mantém o primeiro e põe as compras do segundo
 * nele (`food.product_merge`), depois de confirmar. Só aparece quando há pares.
 */
import { useState } from 'react';

import type { FoodMergeSuggestion } from '@/api/generated/FoodMergeSuggestion';
import { useMergeProducts, useMergeSuggestions } from '@/api/hooks/food';
import { Button, InstrumentCard, toast } from '@/design/components';
import { ConfirmDialog } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

export function MergeSuggestionsCard() {
  const q = useMergeSuggestions();
  const merge = useMergeProducts();
  const [open, setOpen] = useState(false);
  const [pair, setPair] = useState<FoodMergeSuggestion | null>(null);
  const rows = q.data ?? [];
  if (!rows.length) return null;
  return (
    <InstrumentCard
      index="06.19"
      title={t('alimentacao-mesmo-produto')}
      actions={
        <Button size="sm" variant="ghost" aria-expanded={open} onClick={() => setOpen((v) => !v)}>
          {open ? t('alimentacao-mesmo-produto-esconder') : t('alimentacao-mesmo-produto-ver')}
        </Button>
      }
    >
      <p className="text-[12.5px] text-dim">
        {t('alimentacao-mesmo-produto-n', { n: rows.length })}
      </p>
      {open ? (
        <div className="mt-3 flex flex-col gap-3">
          <ul
            className="flex flex-col gap-1.5 text-[13px]"
            aria-label={t('alimentacao-mesmo-produto')}
          >
            {rows.map((m) => (
              <li key={`${m.a_id}-${m.b_id}`} className="flex items-center justify-between gap-3">
                <span className="min-w-0">
                  {t('alimentacao-mesmo-produto-par', { a: m.a_name, b: m.b_name })}
                </span>
                <Button
                  size="sm"
                  variant="secondary"
                  aria-label={t('alimentacao-juntar-par', { a: m.a_name, b: m.b_name })}
                  onClick={() => setPair(m)}
                >
                  {t('alimentacao-juntar-botao')}
                </Button>
              </li>
            ))}
          </ul>
          <p className="text-[12px] text-dim">{t('alimentacao-mesmo-produto-dica')}</p>
        </div>
      ) : null}
      <ConfirmDialog
        open={pair != null}
        onOpenChange={(o) => (!o ? setPair(null) : undefined)}
        title={t('alimentacao-juntar-titulo')}
        description={
          pair ? t('alimentacao-juntar-texto', { a: pair.a_name, b: pair.b_name }) : undefined
        }
        confirmLabel={t('alimentacao-juntar-botao')}
        tone="primary"
        pending={merge.isPending}
        error={merge.error}
        onConfirm={() =>
          pair &&
          merge.mutate(
            { from_id: pair.b_id, into_id: pair.a_id },
            {
              onSuccess: () => {
                toast({ tone: 'success', title: t('alimentacao-juntados') });
                setPair(null);
              },
            },
          )
        }
      />
    </InstrumentCard>
  );
}
