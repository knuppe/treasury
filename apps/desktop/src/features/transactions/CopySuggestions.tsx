/**
 * Cópias entre contas: o mesmo lançamento do banco (mesmo identificador) gravado em duas
 * contas, em geral porque o arquivo do cartão foi importado na conta à vista. Contam duas vezes
 * em "sem categoria" e, categorizadas, dobram o gasto no fluxo de caixa. Um aviso acima dos
 * lançamentos abre a revisão; nada sai sem a pessoa confirmar (dá para desfazer pelo histórico).
 */
import { useMemo, useState } from 'react';

import type { CrossAccountDuplicate } from '@/api/generated/CrossAccountDuplicate';
import { useCrossAccountDuplicates, useRemoveCopies } from '@/api/hooks/transactions';
import { useUndoToast } from '@/api/hooks/undoToast';
import { Button, Dialog, Icon, Money, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

import { Checkbox, InlineError } from './ui/controls';

function Side({ label, v }: { label: string; v: CrossAccountDuplicate['keep'] }) {
  return (
    <span className="truncate" title={v.description}>
      {t('lancamentos-copia-lado', {
        lado: label,
        conta: v.account_name,
        data: formatDate(v.date),
        descricao: v.description || '—',
      })}
      {v.category_path ? ` (${v.category_path})` : ''}
    </span>
  );
}

function CopyRow({
  d,
  checked,
  onToggle,
}: {
  d: CrossAccountDuplicate;
  checked: boolean;
  onToggle: (on: boolean) => void;
}) {
  return (
    <li
      className={cx(
        'grid grid-cols-[auto_minmax(0,1fr)] items-start gap-3 border-b border-line/60 py-2.5 last:border-b-0',
        !checked && 'opacity-70',
      )}
    >
      <Checkbox
        hideLabel
        label={t('lancamentos-copia-rotulo', {
          conta: d.copy.account_name,
          data: formatDate(d.copy.date),
          descricao: d.copy.description || '—',
        })}
        checked={checked}
        onChange={onToggle}
      />
      <div className="flex min-w-0 flex-col gap-1">
        <div className="flex flex-wrap items-baseline gap-x-2 gap-y-0.5 text-[13.5px] text-fg">
          <span className="font-medium">{d.copy.description || '—'}</span>
          <span className="text-dim">{'·'}</span>
          <Money cents={d.copy.amount} currency={d.copy.currency} />
        </div>
        <div className="grid gap-x-4 gap-y-0.5 text-[12px] text-dim @min-[620px]:grid-cols-2">
          <Side label={t('lancamentos-copia-apagar')} v={d.copy} />
          <Side label={t('lancamentos-copia-fica')} v={d.keep} />
        </div>
        {d.copy_is_transfer_leg ? (
          <span className="text-[11.5px] text-dim">{t('lancamentos-copia-transferencia')}</span>
        ) : null}
      </div>
    </li>
  );
}

/** Aviso com a contagem; abre a revisão. Some quando não há cópias. */
export function CopySuggestions({ className }: { className?: string }) {
  const q = useCrossAccountDuplicates();
  const list = useMemo(() => q.data ?? [], [q.data]);
  const [open, setOpen] = useState(false);
  if (list.length === 0) return null;
  return (
    <>
      <div
        role="status"
        className={cx(
          'flex flex-wrap items-center gap-3 rounded-card border border-warning/50 bg-warning/5 px-3 py-2 text-[13px]',
          className,
        )}
      >
        <Icon name="alert" size={14} className="text-warning" />
        <span className="min-w-0 flex-1 text-fg">
          {t('lancamentos-copias-aviso', { n: list.length })}
          <span className="text-dim"> {t('lancamentos-copias-aviso-nota')}</span>
        </span>
        <Button size="sm" variant="primary" onClick={() => setOpen(true)}>
          {t('lancamentos-revisar')}
        </Button>
      </div>
      <CopyReview open={open} onOpenChange={setOpen} copies={list} />
    </>
  );
}

export function CopyReview({
  open,
  onOpenChange,
  copies,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  copies: readonly CrossAccountDuplicate[];
}) {
  // Todas marcadas por padrão; o usuário desmarca o que não for cópia.
  const [picked, setPicked] = useState<Set<string> | null>(null);
  const remove = useRemoveCopies();
  const undoToast = useUndoToast();
  const selected = picked ?? new Set(copies.map((d) => d.copy.id));
  const chosen = copies.filter((d) => selected.has(d.copy.id));
  const toggle = (d: CrossAccountDuplicate, on: boolean) => {
    const next = new Set(selected);
    if (on) next.add(d.copy.id);
    else next.delete(d.copy.id);
    setPicked(next);
  };
  return (
    <Dialog
      open={open}
      onOpenChange={(o) => {
        if (!o) setPicked(null);
        onOpenChange(o);
      }}
      index="03"
      size="xl"
      resizable
      resizeId="copy-review"
      title={t('lancamentos-copias-titulo')}
      description={t('lancamentos-copias-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t('lancamentos-agora-nao')}
          </Button>
          <Button
            variant="danger"
            loading={remove.isPending}
            disabled={chosen.length === 0}
            onClick={() =>
              remove.mutate(
                { ids: chosen.map((d) => d.copy.id) },
                {
                  onError: (e) =>
                    toast({
                      tone: 'error',
                      title: t('lancamentos-copias-erro'),
                      description: e instanceof Error ? e.message : String(e),
                    }),
                  onSuccess: (r) => {
                    undoToast({
                      tone: 'success',
                      title: t('lancamentos-n-copias-apagadas', { n: r.removed }),
                      ...(r.balance_syncs_adjusted > 0
                        ? {
                            description: t('lancamentos-copias-saldo-refeito', {
                              n: r.balance_syncs_adjusted,
                            }),
                          }
                        : {}),
                      changeSetId: r.change_set_id,
                    });
                    setPicked(null);
                    onOpenChange(false);
                  },
                },
              )
            }
          >
            {t('lancamentos-apagar-n-copias', { n: chosen.length })}
          </Button>
        </>
      }
    >
      {/* O erro no topo: com centenas de linhas, embaixo ninguém vê (parecia que nada acontecia). */}
      <InlineError error={remove.error} />
      <ul aria-label={t('lancamentos-copias-lista')} className="flex flex-col">
        {copies.map((d) => (
          <CopyRow
            key={d.copy.id}
            d={d}
            checked={selected.has(d.copy.id)}
            onToggle={(on) => toggle(d, on)}
          />
        ))}
      </ul>
    </Dialog>
  );
}
