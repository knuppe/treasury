/**
 * Boleto, Pix "copia e cola" ou comprovante lido no computador (docs/ocr-other-documents.md), na
 * janela de importação, antes de qualquer IA:
 *
 * - **boleto ou Pix** → conta a pagar prevista (valor, vencimento, conta, categoria), que entra
 *   nas contas a vencer e no lembrete; a linha digitável fica para copiar no app do banco;
 * - **comprovante** → liga ao lançamento do extrato que combina (ou cria um), com quem recebeu e
 *   os dados lidos na nota; quita o previsto que ele paga. A imagem não é guardada.
 *
 * Nada é pago pelo app. O que não for reconhecido fica com o agente, se a pessoa quiser.
 */
import { useEffect, useMemo, useState } from 'react';

import type { PaymentReading } from '@/api/generated/PaymentReading';
import { useAccounts } from '@/api/hooks/accounts';
import { usePlanBill, useReadPayment, useSaveProof } from '@/api/hooks/payments';
import {
  Button,
  DateInput,
  Dialog,
  Field,
  Input,
  KeyValue,
  Money,
  MoneyInput,
  Select,
  toast,
} from '@/design/components';
import { CategoryPicker } from '@/features/transactions/ui/CategoryPicker';
import { Checkbox, InlineError } from '@/features/transactions/ui/controls';
import { photoArgs, readPaymentCodes } from '@/features/receipts/photo';
import { t } from '@/i18n';
import { formatDate } from '@/lib/format';

export function PaymentDialog({
  file,
  onClose,
  onDone,
  onAgent,
}: {
  file: File;
  onClose: () => void;
  /** Gravou (o arquivo pode sair da lista e da pasta Downloads). */
  onDone: () => void;
  /** Não reconheceu: mandar para o agente (só com motor configurado). */
  onAgent?: () => void;
}) {
  const read = useReadPayment();
  const [reading, setReading] = useState<PaymentReading | null>(null);

  useEffect(() => {
    let alive = true;
    void (async () => {
      const [args, codes] = await Promise.all([photoArgs(file), readPaymentCodes(file)]);
      read.mutate({ ...args, codes }, { onSuccess: (r) => alive && setReading(r) });
    })();
    return () => {
      alive = false;
    };
    // Uma leitura por arquivo.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [file]);

  const title =
    reading?.kind === 'bill'
      ? t('importacao-pagamento-titulo-boleto')
      : reading?.kind === 'pix'
        ? t('importacao-pagamento-titulo-pix')
        : reading?.kind === 'proof'
          ? t('importacao-pagamento-titulo-comprovante')
          : t('importacao-pagamento-titulo');

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      size="md"
      title={title}
      description={file.name}
    >
      {read.error ? <InlineError error={read.error} /> : null}
      {!reading && !read.error ? (
        <p className="text-[13px] text-dim" role="status">
          {t('importacao-pagamento-lendo')}
        </p>
      ) : null}
      {reading && (reading.kind === 'bill' || reading.kind === 'pix') ? (
        <BillForm reading={reading} onDone={onDone} />
      ) : null}
      {reading?.kind === 'proof' ? <ProofForm reading={reading} onDone={onDone} /> : null}
      {reading?.kind === 'none' ? (
        <div className="flex flex-col gap-3">
          <p className="text-[13px] text-fg">{t('importacao-pagamento-nada')}</p>
          {onAgent ? (
            <Button variant="secondary" onClick={onAgent}>
              {t('importacao-pagamento-pedir-agente')}
            </Button>
          ) : null}
        </div>
      ) : null}
    </Dialog>
  );
}

function Warnings({ list }: { list: string[] }) {
  if (!list.length) return null;
  return (
    <ul className="flex flex-col gap-1 text-[12.5px] text-warning">
      {list.map((w) => (
        <li key={w}>{w}</li>
      ))}
    </ul>
  );
}

function useAccountOptions() {
  const { data: accounts = [] } = useAccounts();
  const usable = accounts.filter((a) => !a.archived);
  return {
    accounts: usable,
    options: usable.map((a) => ({ value: a.id, label: a.name, hint: a.currency })),
  };
}

function CopyLine({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <div className="flex flex-col gap-1">
      <span className="label-caps text-dim">{t('importacao-pagamento-linha')}</span>
      <div className="flex items-start gap-2">
        <code className="num min-w-0 flex-1 rounded-control border border-line bg-bg px-2 py-1.5 text-[12px] break-all text-fg">
          {text}
        </code>
        <Button
          size="sm"
          variant="ghost"
          onClick={() =>
            void navigator.clipboard?.writeText(text).then(() => {
              setCopied(true);
              setTimeout(() => setCopied(false), 1500);
            })
          }
        >
          {copied ? t('importacao-pagamento-copiado') : t('importacao-pagamento-copiar')}
        </Button>
      </div>
    </div>
  );
}

function BillForm({ reading, onDone }: { reading: PaymentReading; onDone: () => void }) {
  const plan = usePlanBill();
  const { accounts, options } = useAccountOptions();
  const bill = reading.bill ?? null;
  const pix = reading.pix ?? null;
  const [description, setDescription] = useState(reading.description);
  const [amount, setAmount] = useState<number | null>(bill?.amount ?? pix?.amount ?? null);
  const [due, setDue] = useState<string | null>(bill?.due_date ?? null);
  const [accountId, setAccountId] = useState('');
  const [category, setCategory] = useState<string | null>(reading.category_id ?? null);
  const account = accounts.find((a) => a.id === accountId);
  const existing = reading.existing ?? null;
  const ready = Boolean(accountId && amount && amount > 0 && due && description.trim());
  const code = bill ? bill.barcode : (pix?.payload ?? '');

  return (
    <div className="flex flex-col gap-3">
      {existing ? (
        <p className="text-[13px] text-fg" role="status">
          {t('importacao-pagamento-ja-lancado', {
            data: formatDate(existing.date),
            descricao: existing.description,
          })}
        </p>
      ) : null}
      <KeyValue
        items={[
          ...(bill?.bank_name
            ? [{ label: t('importacao-pagamento-banco'), value: bill.bank_name }]
            : []),
          ...(bill?.beneficiary
            ? [{ label: t('importacao-pagamento-beneficiario'), value: bill.beneficiary }]
            : []),
          ...(pix?.name ? [{ label: t('importacao-pagamento-recebedor'), value: pix.name }] : []),
          ...(pix?.key ? [{ label: t('importacao-pagamento-chave'), value: pix.key }] : []),
        ]}
      />
      {bill ? <CopyLine text={bill.line} /> : pix ? <CopyLine text={pix.payload} /> : null}
      <div className="grid gap-3 @min-[480px]:grid-cols-2">
        <Field label={t('importacao-pagamento-descricao')} className="@min-[480px]:col-span-2">
          <Input value={description} onChange={(e) => setDescription(e.target.value)} />
        </Field>
        <Field label={t('importacao-pagamento-valor')}>
          <MoneyInput
            currency={account?.currency ?? 'BRL'}
            value={amount}
            onValueChange={setAmount}
          />
        </Field>
        <Field label={t('importacao-pagamento-vencimento')}>
          <DateInput value={due} onValueChange={setDue} />
        </Field>
        <Field label={t('importacao-pagamento-conta')}>
          <Select
            value={accountId || undefined}
            onValueChange={setAccountId}
            options={options}
            placeholder={t('importacao-pagamento-escolha-conta')}
          />
        </Field>
        <Field label={t('importacao-pagamento-categoria')}>
          <CategoryPicker value={category} onChange={setCategory} kind="expense" />
        </Field>
      </div>
      <Warnings list={reading.warnings} />
      <p className="text-[12px] text-dim">{t('importacao-pagamento-previsto-dica')}</p>
      <InlineError error={plan.error} />
      <div className="flex justify-end">
        <Button
          variant="primary"
          disabled={!ready || Boolean(existing)}
          loading={plan.isPending}
          onClick={() =>
            plan.mutate(
              {
                account_id: accountId,
                description: description.trim(),
                amount: amount ?? 0,
                due_date: due ?? '',
                ...(category ? { category_id: category } : {}),
                code,
                ...(pix && !bill ? { pix: true } : {}),
                ...(reading.sha256 ? { sha256: reading.sha256 } : {}),
              },
              {
                onSuccess: () => {
                  toast({
                    tone: 'success',
                    title: t('importacao-pagamento-previsto-criado'),
                    description: description.trim(),
                  });
                  onDone();
                },
              },
            )
          }
        >
          {t('importacao-pagamento-criar-previsto')}
        </Button>
      </div>
    </div>
  );
}

const PROOF_KIND = {
  pix: 'importacao-pagamento-tipo-pix',
  transfer: 'importacao-pagamento-tipo-transferencia',
  bill: 'importacao-pagamento-tipo-boleto',
  other: 'importacao-pagamento-tipo-outro',
} as const;

function ProofForm({ reading, onDone }: { reading: PaymentReading; onDone: () => void }) {
  const save = useSaveProof();
  const { accounts, options } = useAccountOptions();
  const proof = reading.proof!;
  const [choice, setChoice] = useState<string>(reading.matches[0]?.txn_id ?? 'new');
  const [closePlanned, setClosePlanned] = useState<string | null>(
    reading.planned[0]?.txn_id ?? null,
  );
  const [payee, setPayee] = useState(proof.counterpart ?? '');
  const [date, setDate] = useState<string | null>(proof.date ?? null);
  const [amount, setAmount] = useState<number | null>(proof.amount ?? null);
  const [accountId, setAccountId] = useState('');
  const [description, setDescription] = useState(reading.description);
  const [category, setCategory] = useState<string | null>(null);
  const account = accounts.find((a) => a.id === accountId);
  const linked = reading.linked ?? null;
  // Sem lançamento do extrato, o previsto que ele paga vira o pagamento.
  const target = useMemo(
    () => (choice === 'new' ? (closePlanned ? `planned:${closePlanned}` : 'new') : choice),
    [choice, closePlanned],
  );
  const creating = target === 'new';
  const ready = Boolean(date) && (!creating || Boolean(accountId && amount && description.trim()));
  const sign = proof.direction === 'in' ? 1 : -1;

  return (
    <div className="flex flex-col gap-3">
      {linked ? (
        <p className="text-[13px] text-fg" role="status">
          {t('importacao-pagamento-ja-ligado', {
            descricao: linked.description,
            data: formatDate(linked.date),
          })}
        </p>
      ) : null}
      <KeyValue
        items={[
          { label: t('importacao-pagamento-tipo'), value: t(PROOF_KIND[proof.kind]) },
          ...(proof.amount != null
            ? [{ label: t('importacao-pagamento-valor'), value: <Money cents={proof.amount} /> }]
            : []),
          ...(proof.date
            ? [
                {
                  label: t('importacao-pagamento-data'),
                  value: `${formatDate(proof.date)}${proof.time ? ` ${proof.time}` : ''}`,
                },
              ]
            : []),
          ...(proof.counterpart
            ? [
                {
                  label:
                    proof.direction === 'in'
                      ? t('importacao-pagamento-quem-pagou')
                      : t('importacao-pagamento-quem-recebeu'),
                  value: proof.counterpart,
                },
              ]
            : []),
          ...(proof.institution
            ? [{ label: t('importacao-pagamento-instituicao'), value: proof.institution }]
            : []),
          ...(proof.e2e_id
            ? [
                {
                  label: t('importacao-pagamento-e2e'),
                  value: <span className="num">{proof.e2e_id}</span>,
                },
              ]
            : []),
        ]}
      />
      <fieldset className="flex flex-col gap-1.5">
        <legend className="label-caps mb-1 text-fg">{t('importacao-pagamento-ligar-a')}</legend>
        {reading.matches.map((m) => (
          <label key={m.txn_id} className="flex items-center gap-2 text-[13px]">
            <input
              type="radio"
              name="proof-target"
              checked={choice === m.txn_id}
              onChange={() => setChoice(m.txn_id)}
            />
            <span className="min-w-0 flex-1 truncate">
              {formatDate(m.date)} · {m.description} · {m.account_name}
            </span>
            <Money cents={m.amount} signed />
          </label>
        ))}
        {!reading.matches.length ? (
          <p className="text-[12.5px] text-dim">{t('importacao-pagamento-nenhum')}</p>
        ) : null}
        <label className="flex items-center gap-2 text-[13px]">
          <input
            type="radio"
            name="proof-target"
            checked={choice === 'new'}
            onChange={() => setChoice('new')}
          />
          {closePlanned && !reading.matches.length
            ? t('importacao-pagamento-usar-previsto')
            : t('importacao-pagamento-criar-lancamento')}
        </label>
      </fieldset>
      {reading.planned.length ? (
        <div className="flex flex-col gap-1.5">
          {reading.planned.map((p) => (
            <Checkbox
              key={p.txn_id}
              checked={closePlanned === p.txn_id}
              onChange={(on) => setClosePlanned(on ? p.txn_id : null)}
              label={t('importacao-pagamento-quitar', {
                descricao: p.description,
                data: formatDate(p.date),
              })}
            />
          ))}
        </div>
      ) : null}
      <div className="grid gap-3 @min-[480px]:grid-cols-2">
        <Field label={t('importacao-pagamento-favorecido')}>
          <Input value={payee} onChange={(e) => setPayee(e.target.value)} />
        </Field>
        <Field label={t('importacao-pagamento-data')}>
          <DateInput value={date} onValueChange={setDate} />
        </Field>
        {creating ? (
          <>
            <Field label={t('importacao-pagamento-descricao')}>
              <Input value={description} onChange={(e) => setDescription(e.target.value)} />
            </Field>
            <Field label={t('importacao-pagamento-valor')}>
              <MoneyInput
                currency={account?.currency ?? 'BRL'}
                value={amount}
                onValueChange={setAmount}
              />
            </Field>
            <Field label={t('importacao-pagamento-conta')}>
              <Select
                value={accountId || undefined}
                onValueChange={setAccountId}
                options={options}
                placeholder={t('importacao-pagamento-escolha-conta')}
              />
            </Field>
            <Field label={t('importacao-pagamento-categoria')}>
              <CategoryPicker
                value={category}
                onChange={setCategory}
                kind={sign > 0 ? 'income' : 'expense'}
              />
            </Field>
          </>
        ) : null}
      </div>
      <Warnings list={reading.warnings} />
      <p className="text-[12px] text-dim">{t('importacao-pagamento-sem-imagem-dica')}</p>
      <InlineError error={save.error} />
      <div className="flex justify-end">
        <Button
          variant="primary"
          disabled={!ready}
          loading={save.isPending}
          onClick={() => {
            const planned = target.startsWith('planned:') ? target.slice(8) : null;
            save.mutate(
              {
                date: date ?? '',
                ...(payee.trim() ? { payee: payee.trim() } : {}),
                ...(planned
                  ? { txn_id: planned }
                  : creating
                    ? {
                        create: {
                          account_id: accountId,
                          amount: sign * (amount ?? 0),
                          description: description.trim(),
                          ...(category ? { category_id: category } : {}),
                        },
                      }
                    : { txn_id: target }),
                ...(closePlanned && !planned ? { close_planned_id: closePlanned } : {}),
                proof,
                ...(reading.sha256 ? { sha256: reading.sha256 } : {}),
              },
              {
                onSuccess: () => {
                  toast({ tone: 'success', title: t('importacao-pagamento-comprovante-salvo') });
                  onDone();
                },
              },
            );
          }}
        >
          {t('importacao-pagamento-salvar-comprovante')}
        </Button>
      </div>
    </div>
  );
}
