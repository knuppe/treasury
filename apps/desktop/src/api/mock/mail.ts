/**
 * Mock de `mail.*` (docs/email-ingest.md): contas, regras, a fila e a auditoria em memória.
 * "Buscar agora" traz duas mensagens de exemplo (uma fatura OFX e um aviso de compra); o
 * núcleo é que lê o IMAP, a pasta e os layouts de verdade. A senha não é guardada aqui: só o
 * "cadastrada" e a dica, como no núcleo.
 */
import type { MailAccount } from '@/api/generated/MailAccount';
import type { MailAccountInput } from '@/api/generated/MailAccountInput';
import type { MailAuditEntry } from '@/api/generated/MailAuditEntry';
import type { MailDecision } from '@/api/generated/MailDecision';
import type { MailMessage } from '@/api/generated/MailMessage';
import type { MailPart } from '@/api/generated/MailPart';
import type { MailProvider } from '@/api/generated/MailProvider';
import type { MailRule } from '@/api/generated/MailRule';
import type { MailRuleInput } from '@/api/generated/MailRuleInput';
import type { MailStatus } from '@/api/generated/MailStatus';
import type { RetentionState } from '@/api/generated/RetentionState';

import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { conflict, fail, invalid, notFound } from './ledger/errors';
import { newId, rows } from './ledger/store';

const PROVIDERS: MailProvider[] = [
  {
    id: 'gmail',
    label: 'Gmail',
    host: 'imap.gmail.com',
    port: 993,
    security: 'tls',
    username: 'address',
    authserv_id: 'mx.google.com',
    app_password_url: 'https://myaccount.google.com/apppasswords',
    help: 'Senha de app de 16 letras, criada na Conta do Google; precisa da verificação em duas etapas ligada.',
    gmail: true,
    folder_names: [],
  },
  {
    id: 'icloud',
    label: 'iCloud Mail',
    host: 'imap.mail.me.com',
    port: 993,
    security: 'tls',
    username: 'local_part',
    authserv_id: 'spf.icloud.com,dkim-verifier.icloud.com,dmarc.icloud.com',
    app_password_url: 'https://account.apple.com',
    help: 'Senha de app; precisa da autenticação de dois fatores. O usuário é o nome antes do @.',
    gmail: false,
    folder_names: [['Deleted Messages', '\\Trash']],
  },
  {
    id: 'proton',
    label: 'Proton Mail (Bridge)',
    host: '127.0.0.1',
    port: 1143,
    security: 'starttls',
    username: 'address',
    authserv_id: 'mail.protonmail.ch',
    app_password_url: 'https://proton.me/support/imap-smtp-and-pop3-setup',
    help: 'Precisa do Proton Mail Bridge aberto neste computador; fixe o cert.pem exportado.',
    gmail: false,
    folder_names: [],
  },
];

const DOMAINS: Record<string, string> = {
  'gmail.com': 'gmail',
  'icloud.com': 'icloud',
  'me.com': 'icloud',
  'proton.me': 'proton',
};

const OFX_B64 = btoa(
  'OFXHEADER:100\nDATA:OFXSGML\n\n<OFX><CREDITCARDMSGSRSV1><CCSTMTTRNRS><CCSTMTRS><CURDEF>BRL<CCACCTFROM><ACCTID>5555********4444</CCACCTFROM></CCSTMTRS></CCSTMTTRNRS></CREDITCARDMSGSRSV1></OFX>',
);

interface Stored extends MailAccount {
  password_hint: string | null;
}

let accounts: Stored[] = [];
let rules: MailRule[] = [];
let messages: MailMessage[] = [];
let audit: MailAuditEntry[] = [];

onMockReset(() => {
  accounts = [];
  rules = [];
  messages = [];
  audit = [];
});

const now = () => new Date().toISOString();
const str = (v: unknown) => (typeof v === 'string' ? v : undefined);

function log(account_id: string | null, action: string, detail: string, message_id?: string) {
  audit.unshift({
    id: newId(),
    account_id,
    at: now(),
    action,
    message_id: message_id ?? null,
    detail,
  });
}

function state(a: Stored): RetentionState {
  const deletes =
    a.retention.mode !== 'never' ||
    rules.some(
      (r) => (r.account_id ?? a.id) === a.id && r.retention && r.retention.mode !== 'never',
    );
  if (!deletes) return 'off';
  if (a.retention_confirmed) return 'active';
  return a.retention_test_until && a.retention_test_until > now()
    ? 'testing'
    : 'awaiting_confirmation';
}

function view(a: Stored): MailAccount {
  const { password_hint, ...rest } = a;
  return {
    ...rest,
    retention_state: state(a),
    password:
      a.kind === 'imap'
        ? {
            set: Boolean(password_hint),
            hint: password_hint,
            needs_reentry: false,
            updated_at: null,
          }
        : null,
    pending: messages.filter((m) => m.account_id === a.id && m.status === 'awaiting_review').length,
  };
}

function find(id: unknown): Stored {
  return accounts.find((a) => a.id === id) ?? notFound(`conta de e-mail ${String(id)}`);
}

function syncTestMode(a: Stored) {
  const s = state(a);
  if (s === 'off') {
    a.retention_test_until = null;
    a.retention_confirmed = false;
  } else if (!a.retention_test_until && !a.retention_confirmed) {
    a.retention_test_until = new Date(Date.now() + 7 * 86_400_000).toISOString();
  }
}

function saveAccount(args: Record<string, unknown>): MailAccount {
  const a = args as unknown as MailAccountInput;
  if (!a.name?.trim()) invalid('dê um nome à conta');
  const old = a.id ? find(a.id) : null;
  const kind = a.kind ?? old?.kind ?? invalid('escolha o tipo: pasta local ou IMAP');
  const prov = PROVIDERS.find((p) => p.id === (a.provider ?? old?.provider));
  if (kind === 'folder' && !(a.folder_path ?? old?.folder_path)) invalid('escolha a pasta');
  if (kind === 'imap' && !(a.username ?? old?.username))
    invalid('informe o usuário (o endereço da caixa)');
  const host = a.host ?? old?.host ?? prov?.host ?? null;
  const mailbox = a.mailbox_kind ?? old?.mailbox_kind ?? 'dedicated';
  const base: Stored = {
    id: old?.id ?? newId(),
    name: a.name.trim(),
    kind,
    folder_path: kind === 'folder' ? (a.folder_path ?? old?.folder_path ?? null) : null,
    provider: kind === 'imap' ? (prov?.id ?? 'manual') : null,
    host: kind === 'imap' ? host : null,
    port: kind === 'imap' ? (a.port ?? old?.port ?? prov?.port ?? 993) : null,
    security: a.security ?? old?.security ?? prov?.security ?? 'tls',
    username: kind === 'imap' ? (a.username ?? old?.username ?? null) : null,
    pinned_cert: a.pinned_cert ? 'ab'.repeat(32) : (old?.pinned_cert ?? null),
    authserv_id: a.authserv_id ?? old?.authserv_id ?? prov?.authserv_id ?? null,
    mailbox_kind: mailbox,
    allowed_folders:
      a.allowed_folders ?? old?.allowed_folders ?? (mailbox === 'dedicated' ? ['INBOX'] : []),
    blocked_folders: a.blocked_folders ??
      old?.blocked_folders ?? ['\\Junk', '\\Trash', '\\Sent', '\\Drafts', '\\All'],
    allow_senders: a.allow_senders ?? old?.allow_senders ?? [],
    block_senders: a.block_senders ?? old?.block_senders ?? [],
    family_addresses: a.family_addresses ?? old?.family_addresses ?? [],
    default_action:
      (a.default_action ?? old?.default_action ?? (kind === 'folder' ? 'review' : 'ignore')) ===
      'review'
        ? 'review'
        : 'ignore',
    retention: a.retention ??
      old?.retention ?? {
        mode: 'never',
        days: 30,
        grace_days: 7,
        local: 'move',
        purge_trash: false,
      },
    retention_test_until: old?.retention_test_until ?? null,
    retention_confirmed: old?.retention_confirmed ?? false,
    retention_state: 'off',
    keep_original: a.keep_original ?? old?.keep_original ?? true,
    interval_minutes: a.interval_minutes ?? old?.interval_minutes ?? 15,
    enabled: a.enabled ?? old?.enabled ?? true,
    needs_reconfirm: false,
    syncing: false,
    password: null,
    status: old?.status ?? null,
    pending: 0,
    created_at: old?.created_at ?? now(),
    updated_at: now(),
    password_hint: old && old.host === host ? old.password_hint : null,
  };
  if (base.interval_minutes < 5 || base.interval_minutes > 120)
    invalid('intervalo das buscas: de 5 a 120 minutos');
  accounts = old ? accounts.map((x) => (x.id === old.id ? base : x)) : [...accounts, base];
  syncTestMode(base);
  return view(base);
}

function saveRule(args: Record<string, unknown>): MailRule {
  const r = args as unknown as MailRuleInput;
  if (!r.name?.trim()) invalid('dê um nome à regra');
  if (r.conditions?.subject?.regex) {
    try {
      new RegExp(r.conditions.subject.regex);
    } catch (e) {
      invalid(`expressão regular inválida:\n${String(e)}`);
    }
  }
  const old = r.id ? rules.find((x) => x.id === r.id) : undefined;
  const rule: MailRule = {
    id: old?.id ?? newId(),
    account_id: r.account_id ?? null,
    name: r.name.trim(),
    priority: r.priority ?? old?.priority ?? 100,
    enabled: r.enabled ?? true,
    conditions: r.conditions,
    action: r.action ?? 'review',
    ai: r.ai ?? 'never',
    retention: r.retention ?? null,
    template: r.template ?? null,
    created_at: old?.created_at ?? now(),
    updated_at: now(),
  };
  rules = (old ? rules.map((x) => (x.id === old.id ? rule : x)) : [...rules, rule]).sort(
    (a, b) => a.priority - b.priority,
  );
  accounts.forEach(syncTestMode);
  return rule;
}

function senderMatches(pattern: string, from: string): boolean {
  const p = pattern.trim().toLowerCase();
  const domain = from.split('@')[1] ?? from;
  if (p.startsWith('*.')) return domain.endsWith(p.slice(1));
  if (p.startsWith('@')) return domain === p.slice(1);
  if (p.includes('@')) return p === from;
  return domain === p;
}

function decide(rule: MailRuleInput, from: string, subject: string): MailDecision {
  const s = subject.toLowerCase().replace(/^(\s*(fwd|fw|enc|res|re):\s*)+/i, '');
  const fromOk =
    !rule.conditions.from.length || rule.conditions.from.some((p) => senderMatches(p, from));
  const subj = rule.conditions.subject;
  const has = (t: string) => s.includes(t.toLowerCase());
  const subjectOk =
    (!subj.contains.length || (subj.all ? subj.contains.every(has) : subj.contains.some(has))) &&
    !subj.not_contains.some(has);
  const passed = fromOk && subjectOk && rule.action !== 'ignore';
  return {
    passed,
    rule_id: fromOk && subjectOk ? (rule.id ?? 'teste') : null,
    rule_name: fromOk && subjectOk ? rule.name : null,
    action: fromOk && subjectOk ? (rule.action ?? 'review') : null,
    reason:
      fromOk && subjectOk
        ? `regra "${rule.name}"`
        : 'nenhuma regra casou: padrão da conta (ignorar)',
    needs_body: false,
  };
}

function part(over: Partial<MailPart>): MailPart {
  return {
    id: newId(),
    idx: 0,
    filename: 'anexo',
    mime: null,
    size: 1000,
    kind: 'unknown',
    label: 'Não reconhecido',
    layout_id: null,
    status: 'awaiting_review',
    detail: null,
    result_ref: null,
    error: null,
    has_content: true,
    ...over,
  };
}

function rollup(m: MailMessage) {
  const s = m.parts.map((p) => p.status);
  const status: MailStatus = s.some((x) => x === 'awaiting_review' || x === 'ai_pending')
    ? 'awaiting_review'
    : s.includes('imported')
      ? 'imported'
      : s.includes('failed')
        ? 'failed'
        : s.includes('unrecognized')
          ? 'unrecognized'
          : s.includes('duplicate')
            ? 'duplicate'
            : 'ignored';
  m.status = status;
  m.decided_at = status === 'awaiting_review' ? null : (m.decided_at ?? now());
}

function sample(a: Stored) {
  if (messages.some((m) => m.account_id === a.id)) return 0;
  const card = rows('credit_card').find((c) => c.last4) ?? rows('credit_card')[0];
  const rule = rules.find((r) => (r.account_id ?? a.id) === a.id);
  const base = {
    account_id: a.id,
    account_name: a.name,
    folder: 'INBOX',
    file_name: null,
    size: 24_000,
    rule_id: rule?.id ?? null,
    rule_name: rule?.name ?? null,
    error: null,
    received_at: now(),
    decided_at: null,
    deleted_at: null,
    has_raw: a.keep_original,
    status: 'awaiting_review' as MailStatus,
  };
  messages.unshift(
    {
      ...base,
      id: newId(),
      sender: 'ana@gmail.com',
      original_sender: 'todomundo@nubank.com.br',
      subject: 'Sua fatura de outubro chegou',
      date: now().slice(0, 10),
      authenticated: true,
      trusted: true,
      auth_detail: 'DMARC pass (mx.google.com)',
      parts: [
        part({
          filename: 'fatura-outubro.ofx',
          mime: 'application/x-ofx',
          kind: 'statement',
          label: 'Fatura de cartão OFX',
        }),
      ],
    },
    {
      ...base,
      id: newId(),
      sender: 'avisos@nubank.com.br',
      original_sender: null,
      subject: 'Compra aprovada',
      date: now().slice(0, 10),
      authenticated: true,
      trusted: true,
      auth_detail: 'DKIM pass alinhado ao remetente (mx.google.com)',
      parts: [
        part({
          filename: 'corpo do e-mail',
          mime: 'text/plain',
          kind: 'purchase',
          label: 'Aviso de compra: PADARIA PAO QUENTE',
          detail: {
            kind: 'purchase',
            amount: -4590,
            date: now().slice(0, 10),
            description: 'PADARIA PAO QUENTE',
            card_last4: card?.last4 ?? '4444',
            account_id: card?.id ?? null,
          },
        }),
      ],
    },
  );
  return 2;
}

function findPart(id: unknown): { m: MailMessage; p: MailPart } {
  for (const m of messages) {
    const p = m.parts.find((x) => x.id === id);
    if (p) return { m, p };
  }
  return notFound(`peça ${String(id)}`);
}

registerMockHandlers(
  guarded({
    'mail.providers': () => PROVIDERS,
    'mail.accounts.list': () => accounts.map(view),
    'mail.accounts.get': (args) => view(find(args.id)),
    'mail.accounts.save': (args) => saveAccount(args),
    'mail.accounts.delete': (args) => {
      find(args.id);
      accounts = accounts.filter((a) => a.id !== args.id);
      messages = messages.filter((m) => m.account_id !== args.id);
      return null;
    },
    'mail.accounts.set_password': (args) => {
      const a = find(args.id);
      if (a.kind !== 'imap') invalid('a pasta local não tem senha');
      const pw = str(args.password)?.trim() ?? '';
      if (!pw) invalid('informe a chave');
      a.password_hint = `…${pw.slice(-4)}`;
      return { set: true, hint: a.password_hint, needs_reentry: false, updated_at: now() };
    },
    'mail.accounts.confirm_retention': (args) => {
      const a = find(args.id);
      const s = state(a);
      if (s === 'testing')
        conflict(
          'a exclusão ainda está no modo de teste: confira o que seria apagado até o fim do teste',
        );
      if (s !== 'awaiting_confirmation') conflict('não há exclusão esperando confirmação');
      a.retention_confirmed = true;
      return view(a);
    },
    'mail.rules.list': (args) =>
      rules.filter((r) => !args.account_id || !r.account_id || r.account_id === args.account_id),
    'mail.rules.save': (args) => saveRule(args),
    'mail.rules.delete': (args) => {
      rules = rules.filter((r) => r.id !== args.id);
      accounts.forEach(syncTestMode);
      return null;
    },
    'mail.rules.test': (args) => {
      const rule = args.rule as MailRuleInput;
      const s = args.sample as { from?: string; subject?: string } | undefined;
      return {
        sample: s ? decide(rule, s.from ?? '', s.subject ?? '') : null,
        recent: messages
          .filter((m) => m.account_id === args.account_id)
          .map((m) => {
            const d = decide(rule, m.original_sender ?? m.sender ?? '', m.subject ?? '');
            return {
              message_id: m.id,
              subject: m.subject,
              sender: m.original_sender ?? m.sender,
              matches: d.rule_id != null,
              decision: d,
            };
          }),
      };
    },
    'mail.discover': (args) => {
      const address = str(args.address) ?? '';
      const domain = address.split('@')[1]?.toLowerCase() ?? '';
      if (/^(outlook|hotmail|live|msn)\./.test(domain))
        invalid(
          'Outlook e Hotmail só aceitam login pela Microsoft (OAuth2), sem senha de app: por enquanto, use a pasta local (salve os anexos ou exporte as mensagens .eml nela)',
        );
      const p = PROVIDERS.find((x) => x.id === DOMAINS[domain]);
      if (!p)
        invalid(
          `não achei a configuração IMAP de ${domain}: preencha o servidor à mão (o provedor mostra nos ajustes da caixa)`,
        );
      return {
        provider: p.id,
        host: p.host,
        port: p.port,
        security: p.security,
        username: p.username === 'local_part' ? address.split('@')[0] : address.toLowerCase(),
        source: 'catalog',
      };
    },
    'mail.test_connection': (args) => {
      const a = find(args.id);
      if (a.kind === 'folder') return { files: 2, folder: a.folder_path };
      if (!a.password_hint && !str(args.password)) invalid('informe a senha de app');
      a.needs_reconfirm = false;
      const gmail = a.provider === 'gmail';
      return {
        folders: [
          { name: 'INBOX', flags: [], selectable: true },
          { name: gmail ? '[Gmail]/Lixeira' : 'Trash', flags: ['\\Trash'], selectable: true },
          { name: gmail ? '[Gmail]/Spam' : 'Junk', flags: ['\\Junk'], selectable: true },
        ],
        reads: a.allowed_folders.includes('INBOX') ? ['INBOX'] : [],
        trash: gmail ? '[Gmail]/Lixeira' : 'Trash',
        delete_how: gmail
          ? 'move para "[Gmail]/Lixeira" (no Gmail, tirar da caixa só arquiva; a lixeira esvazia em 30 dias)'
          : 'move para "Trash"',
        capabilities: ['IMAP4rev1', 'MOVE', 'UIDPLUS'],
      };
    },
    'mail.sync_now': (args) => {
      const a = find(args.id);
      if (a.needs_reconfirm)
        conflict('backup restaurado: confirme a conexão (Testar) antes de buscar');
      if (a.kind === 'imap' && !a.password_hint) invalid('cadastre a senha de app da conta');
      const n = sample(a);
      a.status = { at: now(), ok: true, message: '', seen: n, passed: n, rejected: 0, applied: 0 };
      log(
        a.id,
        'sync',
        `${n} novas, ${n} na fila, 0 recusadas pelos filtros, 0 aplicadas sozinhas`,
      );
      return { started: true };
    },
    'mail.list': (args) => {
      const st = Array.isArray(args.status) ? (args.status as string[]) : [];
      return messages
        .filter((m) => !args.account_id || m.account_id === args.account_id)
        .filter((m) => !st.length || st.includes(m.status));
    },
    'mail.get': (args) =>
      messages.find((m) => m.id === args.id) ?? notFound(`mensagem ${String(args.id)}`),
    'mail.audit': (args) =>
      audit.filter((e) => !args.account_id || e.account_id === args.account_id),
    'mail.would_delete': (args) =>
      audit.filter((e) => e.account_id === args.account_id && e.action === 'would_delete'),
    'mail.part_content': (args) => {
      const { p } = findPart(args.id);
      if (!p.has_content) conflict('o conteúdo desta peça não está mais no cofre (já foi tratada)');
      return {
        filename: p.filename,
        mime: p.mime,
        kind: p.kind,
        content_base64: p.kind === 'statement' ? OFX_B64 : btoa('exemplo'),
      };
    },
    'mail.mark_part': (args) => {
      const { m, p } = findPart(args.id);
      if (!['imported', 'ignored', 'duplicate'].includes(String(args.status)))
        invalid('marque como importada, repetida ou ignorada');
      p.status = args.status as MailStatus;
      rollup(m);
      log(m.account_id, 'reviewed', `peça marcada como ${p.status}`, m.id);
      return m;
    },
    'mail.ignore': (args) => {
      const m = messages.find((x) => x.id === args.id) ?? notFound('mensagem');
      for (const p of m.parts)
        if (['awaiting_review', 'ai_pending', 'unrecognized', 'failed'].includes(p.status))
          p.status = 'ignored';
      rollup(m);
      return m;
    },
    'mail.apply_part': (args) => {
      const { m, p } = findPart(args.part_id);
      const d = p.detail as { account_id?: string | null } | null;
      if (['purchase', 'pix', 'template'].includes(p.kind) && !(args.account_id || d?.account_id)) {
        p.error = 'o aviso não diz a conta: escolha na revisão';
        return { status: 'awaiting_review', message: p.error };
      }
      p.status = 'imported';
      p.error = null;
      rollup(m);
      log(m.account_id, 'imported', 'importada na revisão', m.id);
      return { status: m.status, message: 'importada' };
    },
    'mail.ask_ai': (args) => {
      const { m, p } = findPart(args.part_id);
      const rule = rules.find((r) => r.id === m.rule_id);
      if (!rule || rule.ai === 'never')
        fail('forbidden', 'a regra desta mensagem não deixa a IA ler (ligue em Regras › IA)');
      p.status = 'unrecognized';
      p.detail = {
        ai: { summary: 'Comprovante de Pix (exemplo do modo de demonstração).', proposals: [] },
      };
      rollup(m);
      return { job_id: newId() };
    },
  }),
);
