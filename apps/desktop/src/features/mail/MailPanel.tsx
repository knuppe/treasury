/**
 * 04 · Lançamentos › E-mail (docs/email-ingest.md): a fila de revisão do e-mail de ingestão,
 * as contas (pasta local ou caixa IMAP com senha de app), as regras e a auditoria. O que passa
 * nos filtros é lido pelos layouts (extrato OFX, cupom fiscal, aviso de compra e de Pix) e
 * espera aqui; "aplicar sozinho" só com remetente autenticado e prévia sem avisos.
 */
import { useMemo, useState } from 'react';

import type { MailAccount } from '@/api/generated/MailAccount';
import type { MailMessage } from '@/api/generated/MailMessage';
import type { MailPart } from '@/api/generated/MailPart';
import type { MailRule } from '@/api/generated/MailRule';
import type { MailTxnDraft } from '@/api/generated/MailTxnDraft';
import { useAccounts } from '@/api/hooks/accounts';
import {
  mailPartFile,
  useApplyMailPart,
  useAskMailAi,
  useConfirmRetention,
  useDeleteMailAccount,
  useDeleteMailRule,
  useIgnoreMail,
  useMailAccounts,
  useMailAudit,
  useMailMessages,
  useMailProviders,
  useMailRules,
  useMarkMailPart,
  useSyncMail,
  useTestMailConnection,
  useWouldDelete,
} from '@/api/hooks/mail';
import {
  Button,
  EmptyState,
  Icon,
  InstrumentCard,
  Money,
  Select,
  TechTable,
  toast,
} from '@/design/components';
import { useImportHub } from '@/features/import/importHub';
import {
  ConfirmDialog,
  InlineError,
  QueryState,
  Segmented,
} from '@/features/transactions/ui/controls';
import { errorText } from '@/features/transactions/ui/errors';
import { formatBytes } from '@/features/transactions/ui/hooks';
import { t } from '@/i18n';
import { formatDate, formatDateTime } from '@/lib/format';
import { useCompanion } from '@/state/companion';

import {
  ACTION_LABEL,
  AI_LABEL,
  KIND_LABEL,
  STATE_LABEL,
  STATUS_LABEL,
  STATUS_TONE,
  auditLabel,
  joinList,
} from './labels';
import { MailAccountDialog } from './MailAccountDialog';
import { MailRuleDialog } from './MailRuleDialog';

/** Peças que o importador universal abre (o usuário confere e grava lá). */
const IMPORTER_KINDS = new Set([
  'statement',
  'card_statement',
  'brokerage_note',
  'b3_report',
  'pension_statement',
  'payslip',
]);
/** Peças que o próprio e-mail importa (OFX pela conta, cupom pela SEFAZ, aviso do corpo). */
const DIRECT_KINDS = new Set(['statement', 'receipt', 'purchase', 'pix', 'template']);
const OPEN_STATUSES = new Set(['awaiting_review', 'unrecognized', 'failed']);

function isDraft(p: MailPart): boolean {
  return p.kind === 'purchase' || p.kind === 'pix' || p.kind === 'template';
}

function DraftLine({ d }: { d: MailTxnDraft }) {
  return (
    <span className="text-[12.5px] text-fg">
      {d.date ? `${formatDate(d.date)} · ` : ''}
      {d.description} · <Money cents={d.amount} />
      {d.card_last4 ? (
        <span className="text-dim">
          {' · '}
          {t('email-cartao-final', { final: d.card_last4 })}
        </span>
      ) : null}
    </span>
  );
}

function PartRow({ m, p, rules }: { m: MailMessage; p: MailPart; rules: MailRule[] }) {
  const apply = useApplyMailPart();
  const mark = useMarkMailPart();
  const ask = useAskMailAi();
  const accounts = useAccounts().data ?? [];
  const draft = isDraft(p) ? (p.detail as unknown as MailTxnDraft | null) : null;
  const [account, setAccount] = useState<string>('');
  const rule = rules.find((r) => r.id === m.rule_id);
  const aiAllowed = rule ? rule.ai !== 'never' : false;
  const open = OPEN_STATUSES.has(p.status);
  const ai = (p.detail as { ai?: { summary?: string; proposals?: string[] } } | null)?.ai;

  const run = (account_id?: string) =>
    apply.mutate(
      { part_id: p.id, ...(account_id ? { account_id } : {}) },
      {
        onSuccess: (r) =>
          toast({
            tone: r.status === 'imported' ? 'success' : 'info',
            title:
              r.status === 'imported'
                ? t('email-importado')
                : r.status === 'duplicate'
                  ? t('email-ja-estava')
                  : t('email-ficou-revisao'),
            description: r.message,
          }),
      },
    );

  const openImporter = async () => {
    try {
      const f = await mailPartFile(p.id);
      useImportHub.getState().openWith([f], { [f.name]: `mail:${p.id}` });
    } catch (e) {
      toast({ tone: 'error', title: t('email-erro-abrir'), description: errorText(e) });
    }
  };

  const toAgent = async () => {
    try {
      const f = await mailPartFile(p.id);
      useCompanion
        .getState()
        .attach(
          [f],
          `${t('email-agente-chegou', { remetente: m.original_sender ?? m.sender ?? '?' })} `,
        );
    } catch (e) {
      toast({ tone: 'error', title: t('email-erro-anexar'), description: errorText(e) });
    }
  };

  return (
    <li className="flex flex-col gap-1.5 px-3 py-2">
      <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1">
        <span className="text-[13px] text-fg">{p.label}</span>
        <span className="label-caps">{KIND_LABEL[p.kind] ?? p.kind}</span>
        <span className={`text-[12px] ${STATUS_TONE[p.status]}`}>{STATUS_LABEL[p.status]}</span>
        <span className="flex-1" />
        <span className="text-[11.5px] text-dim">
          {p.filename} · {formatBytes(p.size)}
        </span>
      </div>
      {draft ? <DraftLine d={draft} /> : null}
      {p.error ? <p className="text-[12px] text-warning">{p.error}</p> : null}
      {ai?.summary ? (
        <p className="text-[12px] text-dim">
          <Icon name="agent" size={11} className="mr-1 inline" />
          {ai.summary}
          {ai.proposals?.length ? ` · ${t('email-ia-propostas', { n: ai.proposals.length })}` : ''}
        </p>
      ) : null}
      {open ? (
        <div className="flex flex-wrap items-center gap-2">
          {draft && !draft.account_id ? (
            <Select
              aria-label={t('email-conta-lancamento')}
              value={account}
              onValueChange={setAccount}
              placeholder={t('email-escolha-conta')}
              options={accounts.map((a) => ({ value: a.id, label: a.name }))}
            />
          ) : null}
          {DIRECT_KINDS.has(p.kind) && p.has_content ? (
            <Button
              size="sm"
              variant="primary"
              loading={apply.isPending}
              disabled={Boolean(draft && !draft.account_id && !account)}
              onClick={() => run(account || undefined)}
            >
              {draft
                ? t('email-lancar')
                : p.kind === 'receipt'
                  ? t('email-importar-cupom')
                  : t('email-importar')}
            </Button>
          ) : null}
          {IMPORTER_KINDS.has(p.kind) && p.has_content ? (
            <Button size="sm" variant="secondary" onClick={() => void openImporter()}>
              {t('email-abrir-importador')}
            </Button>
          ) : null}
          {(p.kind === 'image' || p.kind === 'unknown') && p.has_content ? (
            aiAllowed ? (
              <Button
                size="sm"
                variant="secondary"
                icon={<Icon name="agent" size={12} />}
                loading={ask.isPending}
                onClick={() => ask.mutate({ part_id: p.id })}
              >
                {t('email-pedir-ia')}
              </Button>
            ) : (
              <Button
                size="sm"
                variant="ghost"
                icon={<Icon name="agent" size={12} />}
                onClick={() => void toAgent()}
              >
                {t('email-anexar-agente')}
              </Button>
            )
          ) : null}
          <Button
            size="sm"
            variant="ghost"
            onClick={() => mark.mutate({ id: p.id, status: 'imported' })}
          >
            {t('email-ja-importei')}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => mark.mutate({ id: p.id, status: 'ignored' })}
          >
            {t('email-ignorar')}
          </Button>
        </div>
      ) : null}
      <InlineError error={apply.error ?? ask.error ?? mark.error} />
    </li>
  );
}

function MessageCard({ m, rules }: { m: MailMessage; rules: MailRule[] }) {
  const ignore = useIgnoreMail();
  const forwarded = m.original_sender && m.original_sender !== m.sender;
  return (
    <article
      aria-label={m.subject ?? m.file_name ?? t('email-mensagem')}
      className="flex flex-col rounded-card border border-line"
    >
      <header className="flex flex-wrap items-start gap-x-3 gap-y-1 border-b border-line px-3 py-2">
        <div className="min-w-0 flex-1">
          <p className="truncate text-[13.5px] font-medium text-fg">
            {m.subject || m.file_name || t('email-sem-assunto')}
          </p>
          <p className="text-[12px] text-dim">
            {forwarded
              ? t('email-encaminhada-por', {
                  original: m.original_sender ?? '',
                  remetente: m.sender ?? '',
                })
              : (m.sender ??
                (m.folder === 'INBOX'
                  ? t('email-pasta-principal')
                  : t('email-pasta', { pasta: m.folder })))}
            {m.date ? ` · ${formatDate(m.date)}` : ''}
            {` · ${m.account_name} · `}
            {m.rule_name ? t('email-regra-nome', { regra: m.rule_name }) : t('email-sem-regra')}
          </p>
        </div>
        <span
          className={`text-[11.5px] ${m.authenticated ? 'text-positive' : 'text-dim'}`}
          title={m.auth_detail ?? undefined}
        >
          <Icon name={m.authenticated ? 'shield' : 'info'} size={11} className="mr-1 inline" />
          {m.authenticated ? t('email-autenticado') : t('email-nao-autenticado')}
        </span>
        <span className={`text-[12px] ${STATUS_TONE[m.status]}`}>{STATUS_LABEL[m.status]}</span>
      </header>
      {m.error ? <p className="px-3 pt-2 text-[12px] text-warning">{m.error}</p> : null}
      <ul className="divide-y divide-line">
        {m.parts.map((p) => (
          <PartRow key={p.id} m={m} p={p} rules={rules} />
        ))}
      </ul>
      {m.status === 'awaiting_review' || m.status === 'unrecognized' ? (
        <footer className="flex justify-end border-t border-line px-3 py-1.5">
          <Button
            size="sm"
            variant="ghost"
            loading={ignore.isPending}
            onClick={() => ignore.mutate({ id: m.id })}
          >
            {t('email-ignorar-mensagem')}
          </Button>
        </footer>
      ) : null}
    </article>
  );
}

function QueueCard({ accounts, onNew }: { accounts: MailAccount[]; onNew: () => void }) {
  const [show, setShow] = useState<'pending' | 'all'>('pending');
  const q = useMailMessages(
    show === 'pending'
      ? { status: ['awaiting_review', 'unrecognized', 'failed'], limit: 100 }
      : { limit: 200 },
  );
  const rules = useMailRules().data ?? [];
  const list = q.data ?? [];
  return (
    <InstrumentCard
      index="04.E1"
      title={t('email-fila')}
      actions={
        <Segmented
          size="sm"
          label={t('email-mensagens')}
          value={show}
          onChange={setShow}
          options={[
            { value: 'pending', label: t('email-para-revisar') },
            { value: 'all', label: t('email-tudo') },
          ]}
        />
      }
    >
      {!accounts.length ? (
        <EmptyState
          kicker={t('email-vazio-kicker')}
          title={t('email-vazio-titulo')}
          description={t('email-vazio-descricao')}
          actions={
            <Button variant="primary" icon={<Icon name="plus" size={12} />} onClick={onNew}>
              {t('email-configurar')}
            </Button>
          }
        />
      ) : (
        <QueryState loading={q.isLoading} error={q.error}>
          {list.length ? (
            <div className="flex flex-col gap-3">
              {list.map((m) => (
                <MessageCard key={m.id} m={m} rules={rules} />
              ))}
            </div>
          ) : (
            <p className="text-[13px] text-dim">
              {show === 'pending' ? t('email-nada-revisar') : t('email-nenhuma-mensagem')}
            </p>
          )}
        </QueryState>
      )}
    </InstrumentCard>
  );
}

function RetentionBanner({ a }: { a: MailAccount }) {
  const would = useWouldDelete(a.retention_state === 'off' ? null : a.id);
  const confirm = useConfirmRetention();
  if (a.retention_state !== 'testing' && a.retention_state !== 'awaiting_confirmation') return null;
  const n = would.data?.length ?? 0;
  return (
    <div
      role="note"
      className="flex flex-col gap-2 rounded-card border border-warning/60 bg-warning/5 px-3 py-2 text-[12.5px] text-warning"
    >
      <p>
        <Icon name="alert" size={12} className="mr-1 inline" />
        {a.retention_state === 'testing'
          ? t('email-teste-ate', { data: formatDateTime(a.retention_test_until ?? '') })
          : t('email-teste-acabou')}{' '}
        {n ? t('email-teriam-saido', { n }) : t('email-nada-teria-saido')}
      </p>
      {a.retention_state === 'awaiting_confirmation' ? (
        <div>
          <Button
            size="sm"
            variant="danger"
            loading={confirm.isPending}
            onClick={() => confirm.mutate({ id: a.id })}
          >
            {t('email-confirmar-exclusao')}
          </Button>
        </div>
      ) : null}
      <InlineError error={confirm.error} />
    </div>
  );
}

function AccountsCard({
  accounts,
  onEdit,
  onNew,
}: {
  accounts: MailAccount[];
  onEdit: (a: MailAccount) => void;
  onNew: () => void;
}) {
  const providers = useMailProviders().data ?? [];
  const sync = useSyncMail();
  const test = useTestMailConnection();
  const del = useDeleteMailAccount();
  const [removing, setRemoving] = useState<MailAccount | null>(null);
  const kindOf = (a: MailAccount) =>
    a.kind === 'folder'
      ? t('email-pasta-local')
      : (providers.find((p) => p.id === a.provider)?.label ?? 'IMAP');
  return (
    <InstrumentCard
      index="04.E2"
      title={t('email-contas')}
      actions={
        <Button size="sm" variant="secondary" icon={<Icon name="plus" size={12} />} onClick={onNew}>
          {t('email-nova-conta')}
        </Button>
      }
    >
      <div className="flex flex-col gap-3">
        {accounts.map((a) => (
          <section
            key={a.id}
            aria-label={t('email-conta-nome', { nome: a.name })}
            className="flex flex-col gap-2 rounded-card border border-line px-3 py-2.5"
          >
            <div className="flex flex-wrap items-start gap-x-3 gap-y-1">
              <div className="min-w-0 flex-1">
                <p className="text-[13.5px] font-medium text-fg">
                  {a.name} <span className="label-caps ml-1">{kindOf(a)}</span>
                  {!a.enabled ? (
                    <span className="ml-2 text-[12px] text-dim">{t('email-desligada')}</span>
                  ) : null}
                </p>
                <p className="truncate text-[12px] text-dim">
                  {[
                    a.kind === 'folder' ? a.folder_path : a.username,
                    t('email-a-cada', { min: a.interval_minutes }),
                    STATE_LABEL[a.retention_state],
                    ...(a.pending ? [t('email-n-para-revisar', { n: a.pending })] : []),
                  ].join(' · ')}
                </p>
                <p
                  className={`text-[12px] ${a.status && !a.status.ok ? 'text-negative' : 'text-dim'}`}
                >
                  {a.syncing
                    ? t('email-buscando')
                    : a.status
                      ? t('email-ultima-busca', {
                          quando: formatDateTime(a.status.at),
                          resultado: a.status.ok
                            ? t('email-busca-resumo', {
                                novas: a.status.seen,
                                fila: a.status.passed,
                                recusadas: a.status.rejected,
                                aplicadas: a.status.applied,
                              })
                            : a.status.message,
                        })
                      : t('email-ainda-nao-buscou')}
                </p>
                {a.kind === 'imap' && !a.password?.set ? (
                  <p className="text-[12px] text-warning">{t('email-cadastre-senha')}</p>
                ) : null}
                {a.needs_reconfirm ? (
                  <p className="text-[12px] text-warning">{t('email-backup-restaurado')}</p>
                ) : null}
              </div>
              <div className="flex flex-wrap gap-1.5">
                <Button
                  size="sm"
                  variant="primary"
                  loading={sync.isPending && sync.variables?.id === a.id}
                  disabled={a.syncing || a.needs_reconfirm}
                  onClick={() => sync.mutate({ id: a.id })}
                >
                  {t('email-buscar-agora')}
                </Button>
                <Button
                  size="sm"
                  variant="secondary"
                  loading={test.isPending && test.variables?.id === a.id}
                  onClick={() =>
                    test.mutate(
                      { id: a.id },
                      {
                        onSuccess: (r) =>
                          toast({
                            tone: 'success',
                            title: t('email-conexao-ok'),
                            description:
                              'files' in r
                                ? t('email-n-arquivos-pasta', { n: r.files })
                                : t('email-teste-le', {
                                    pastas: joinList(r.reads) || t('email-nenhuma-pasta'),
                                    como: r.delete_how,
                                  }),
                          }),
                      },
                    )
                  }
                >
                  {t('email-testar')}
                </Button>
                <Button size="sm" variant="ghost" onClick={() => onEdit(a)}>
                  {t('comum-editar')}
                </Button>
                <Button size="sm" variant="ghost" onClick={() => setRemoving(a)}>
                  {t('comum-apagar')}
                </Button>
              </div>
            </div>
            <RetentionBanner a={a} />
          </section>
        ))}
        <InlineError error={sync.error ?? test.error} />
      </div>
      <ConfirmDialog
        open={removing != null}
        onOpenChange={(o) => !o && setRemoving(null)}
        title={t('email-apagar-conta')}
        description={t('email-apagar-conta-descricao')}
        confirmLabel={t('comum-apagar')}
        pending={del.isPending}
        error={del.error}
        onConfirm={() =>
          removing && del.mutate({ id: removing.id }, { onSuccess: () => setRemoving(null) })
        }
      />
    </InstrumentCard>
  );
}

function RulesCard({ accounts }: { accounts: MailAccount[] }) {
  const q = useMailRules();
  const del = useDeleteMailRule();
  const [editing, setEditing] = useState<MailRule | 'new' | null>(null);
  const name = (id: string | null) =>
    id ? (accounts.find((a) => a.id === id)?.name ?? '?') : t('email-todas-contas');
  return (
    <InstrumentCard
      index="04.E3"
      title={t('email-regras')}
      actions={
        <Button
          size="sm"
          variant="secondary"
          icon={<Icon name="plus" size={12} />}
          disabled={!accounts.length}
          onClick={() => setEditing('new')}
        >
          {t('email-nova-regra')}
        </Button>
      }
    >
      <QueryState loading={q.isLoading} error={q.error}>
        {(q.data ?? []).length ? (
          <TechTable
            label={t('email-regras-tabela')}
            prefsId="Regras do e-mail"
            dense
            rows={q.data ?? []}
            getRowId={(r) => r.id}
            columns={[
              {
                key: 'priority',
                header: t('email-col-ordem'),
                numeric: true,
                width: '64px',
                cell: (r) => r.priority,
              },
              {
                key: 'name',
                header: t('email-col-regra'),
                cell: (r) => (
                  <span className={r.enabled ? 'text-fg' : 'text-dim'}>
                    {r.name}
                    <span className="block text-[11.5px] text-dim">
                      {r.conditions.from.length
                        ? t('email-regra-de', { remetentes: joinList(r.conditions.from) })
                        : t('email-qualquer-remetente')}
                      {r.conditions.subject.contains.length
                        ? ` · ${t('email-regra-assunto', { termos: joinList(r.conditions.subject.contains) })}`
                        : ''}
                    </span>
                  </span>
                ),
              },
              { key: 'account', header: t('email-col-conta'), cell: (r) => name(r.account_id) },
              { key: 'action', header: t('email-col-acao'), cell: (r) => ACTION_LABEL[r.action] },
              { key: 'ai', header: t('email-col-ia'), cell: (r) => AI_LABEL[r.ai] },
              {
                key: 'actions',
                header: <span className="sr-only">{t('email-col-acoes')}</span>,
                align: 'right',
                width: '150px',
                cell: (r) => (
                  <span className="inline-flex gap-1">
                    <Button size="sm" variant="ghost" onClick={() => setEditing(r)}>
                      {t('comum-editar')}
                    </Button>
                    <Button size="sm" variant="ghost" onClick={() => del.mutate({ id: r.id })}>
                      {t('comum-apagar')}
                    </Button>
                  </span>
                ),
              },
            ]}
          />
        ) : (
          <p className="text-[13px] text-dim">{t('email-sem-regras')}</p>
        )}
      </QueryState>
      {editing ? (
        <MailRuleDialog
          rule={editing === 'new' ? null : editing}
          accounts={accounts}
          onClose={() => setEditing(null)}
        />
      ) : null}
    </InstrumentCard>
  );
}

function AuditCard() {
  const q = useMailAudit();
  const rows = useMemo(() => (q.data ?? []).slice(0, 150), [q.data]);
  return (
    <InstrumentCard index="04.E4" title={t('email-auditoria')}>
      <QueryState loading={q.isLoading} error={q.error}>
        {rows.length ? (
          <TechTable
            label={t('email-auditoria-tabela')}
            prefsId="Auditoria do e-mail"
            dense
            rows={rows}
            getRowId={(e) => e.id}
            columns={[
              {
                key: 'at',
                header: t('email-col-quando'),
                width: '140px',
                numeric: true,
                align: 'left',
                cell: (e) => formatDateTime(e.at),
              },
              {
                key: 'action',
                header: t('email-col-o-que'),
                width: '130px',
                cell: (e) => auditLabel(e),
              },
              {
                key: 'detail',
                header: t('email-col-detalhe'),
                cell: (e) => <span className="text-dim">{e.detail}</span>,
              },
            ]}
          />
        ) : (
          <p className="text-[13px] text-dim">{t('email-nada-registrado')}</p>
        )}
      </QueryState>
    </InstrumentCard>
  );
}

export function MailPanel() {
  const q = useMailAccounts();
  const accounts = q.data ?? [];
  const [editing, setEditing] = useState<MailAccount | 'new' | null>(null);
  return (
    <div className="flex flex-col gap-4">
      <QueueCard accounts={accounts} onNew={() => setEditing('new')} />
      {accounts.length ? (
        <>
          <AccountsCard accounts={accounts} onEdit={setEditing} onNew={() => setEditing('new')} />
          <RulesCard accounts={accounts} />
          <AuditCard />
        </>
      ) : null}
      {editing ? (
        <MailAccountDialog
          account={editing === 'new' ? null : editing}
          onClose={() => setEditing(null)}
        />
      ) : null}
    </div>
  );
}
