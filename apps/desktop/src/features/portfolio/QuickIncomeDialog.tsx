/**
 * Proventos em grade: a família digita (ou cola de uma planilha) só a data do pagamento, o
 * ticker e o valor recebido; o núcleo (`portfolio.quick_income_preview`) acha o provento
 * anunciado, a corretora, o tipo, a data-com e o IR, e a gravação registra as linhas prontas
 * numa alteração só. Data em branco repete a da linha de cima. A data ganha as barras enquanto
 * se digita só os números, e o ticker sugere os ativos cadastrados.
 */
import {
  useDeferredValue,
  useId,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type ClipboardEvent,
} from 'react';
import { createPortal } from 'react-dom';

import { useAssets } from '@/api/hooks/assets';
import type { QuickIncomeLine } from '@/api/generated/QuickIncomeLine';
import type { QuickIncomeRow } from '@/api/generated/QuickIncomeRow';
import {
  useInvestmentAccounts,
  useQuickIncome,
  useQuickIncomePreview,
} from '@/api/hooks/portfolio';
import { Button, Dialog, DialogClose, Icon, Money, Select, toast } from '@/design/components';
import { cx } from '@/design/cx';
import { slot, withSlots } from '@/features/transactions/ui/slots';
import { t as tr } from '@/i18n';
import { formatDate, formatLocale, parseMoney, todayIso } from '@/lib/format';
import { useBaseCurrency } from '@/state/currency';
import { FormError } from '@/vault/FormError';

import { Segmented } from './components/Segmented';

import { maskAmount, typeAmount } from './incomeMasks';
import { INCOME_KIND_LABEL } from './labels';
import { PendingIncomeConfirm, usePendingToConfirm } from './PendingIncomeConfirm';

export { maskAmount, typeAmount };

interface Draft {
  key: number;
  date: string;
  ticker: string;
  amount: string;
  account_id?: string;
}

/**
 * `15/10/2026`, `15/10/26`, `15/10` (ano de `base`), `08` (dia no mês e ano de `base`),
 * `15.10.2026` ou `2026-10-15`. `base` é a data de referência (a da linha de cima; sem ela,
 * hoje). Em inglês o mês vem antes do dia (`10/15/2026`), como em `formatDate`.
 */
export function parseLooseDate(text: string, base = todayIso()): string | null {
  const t = text.trim();
  if (!t) return null;
  const iso = /^(\d{4})-(\d{1,2})-(\d{1,2})$/.exec(t);
  const br = /^(\d{1,2})[/.-](\d{1,2})(?:[/.-](\d{2}|\d{4}))?$/.exec(t);
  const dayOnly = /^(\d{1,2})$/.exec(t);
  let y: number;
  let m: number;
  let d: number;
  if (dayOnly) {
    y = Number(base.slice(0, 4));
    m = Number(base.slice(5, 7));
    d = Number(dayOnly[1]);
  } else if (iso) {
    [y, m, d] = [Number(iso[1]), Number(iso[2]), Number(iso[3])];
  } else if (br) {
    const monthFirst = formatLocale() === 'en';
    d = Number(monthFirst ? br[2] : br[1]);
    m = Number(monthFirst ? br[1] : br[2]);
    const yy = br[3];
    y = yy == null ? Number(base.slice(0, 4)) : yy.length === 2 ? 2000 + Number(yy) : Number(yy);
  } else {
    return null;
  }
  // Ano fora do razoável (ex.: `07/10/0720`, de um dígito digitado no meio) não é data.
  if (y < 1900 || y > 2100) return null;
  const dt = new Date(Date.UTC(y, m - 1, d));
  if (dt.getUTCFullYear() !== y || dt.getUTCMonth() !== m - 1 || dt.getUTCDate() !== d) {
    return null;
  }
  return `${y}-${String(m).padStart(2, '0')}-${String(d).padStart(2, '0')}`;
}

/**
 * Barras da data enquanto se digita só números: `15` → `15`, `151` → `15/1`,
 * `15102026` → `15/10/2026`. Texto com outro separador (`2026-10-15`, `15.10.26`) fica como está.
 */
export function maskDate(text: string): string {
  if (!/^[\d/]*$/.test(text)) return text;
  const d = text.replace(/\D/g, '').slice(0, 8);
  return [d.slice(0, 2), d.slice(2, 4), d.slice(4, 8)].filter(Boolean).join('/');
}

const TICKER = /^[A-Za-z][A-Za-z0-9.]{1,11}$/;

const fold = (s: string) => s.normalize('NFD').replace(/\p{M}/gu, '').toUpperCase();

/** Altura máxima da lista de sugestões (px). */
const LIST_MAX_HEIGHT = 256;

interface TickerOption {
  ticker: string;
  name: string;
  held: boolean;
}

/** Ticker com sugestões dos ativos cadastrados (em carteira primeiro). */
function TickerInput({
  value,
  options,
  row,
  className,
  onChange,
  onPaste,
  onPicked,
}: {
  value: string;
  options: readonly TickerOption[];
  row: number;
  className: string;
  onChange: (ticker: string) => void;
  onPaste: (e: ClipboardEvent<HTMLInputElement>) => void;
  onPicked: () => void;
}) {
  const listId = useId();
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const q = fold(value.trim());
  const matches = useMemo(() => {
    if (!q) return options.slice(0, 8);
    const starts = options.filter((o) => o.ticker.startsWith(q));
    // Pelo nome só a partir de 3 letras (com 2, "IT" acharia "Bitcoin").
    const rest =
      q.length >= 3
        ? options.filter((o) => !o.ticker.startsWith(q) && fold(o.name).includes(q))
        : [];
    return [...starts, ...rest].slice(0, 8);
  }, [q, options]);
  const show = open && matches.length > 0 && !(matches.length === 1 && matches[0]!.ticker === q);
  // A lista flutua por cima do diálogo (portal com posição fixa), abaixo do campo ou, sem
  // espaço embaixo, acima dele; acompanha a rolagem.
  const inputRef = useRef<HTMLInputElement>(null);
  const [place, setPlace] = useState<{ left: number; top?: number; bottom?: number } | null>(null);
  useLayoutEffect(() => {
    if (!show) return;
    const update = () => {
      const r = inputRef.current?.getBoundingClientRect();
      if (!r) return;
      const below = window.innerHeight - r.bottom;
      const up = below < LIST_MAX_HEIGHT + 8 && r.top > below;
      setPlace(
        up
          ? { left: r.left, bottom: window.innerHeight - r.top + 4 }
          : { left: r.left, top: r.bottom + 4 },
      );
    };
    update();
    window.addEventListener('scroll', update, true);
    window.addEventListener('resize', update);
    return () => {
      window.removeEventListener('scroll', update, true);
      window.removeEventListener('resize', update);
    };
  }, [show]);
  const pick = (t: string) => {
    onChange(t);
    setOpen(false);
    onPicked();
  };
  return (
    <div className="relative">
      <input
        ref={inputRef}
        data-cell={`${row}-1`}
        role="combobox"
        aria-label={tr('carteira-grade-ticker-linha', { n: row + 1 })}
        aria-expanded={show}
        aria-controls={listId}
        aria-autocomplete="list"
        aria-activedescendant={show ? `${listId}-${active}` : undefined}
        autoComplete="off"
        spellCheck={false}
        className={cx(className, 'uppercase')}
        placeholder="ITSA4"
        value={value}
        onFocus={() => setOpen(true)}
        onBlur={() => setOpen(false)}
        onChange={(e) => {
          onChange(e.target.value);
          setOpen(true);
          setActive(0);
        }}
        onPaste={onPaste}
        onKeyDown={(e) => {
          if (!show) return;
          if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
            e.preventDefault();
            const n = matches.length;
            setActive((a) => (a + (e.key === 'ArrowDown' ? 1 : n - 1)) % n);
          } else if (e.key === 'Enter' || (e.key === 'Tab' && q)) {
            if (e.key === 'Enter') e.preventDefault();
            const m = matches[Math.min(active, matches.length - 1)];
            if (m) {
              onChange(m.ticker);
              setOpen(false);
              if (e.key === 'Enter') onPicked();
            }
          }
        }}
      />
      {show && place
        ? createPortal(
            <ul
              id={listId}
              role="listbox"
              aria-label={tr('carteira-grade-ativos-linha', { n: row + 1 })}
              // `pointerEvents`: o diálogo modal desliga o mouse fora dele.
              style={{
                position: 'fixed',
                ...place,
                maxHeight: LIST_MAX_HEIGHT,
                pointerEvents: 'auto',
              }}
              className="z-[60] w-80 overflow-auto rounded-control border border-line-strong bg-surface py-1 shadow-lg"
            >
              {matches.map((o, k) => (
                <li
                  key={o.ticker}
                  id={`${listId}-${k}`}
                  role="option"
                  aria-selected={k === active}
                  className={cx(
                    'flex cursor-pointer items-baseline gap-2 px-2 py-1 text-[12.5px]',
                    k === active ? 'bg-accent-soft text-fg' : 'text-dim hover:bg-surface-2',
                  )}
                  onMouseEnter={() => setActive(k)}
                  onMouseDown={(e) => {
                    // Antes do blur do campo, para a escolha valer.
                    e.preventDefault();
                    pick(o.ticker);
                  }}
                >
                  <span className="num w-16 shrink-0 font-medium text-fg">{o.ticker}</span>
                  <span className="truncate">{o.name}</span>
                  {o.held ? (
                    <span className="ml-auto shrink-0 text-[10.5px]">
                      {tr('carteira-grade-em-carteira')}
                    </span>
                  ) : null}
                </li>
              ))}
            </ul>,
            document.body,
          )
        : null}
    </div>
  );
}

/**
 * Linhas coladas de uma planilha ou extrato (tab, `;` ou espaços): cada célula é
 * reconhecida pelo formato (data, valor, ticker), em qualquer ordem.
 */
export function parsePasted(text: string, today = todayIso()): Omit<Draft, 'key'>[] {
  const out: Omit<Draft, 'key'>[] = [];
  for (const raw of text.split(/\r?\n/)) {
    const cells = raw
      .split(/\t|;|\s{2,}/)
      .map((c) => c.trim())
      .filter(Boolean);
    if (!cells.length) continue;
    let date = '';
    let ticker = '';
    let amount = '';
    for (const c of cells) {
      // Na colagem a data precisa de separador: um número solto é valor, não dia.
      if (!date && /[/.-]/.test(c) && parseLooseDate(c, today)) date = c;
      else if (!ticker && TICKER.test(c) && /[A-Za-z]{3}/.test(c)) ticker = c.toUpperCase();
      else if (!amount && parseMoney(c) != null) amount = c;
    }
    if (ticker || amount) out.push({ date, ticker, amount });
  }
  return out;
}

let seq = 0;
const blank = (): Draft => ({ key: ++seq, date: '', ticker: '', amount: '' });

export function QuickIncomeDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="xl"
      index="06"
      title={tr('carteira-registrar-proventos')}
      description={tr('carteira-grade-texto')}
    >
      {open ? <IncomeModes onDone={() => onOpenChange(false)} /> : null}
    </Dialog>
  );
}

/** "A confirmar" (o que o Treasury já sabe, pronto) primeiro; "Digitar" para o resto. */
function IncomeModes({ onDone }: { onDone: () => void }) {
  const pending = usePendingToConfirm();
  const [mode, setMode] = useState<'confirm' | 'type' | null>(null);
  const current = mode ?? (pending.length ? 'confirm' : 'type');
  return (
    <div className="flex flex-col gap-3">
      <Segmented<'confirm' | 'type'>
        label={tr('carteira-grade-como')}
        value={current}
        onChange={setMode}
        options={[
          { value: 'confirm', label: tr('carteira-grade-a-confirmar', { n: pending.length }) },
          { value: 'type', label: tr('carteira-grade-digitar') },
        ]}
      />
      {current === 'confirm' ? (
        <PendingIncomeConfirm onDone={onDone} />
      ) : (
        <QuickIncomeForm onDone={onDone} />
      )}
    </div>
  );
}

const rowKey = (r: QuickIncomeRow) => `${r.date}|${r.ticker}|${r.amount}|${r.account_id ?? ''}`;

/** Linhas completas (com a data herdada da linha de cima) e o índice de cada uma. */
function toRows(drafts: Draft[]): { rows: QuickIncomeRow[]; index: number[] } {
  const rows: QuickIncomeRow[] = [];
  const index: number[] = [];
  let lastDate: string | null = null;
  drafts.forEach((d, i) => {
    const date =
      parseLooseDate(d.date, lastDate ?? todayIso()) ?? (d.date.trim() ? null : lastDate);
    if (d.date.trim() && date) lastDate = date;
    const amount = parseMoney(d.amount);
    if (!date || !d.ticker.trim() || amount == null || amount <= 0) return;
    rows.push({
      date,
      ticker: d.ticker.trim().toUpperCase(),
      amount,
      ...(d.account_id ? { account_id: d.account_id } : {}),
    });
    index.push(i);
  });
  return { rows, index };
}

function QuickIncomeForm({ onDone }: { onDone: () => void }) {
  const [drafts, setDrafts] = useState<Draft[]>(() => [blank(), blank(), blank()]);
  const accounts = useInvestmentAccounts().data ?? [];
  const save = useQuickIncome();
  const table = useRef<HTMLTableElement>(null);
  const allAssets = useAssets({}).data;
  const heldAssets = useAssets({ position: 'open' }).data;
  const tickers = useMemo<TickerOption[]>(() => {
    const held = new Set((heldAssets ?? []).map((a) => a.id));
    return (allAssets ?? [])
      .filter((a) => a.ticker)
      .map((a) => ({ ticker: a.ticker!.toUpperCase(), name: a.name, held: held.has(a.id) }))
      .sort((a, b) => Number(b.held) - Number(a.held) || a.ticker.localeCompare(b.ticker));
  }, [allAssets, heldAssets]);

  const { rows, index } = useMemo(() => toRows(drafts), [drafts]);
  // Data em vigor acima de cada linha (referência para "08" e para o texto de exemplo).
  const datesAbove = useMemo(() => {
    const out: (string | null)[] = [];
    let last: string | null = null;
    for (const d of drafts) {
      out.push(last);
      last = parseLooseDate(d.date, last ?? todayIso()) ?? last;
    }
    return out;
  }, [drafts]);
  const deferred = useDeferredValue(rows);
  const preview = useQuickIncomePreview({ rows: deferred });
  const stale = deferred !== rows || preview.isFetching;
  // O que já foi deduzido fica guardado pelo conteúdo da linha: enquanto a prévia nova não
  // chega, as linhas mantêm o resumo (sem a grade encolher e o diálogo rolar para o topo).
  const [known, setKnown] = useState(() => ({
    from: undefined as QuickIncomeLine[] | undefined,
    lines: new Map<string, QuickIncomeLine>(),
  }));
  const fresh = preview.isPlaceholderData ? undefined : preview.data;
  if (fresh && fresh !== known.from) {
    // Prévia nova: guarda cada linha pelo conteúdo (ajuste de estado durante a renderização).
    const lines = new Map(known.lines);
    fresh.forEach((l) => {
      const r = deferred[l.row];
      if (r) lines.set(rowKey(r), l);
    });
    setKnown({ from: fresh, lines });
  }
  const lineOf = new Map<number, QuickIncomeLine>();
  rows.forEach((r, k) => {
    const l = known.lines.get(rowKey(r));
    if (l) lineOf.set(index[k]!, l);
  });
  const lines = [...lineOf.values()];
  const ready = lines.filter((l) => l.state === 'ready').length;
  const duplicate = lines.filter((l) => l.state === 'duplicate').length;
  const failed = lines.filter((l) => l.state === 'error').length;

  const patch = (i: number, p: Partial<Draft>) =>
    setDrafts((prev) => {
      const next = prev.map((d, j) => (j === i ? { ...d, ...p } : d));
      const last = next[next.length - 1]!;
      return last.date || last.ticker || last.amount ? [...next, blank()] : next;
    });

  const focusCell = (row: number, col: number) => {
    const find = () =>
      table.current?.querySelector<HTMLInputElement>(`[data-cell="${row}-${col}"]`);
    const el = find();
    if (el) el.focus();
    else requestAnimationFrame(() => find()?.focus());
  };

  const onPaste = (i: number, e: ClipboardEvent<HTMLInputElement>) => {
    const text = e.clipboardData.getData('text');
    if (!/[\t\n;]/.test(text)) return;
    const parsed = parsePasted(text);
    if (!parsed.length) return;
    e.preventDefault();
    setDrafts((prev) => {
      const head = prev.slice(0, i);
      const tail = prev.slice(i).filter((d) => d.date || d.ticker || d.amount);
      return [...head, ...parsed.map((p) => ({ ...p, key: ++seq })), ...tail, blank()];
    });
  };

  const submit = () => {
    if (!ready) return;
    save.mutate(
      { rows },
      {
        onSuccess: (r) => {
          const n = r.report.created;
          toast({
            tone: n ? 'success' : 'info',
            title: n ? tr('carteira-proventos-registrados', { n }) : tr('carteira-grade-nada-novo'),
          });
          if (n) onDone();
        },
      },
    );
  };

  const cell =
    'h-8 w-full rounded-control border border-line bg-bg px-2 text-[13px] outline-none focus:border-accent';

  return (
    <div className="flex flex-col gap-3">
      <div>
        <table
          ref={table}
          className="w-full border-collapse text-[13px]"
          aria-label={tr('carteira-grade-tabela')}
        >
          <thead>
            <tr className="label-caps text-left text-dim">
              <th className="w-[120px] pb-1.5 pr-2 font-normal">{tr('carteira-pagamento')}</th>
              <th className="w-[110px] pb-1.5 pr-2 font-normal">{tr('carteira-grade-ticker')}</th>
              <th className="w-[130px] pb-1.5 pr-2 font-normal">
                {tr('carteira-grade-valor-recebido')}
              </th>
              <th className="pb-1.5 pr-2 font-normal">{tr('carteira-grade-entendeu')}</th>
              <th className="w-8 pb-1.5" />
            </tr>
          </thead>
          <tbody>
            {drafts.map((d, i) => {
              const filled = d.date || d.ticker || d.amount;
              const line = lineOf.get(i);
              const prevDate = datesAbove[i];
              const dateBad =
                d.date.trim() !== '' && !parseLooseDate(d.date, prevDate ?? todayIso());
              const amountBad = d.amount.trim() !== '' && (parseMoney(d.amount) ?? 0) <= 0;
              return (
                <tr key={d.key} className="align-top">
                  <td className="py-1 pr-2">
                    <input
                      data-cell={`${i}-0`}
                      aria-label={tr('carteira-grade-data-linha', { n: i + 1 })}
                      aria-invalid={dateBad || undefined}
                      className={cx(cell, dateBad && 'border-negative')}
                      placeholder={prevDate ? formatDate(prevDate) : tr('carteira-data-formato')}
                      value={d.date}
                      inputMode="numeric"
                      onChange={(e) => patch(i, { date: maskDate(e.target.value) })}
                      onBlur={() => {
                        // Completa o que ficou implícito (dia ou dia/mês) com a data de cima.
                        // Depois do foco ir para o próximo campo: mudar a grade durante o blur
                        // fazia o Tab perder o destino.
                        const iso = parseLooseDate(d.date, prevDate ?? todayIso());
                        if (iso && formatDate(iso) !== d.date) {
                          const full = formatDate(iso);
                          setTimeout(() => patch(i, { date: full }), 0);
                        }
                      }}
                      onPaste={(e) => onPaste(i, e)}
                    />
                  </td>
                  <td className="py-1 pr-2">
                    <TickerInput
                      row={i}
                      value={d.ticker}
                      options={tickers}
                      className={cell}
                      onChange={(t) => patch(i, { ticker: t, account_id: undefined })}
                      onPaste={(e) => onPaste(i, e)}
                      onPicked={() => focusCell(i, 2)}
                    />
                  </td>
                  <td className="py-1 pr-2">
                    <input
                      data-cell={`${i}-2`}
                      aria-label={tr('carteira-grade-valor-linha', { n: i + 1 })}
                      aria-invalid={amountBad || undefined}
                      inputMode="decimal"
                      className={cx(cell, 'num text-right', amountBad && 'border-negative')}
                      placeholder={maskAmount(0)}
                      value={d.amount}
                      onChange={(e) => patch(i, { amount: typeAmount(e.target.value) })}
                      onPaste={(e) => onPaste(i, e)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter') {
                          e.preventDefault();
                          focusCell(i + 1, 0);
                        }
                      }}
                    />
                  </td>
                  <td className="py-1 pr-2">
                    {line ? (
                      <LineSummary
                        line={line}
                        accounts={accounts}
                        onAccount={(id) => patch(i, { account_id: id })}
                      />
                    ) : filled && (dateBad || amountBad || !stale) ? (
                      <span className="text-[12px] text-dim">
                        {dateBad
                          ? tr('carteira-grade-data-invalida')
                          : amountBad
                            ? tr('carteira-valor-invalido')
                            : tr('carteira-grade-complete')}
                      </span>
                    ) : null}
                  </td>
                  <td className="py-1">
                    {filled ? (
                      <button
                        type="button"
                        aria-label={tr('carteira-grade-remover-linha', { n: i + 1 })}
                        className="mt-1.5 text-dim hover:text-fg"
                        onClick={() =>
                          setDrafts((prev) => {
                            const next = prev.filter((_, j) => j !== i);
                            return next.length ? next : [blank()];
                          })
                        }
                      >
                        <Icon name="x" size={13} />
                      </button>
                    ) : null}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      <p className="text-[12px] text-dim">{tr('carteira-grade-ajuda')}</p>
      <FormError error={save.error ?? preview.error} />
      <div className="flex flex-wrap items-center justify-end gap-3 border-t border-line pt-3">
        <span role="status" className="mr-auto text-[12px] text-dim">
          {rows.length
            ? [
                tr('carteira-grade-prontas', { n: ready }),
                duplicate ? tr('carteira-grade-duplicadas', { n: duplicate }) : null,
                failed ? tr('carteira-grade-problema', { n: failed }) : null,
              ]
                .filter(Boolean)
                .join(' · ')
            : tr('carteira-grade-nenhuma-completa')}
        </span>
        <DialogClose asChild>
          <Button variant="ghost">{tr('comum-cancelar')}</Button>
        </DialogClose>
        <Button
          variant="primary"
          disabled={!ready || stale}
          loading={save.isPending}
          onClick={submit}
        >
          {ready ? tr('carteira-registrar-n-proventos', { n: ready }) : tr('carteira-registrar')}
        </Button>
      </div>
    </div>
  );
}

function LineSummary({
  line,
  accounts,
  onAccount,
}: {
  line: QuickIncomeLine;
  accounts: { id: string; name: string }[];
  onAccount: (id: string) => void;
}) {
  const base = useBaseCurrency();
  const cur = line.currency ?? base;
  const tone =
    line.state === 'ready'
      ? line.verified
        ? 'text-positive'
        : 'text-fg'
      : line.state === 'duplicate'
        ? 'text-dim'
        : 'text-negative';
  return (
    <div className="flex flex-col gap-1 text-[12px] leading-snug">
      <div className={cx('flex flex-wrap items-center gap-x-2', tone)}>
        <Icon
          name={
            line.state === 'ready'
              ? line.verified
                ? 'check'
                : 'info'
              : line.state === 'duplicate'
                ? 'copy'
                : 'alert'
          }
          size={12}
        />
        {line.state === 'error' ? (
          <span>{line.error}</span>
        ) : (
          <>
            <span className="text-fg">{line.asset_name}</span>
            {line.kind ? <span>{INCOME_KIND_LABEL[line.kind]}</span> : null}
            {line.gross != null && line.tax_withheld ? (
              <span className="text-dim">
                {withSlots(tr('carteira-bruto-ir', { bruto: slot(0), ir: slot(1) }), [
                  <Money key="b" cents={line.gross} currency={cur} />,
                  <Money key="i" cents={line.tax_withheld} currency={cur} />,
                ])}
              </span>
            ) : null}
            {line.status === 'announced' ? (
              <span className="text-dim">{tr('carteira-anunciado-curto')}</span>
            ) : null}
            {line.state === 'duplicate' ? <span>{line.error}</span> : null}
          </>
        )}
      </div>
      {line.account_id && accounts.length > 1 && line.state !== 'error' ? (
        <div className="w-56">
          <Select
            aria-label={tr('carteira-conta-provento-de', { ativo: line.ticker })}
            value={line.account_id}
            onValueChange={onAccount}
            options={accounts.map((a) => ({ value: a.id, label: a.name }))}
          />
        </div>
      ) : line.account_name ? (
        <span className="text-dim">{line.account_name}</span>
      ) : null}
      {line.state === 'error' && line.needs_account && accounts.length ? (
        <div className="w-56">
          <Select
            aria-label={tr('carteira-conta-provento-de', { ativo: line.ticker })}
            value={undefined}
            placeholder={tr('carteira-escolha-conta')}
            onValueChange={onAccount}
            options={accounts.map((a) => ({ value: a.id, label: a.name }))}
          />
        </div>
      ) : null}
      {line.notes.length ? <span className="text-dim">{line.notes.join(' · ')}</span> : null}
    </div>
  );
}
