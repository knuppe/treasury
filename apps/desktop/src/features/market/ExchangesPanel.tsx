/**
 * Bolsas (Ativos › Bolsas): o catálogo do app (B3, EUA, Europa, Ásia, Oceania, América
 * Latina, África) com país, moeda, sufixo do Yahoo, fuso, pregão e a situação agora; e
 * o cadastro de bolsas que faltam (código, país, moeda, fuso e horário).
 */
import { useEffect, useState, type FormEvent } from 'react';

import type { Exchange } from '@/api/generated/Exchange';
import { useDeleteExchange, useExchanges, useSaveExchange } from '@/api/hooks/market';
import {
  Button,
  Dialog,
  DialogClose,
  EmptyState,
  Field,
  Icon,
  Input,
  TechTable,
  toast,
  type TechColumn,
} from '@/design/components';
import { cx } from '@/design/cx';
import { localTime, SESSION_LABEL, sessionAt } from '@/lib/marketHours';
import { t } from '@/i18n';
import { FormError } from '@/vault/FormError';

import { ConfirmButton, ErrorBlock, LoadingBlock } from '../portfolio/components/States';

/** Relógio de 1 minuto para a situação do pregão. */
function useNow(ms = 60_000): Date {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), ms);
    return () => clearInterval(timer);
  }, [ms]);
  return now;
}

function daysLabel(d: string): string {
  if (d === '12345') return t('mercado-dias-seg-sex');
  if (d === '71234') return t('mercado-dias-dom-qui');
  // 1 = segunda … 7 = domingo.
  const days = ['', ...t('mercado-dias-curtos').split(',')];
  return d
    .split('')
    .map((c) => days[Number(c)] ?? c)
    .join(', ');
}

export function hoursLabel(e: Exchange): string {
  const lunch =
    e.lunch_start && e.lunch_end
      ? t('mercado-almoco', { inicio: e.lunch_start, fim: e.lunch_end })
      : '';
  return `${e.open_time}–${e.close_time}${lunch}, ${daysLabel(e.trading_days)}`;
}

export function ExchangesPanel() {
  const q = useExchanges();
  const del = useDeleteExchange();
  const now = useNow();
  const [editing, setEditing] = useState<Exchange | null>(null);
  const [adding, setAdding] = useState(false);
  const [text, setText] = useState('');
  const rows = (q.data ?? []).filter((e) => {
    const needle = text.trim().toLowerCase();
    return (
      !needle ||
      [e.code, e.name, e.country, e.currency, e.yahoo_suffix, ...e.aliases].some((v) =>
        v.toLowerCase().includes(needle),
      )
    );
  });
  const columns: TechColumn<Exchange>[] = [
    {
      key: 'code',
      header: t('mercado-bolsa'),
      width: '26%',
      cell: (e) => (
        <span className="flex flex-col leading-tight">
          <span className="font-medium text-fg">{e.code}</span>
          <span className="truncate font-sans text-[11.5px] text-dim">{e.name}</span>
        </span>
      ),
      sortValue: (e) => e.code,
    },
    {
      key: 'country',
      header: t('mercado-pais'),
      cell: (e) => e.country,
      sortValue: (e) => e.country,
    },
    {
      key: 'currency',
      header: t('mercado-moeda'),
      cell: (e) => (
        <span className="inline-flex items-center gap-1">
          {e.currency}
          {!e.fx_automatic ? (
            <span className="text-[11px] text-warning" title={t('mercado-cambio-manual-dica')}>
              {t('mercado-cambio-manual')}
            </span>
          ) : null}
        </span>
      ),
    },
    {
      key: 'suffix',
      header: 'Yahoo',
      cell: (e) =>
        e.yahoo_suffix ? (
          <span className="num text-dim">{e.yahoo_suffix}</span>
        ) : (
          <span
            className="font-sans text-[12px] whitespace-nowrap text-dim"
            title={t('mercado-sem-sufixo-dica')}
          >
            {t('mercado-sem-sufixo')}
          </span>
        ),
    },
    {
      key: 'hours',
      header: t('mercado-pregao-local'),
      cell: (e) => (
        <span className="flex flex-col leading-tight">
          <span className="num">{hoursLabel(e)}</span>
          <span className="font-sans text-[11px] text-dim">{e.timezone}</span>
        </span>
      ),
    },
    {
      key: 'now',
      header: t('mercado-agora-col'),
      cell: (e) => {
        const s = sessionAt(e, now);
        return (
          <span className="flex flex-col leading-tight whitespace-nowrap">
            <span
              className={cx('font-sans text-[12px]', s === 'open' ? 'text-positive' : 'text-dim')}
            >
              {SESSION_LABEL[s]}
            </span>
            <span className="num text-[11px] text-dim">
              {t('mercado-hora-local', { hora: localTime(e.timezone, now) })}
            </span>
          </span>
        );
      },
      sortValue: (e) => (sessionAt(e, now) === 'open' ? 0 : 1),
    },
    {
      key: 'actions',
      header: <span className="sr-only">{t('ativos-acoes')}</span>,
      align: 'right',
      cell: (e) => (
        <span className="flex justify-end gap-1" onKeyDown={(ev) => ev.stopPropagation()}>
          <Button size="sm" variant="ghost" onClick={() => setEditing(e)}>
            {t('comum-editar')}
          </Button>
          {!e.builtin ? (
            <ConfirmButton
              label={t('ativos-remover')}
              confirmLabel={t('ativos-remover')}
              title={t('mercado-remover-bolsa')}
              description={t('mercado-remover-bolsa-descricao', { codigo: e.code })}
              pending={del.isPending}
              onConfirm={() =>
                del
                  .mutateAsync({ code: e.code })
                  .then(() => toast({ tone: 'success', title: t('mercado-bolsa-removida') }))
              }
            />
          ) : null}
        </span>
      ),
    },
  ];
  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-3">
        <Input
          aria-label={t('mercado-filtrar-bolsas')}
          placeholder={t('mercado-filtrar-placeholder')}
          value={text}
          onChange={(e) => setText(e.target.value)}
          wrapperClassName="max-w-72"
        />
        <span className="text-[12.5px] text-dim">
          {q.data
            ? t('mercado-resumo', {
                n: q.data.length,
                abertas: q.data.filter((e) => sessionAt(e, now) === 'open').length,
              })
            : ''}
        </span>
        <Button
          className="ml-auto"
          variant="primary"
          icon={<Icon name="plus" size={12} />}
          onClick={() => setAdding(true)}
        >
          {t('mercado-cadastrar-bolsa')}
        </Button>
      </div>
      <p className="text-[12px] text-dim">{t('mercado-bolsas-ajuda')}</p>
      {q.isPending ? (
        <LoadingBlock />
      ) : q.isError ? (
        <ErrorBlock error={q.error} onRetry={() => void q.refetch()} />
      ) : rows.length === 0 ? (
        <EmptyState title={t('mercado-nenhuma-bolsa')} description={t('mercado-nada-filtro')} />
      ) : (
        <div className="rounded-card border border-line bg-surface">
          <TechTable
            label={t('ativos-aba-bolsas')}
            prefsId="Bolsas"
            rows={rows}
            columns={columns}
            getRowId={(e) => e.code}
          />
        </div>
      )}
      <ExchangeDialog open={adding} onOpenChange={setAdding} />
      <ExchangeDialog
        open={editing != null}
        onOpenChange={(o) => !o && setEditing(null)}
        exchange={editing}
      />
    </div>
  );
}

export function ExchangeDialog({
  open,
  onOpenChange,
  exchange,
}: {
  open: boolean;
  onOpenChange: (o: boolean) => void;
  exchange?: Exchange | null;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      index="07"
      title={
        exchange ? t('ativos-form-editar', { nome: exchange.code }) : t('mercado-cadastrar-bolsa')
      }
      description={t('mercado-dialogo-descricao')}
    >
      {open ? (
        <ExchangeForm exchange={exchange ?? null} onDone={() => onOpenChange(false)} />
      ) : null}
    </Dialog>
  );
}

function ExchangeForm({ exchange, onDone }: { exchange: Exchange | null; onDone: () => void }) {
  const save = useSaveExchange();
  const [f, setF] = useState({
    code: exchange?.code ?? '',
    name: exchange?.name ?? '',
    country: exchange?.country ?? '',
    currency: exchange?.currency ?? '',
    yahoo_suffix: exchange?.yahoo_suffix ?? '',
    mic: exchange?.mic ?? '',
    timezone: exchange?.timezone ?? '',
    open_time: exchange?.open_time ?? '09:00',
    close_time: exchange?.close_time ?? '17:00',
    lunch_start: exchange?.lunch_start ?? '',
    lunch_end: exchange?.lunch_end ?? '',
    trading_days: exchange?.trading_days ?? '12345',
    aliases: exchange?.aliases.join(', ') ?? '',
    notes: exchange?.notes ?? '',
  });
  const [submitted, setSubmitted] = useState(false);
  const set = (k: keyof typeof f, v: string) => setF((s) => ({ ...s, [k]: v }));
  const errors = {
    code: !/^[A-Za-z0-9_]{1,12}$/.test(f.code.trim()) ? t('mercado-erro-codigo') : null,
    name: !f.name.trim() ? t('mercado-erro-nome') : null,
    country: !/^[A-Za-z]{2}$/.test(f.country.trim()) ? t('mercado-erro-pais') : null,
    currency: !/^[A-Za-z]{3}$/.test(f.currency.trim()) ? t('mercado-erro-moeda') : null,
    timezone: !f.timezone.trim() ? t('mercado-erro-fuso') : null,
  };
  const invalid = Object.values(errors).some(Boolean);
  const show = (e: string | null) => (submitted ? e : null) ?? undefined;
  const submit = (e: FormEvent) => {
    e.preventDefault();
    setSubmitted(true);
    if (invalid) return;
    save.mutate(
      {
        code: f.code.trim(),
        name: f.name.trim(),
        country: f.country.trim(),
        currency: f.currency.trim(),
        yahoo_suffix: f.yahoo_suffix.trim(),
        mic: f.mic.trim() || undefined,
        timezone: f.timezone.trim(),
        open_time: f.open_time.trim(),
        close_time: f.close_time.trim(),
        lunch_start: f.lunch_start.trim() || undefined,
        lunch_end: f.lunch_end.trim() || undefined,
        trading_days: f.trading_days.trim() || undefined,
        aliases: f.aliases
          .split(',')
          .map((a) => a.trim())
          .filter(Boolean),
        notes: f.notes.trim() || undefined,
      },
      {
        onSuccess: (x) => {
          toast({
            tone: 'success',
            title: exchange ? t('mercado-bolsa-atualizada') : t('mercado-bolsa-cadastrada'),
            description: x.code,
          });
          onDone();
        },
      },
    );
  };
  return (
    <form
      onSubmit={submit}
      noValidate
      aria-label={t('mercado-bolsa')}
      className="flex flex-col gap-4"
    >
      <div className="grid gap-4 sm:grid-cols-3">
        <Field
          label={t('ativos-codigo')}
          error={show(errors.code)}
          hint={exchange ? t('ativos-form-so-leitura') : t('ativos-exemplo', { exemplo: 'PSE' })}
        >
          <Input
            value={f.code}
            onChange={(e) => set('code', e.target.value.toUpperCase())}
            className="num uppercase"
            disabled={exchange != null}
          />
        </Field>
        <Field label={t('ativos-col-nome')} error={show(errors.name)} className="sm:col-span-2">
          <Input value={f.name} onChange={(e) => set('name', e.target.value)} />
        </Field>
        <Field label={t('mercado-pais')} error={show(errors.country)} hint={t('mercado-iso-2')}>
          <Input
            value={f.country}
            maxLength={2}
            onChange={(e) => set('country', e.target.value.toUpperCase())}
            className="num uppercase"
          />
        </Field>
        <Field label={t('mercado-moeda')} error={show(errors.currency)} hint={t('mercado-iso-3')}>
          <Input
            value={f.currency}
            maxLength={3}
            onChange={(e) => set('currency', e.target.value.toUpperCase())}
            className="num uppercase"
          />
        </Field>
        <Field label={t('mercado-sufixo')} hint={t('mercado-sufixo-dica')}>
          <Input
            value={f.yahoo_suffix}
            onChange={(e) => set('yahoo_suffix', e.target.value.toUpperCase())}
            className="num uppercase"
          />
        </Field>
        <Field label={t('mercado-fuso')} error={show(errors.timezone)} className="sm:col-span-2">
          <Input
            value={f.timezone}
            onChange={(e) => set('timezone', e.target.value)}
            placeholder="Asia/Manila"
          />
        </Field>
        <Field label="MIC" hint={t('mercado-mic-dica')}>
          <Input
            value={f.mic}
            onChange={(e) => set('mic', e.target.value.toUpperCase())}
            className="num"
          />
        </Field>
        <Field label={t('mercado-abertura')}>
          <Input
            value={f.open_time}
            onChange={(e) => set('open_time', e.target.value)}
            className="num"
          />
        </Field>
        <Field label={t('ativos-fechamento')}>
          <Input
            value={f.close_time}
            onChange={(e) => set('close_time', e.target.value)}
            className="num"
          />
        </Field>
        <Field label={t('mercado-dias-pregao')} hint={t('mercado-dias-pregao-dica')}>
          <Input
            value={f.trading_days}
            onChange={(e) => set('trading_days', e.target.value.replace(/[^1-7]/g, ''))}
            className="num"
          />
        </Field>
        <Field label={t('mercado-almoco-inicio')} hint={t('ativos-opcional-ponto')}>
          <Input
            value={f.lunch_start}
            onChange={(e) => set('lunch_start', e.target.value)}
            className="num"
          />
        </Field>
        <Field label={t('mercado-almoco-fim')}>
          <Input
            value={f.lunch_end}
            onChange={(e) => set('lunch_end', e.target.value)}
            className="num"
          />
        </Field>
        <Field label={t('mercado-apelidos')} hint={t('mercado-apelidos-dica')}>
          <Input value={f.aliases} onChange={(e) => set('aliases', e.target.value)} />
        </Field>
        <Field label={t('ativos-form-observacoes')} className="sm:col-span-3">
          <Input value={f.notes} onChange={(e) => set('notes', e.target.value)} />
        </Field>
      </div>
      <FormError error={save.error} />
      <div className="flex justify-end gap-2 border-t border-line pt-3">
        <DialogClose asChild>
          <Button variant="ghost">{t('comum-cancelar')}</Button>
        </DialogClose>
        <Button type="submit" variant="primary" loading={save.isPending}>
          {t('mercado-salvar-bolsa')}
        </Button>
      </div>
    </form>
  );
}
