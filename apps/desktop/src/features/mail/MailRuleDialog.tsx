/**
 * Regra do e-mail de ingestão: remetente (endereço, domínio, subdomínios, curinga, /regex/ ou
 * membros), assunto (contém, todos, não contém, regex), pastas e condições avançadas; o que
 * fazer (revisar, aplicar sozinho, ignorar), a IA (nunca, a pedido, sozinha), a retenção
 * própria e o modelo do corpo. "Testar" mostra o que a regra decidiria sem importar nada.
 */
import { useState } from 'react';

import type { MailAccount } from '@/api/generated/MailAccount';
import type { MailAiMode } from '@/api/generated/MailAiMode';
import type { MailCondition } from '@/api/generated/MailCondition';
import type { MailRule } from '@/api/generated/MailRule';
import type { MailRuleAction } from '@/api/generated/MailRuleAction';
import type { MailRuleInput } from '@/api/generated/MailRuleInput';
import type { RetentionMode } from '@/api/generated/RetentionMode';
import { useAccounts } from '@/api/hooks/accounts';
import { useSaveMailRule, useTestMailRule, type MailRuleTest } from '@/api/hooks/mail';
import { Button, Dialog, Field, Input, Select, toast } from '@/design/components';
import {
  Checkbox,
  InlineError,
  Segmented,
  Switch,
  TextArea,
} from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

import { ACTION_LABEL, AI_LABEL, RETENTION_LABEL, joinList, splitList } from './labels';

const ALL_ACCOUNTS = '__todas__';
const ACCOUNT_DEFAULT = '__conta__';

/** Exemplo do modelo do corpo (expressão regular com os grupos que o núcleo reconhece). */
const TEMPLATE_EXAMPLE = 'Débito de (?P<valor>R\\$\\s*[\\d.,]+) - (?P<descricao>[^\\n]+)';

/** As condições avançadas simples (anexos, corpo, tamanho) de volta para o formulário. */
function simpleOf(c: MailCondition | undefined) {
  const out = { attachments: '', body: '', maxKb: '', other: false };
  if (!c) return out;
  const list = c.type === 'all' ? c.of : [c];
  for (const x of list) {
    if (x.type === 'attachment') out.attachments = joinList(x.names);
    else if (x.type === 'body_contains') out.body = joinList(x.terms);
    else if (x.type === 'size_at_most') out.maxKb = String(x.kb);
    else out.other = true;
  }
  return out;
}

export function MailRuleDialog({
  rule,
  accounts,
  onClose,
}: {
  rule: MailRule | null;
  accounts: MailAccount[];
  onClose: () => void;
}) {
  const save = useSaveMailRule();
  const testRule = useTestMailRule();
  const ledgerAccounts = useAccounts().data ?? [];
  const simple = simpleOf(rule?.conditions.advanced);
  const [name, setName] = useState(rule?.name ?? '');
  const [account, setAccount] = useState(rule?.account_id ?? ALL_ACCOUNTS);
  const [priority, setPriority] = useState(String(rule?.priority ?? 100));
  const [enabled, setEnabled] = useState(rule?.enabled ?? true);
  const [from, setFrom] = useState(joinList(rule?.conditions.from));
  const [fromBlock, setFromBlock] = useState(joinList(rule?.conditions.from_block));
  const [contains, setContains] = useState(joinList(rule?.conditions.subject.contains));
  const [all, setAll] = useState(rule?.conditions.subject.all ?? false);
  const [notContains, setNotContains] = useState(joinList(rule?.conditions.subject.not_contains));
  const [regex, setRegex] = useState(rule?.conditions.subject.regex ?? '');
  const [folders, setFolders] = useState(joinList(rule?.conditions.folders));
  const [attachments, setAttachments] = useState(simple.attachments);
  const [body, setBody] = useState(simple.body);
  const [maxKb, setMaxKb] = useState(simple.maxKb);
  const [action, setAction] = useState<MailRuleAction>(rule?.action ?? 'review');
  const [ai, setAi] = useState<MailAiMode>(rule?.ai ?? 'never');
  const [retention, setRetention] = useState<RetentionMode | typeof ACCOUNT_DEFAULT>(
    rule?.retention?.mode ?? ACCOUNT_DEFAULT,
  );
  const [days, setDays] = useState(String(rule?.retention?.days ?? 30));
  const [grace, setGrace] = useState(String(rule?.retention?.grace_days ?? 7));
  const [pattern, setPattern] = useState(rule?.template?.pattern ?? '');
  const [direction, setDirection] = useState(rule?.template?.direction ?? 'expense');
  const [templateAccount, setTemplateAccount] = useState(rule?.template?.account_id ?? '');
  const [sampleFrom, setSampleFrom] = useState('');
  const [sampleSubject, setSampleSubject] = useState('');
  const [tested, setTested] = useState<MailRuleTest | null>(null);

  const advanced = (): MailCondition | undefined => {
    if (simple.other) return rule?.conditions.advanced;
    const of: MailCondition[] = [];
    if (splitList(attachments).length)
      of.push({ type: 'attachment', names: splitList(attachments) });
    if (splitList(body).length) of.push({ type: 'body_contains', terms: splitList(body) });
    if (Number(maxKb) > 0) of.push({ type: 'size_at_most', kb: Number(maxKb) });
    if (!of.length) return undefined;
    return of.length === 1 ? of[0] : { type: 'all', of };
  };

  const input = (): MailRuleInput => {
    const adv = advanced();
    return {
      ...(rule ? { id: rule.id } : {}),
      ...(account !== ALL_ACCOUNTS ? { account_id: account } : {}),
      name: name.trim(),
      priority: Number(priority) || 100,
      enabled,
      conditions: {
        folders: splitList(folders),
        from: splitList(from),
        from_block: splitList(fromBlock),
        subject: {
          contains: splitList(contains),
          all,
          not_contains: splitList(notContains),
          ...(regex.trim() ? { regex: regex.trim() } : {}),
        },
        ...(adv ? { advanced: adv } : {}),
      },
      action,
      ai,
      ...(retention !== ACCOUNT_DEFAULT
        ? {
            retention: {
              mode: retention,
              days: Number(days) || 30,
              grace_days: Number(grace) || 0,
              local: 'move',
              purge_trash: false,
            },
          }
        : {}),
      ...(pattern.trim()
        ? {
            template: {
              pattern: pattern.trim(),
              direction,
              ...(templateAccount ? { account_id: templateAccount } : {}),
            },
          }
        : {}),
    };
  };

  const runTest = () => {
    const accountId = account !== ALL_ACCOUNTS ? account : accounts[0]?.id;
    if (!accountId) return;
    testRule.mutate(
      {
        account_id: accountId,
        rule: input(),
        ...(sampleFrom.trim() || sampleSubject.trim()
          ? {
              sample: {
                folder: 'INBOX',
                folder_flags: [],
                from: sampleFrom.trim().toLowerCase(),
                original_from: null,
                to: [],
                subject: sampleSubject,
                date: null,
                size: 1000,
                attachments: [],
                body: null,
              },
            }
          : {}),
      },
      { onSuccess: setTested },
    );
  };

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      size="lg"
      resizable
      resizeId="mail-rule"
      title={rule ? t('email-regra-titulo', { nome: rule.name }) : t('email-regra-nova')}
      description={t('email-regra-descricao')}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="secondary"
            loading={testRule.isPending}
            disabled={!name.trim()}
            onClick={runTest}
          >
            {t('email-testar')}
          </Button>
          <Button
            variant="primary"
            loading={save.isPending}
            disabled={!name.trim()}
            onClick={() =>
              save.mutate(input(), {
                onSuccess: (r) => {
                  toast({ tone: 'success', title: t('email-regra-salva'), description: r.name });
                  onClose();
                },
              })
            }
          >
            {t('comum-salvar')}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-4">
        <div className="grid gap-3 @min-[640px]:grid-cols-[2fr_1.5fr_0.7fr]">
          <Field label={t('email-nome')}>
            <Input
              value={name}
              placeholder={t('email-regra-nome-exemplo')}
              onChange={(e) => setName(e.target.value)}
            />
          </Field>
          <Field label={t('email-regra-conta')}>
            <Select
              value={account}
              onValueChange={setAccount}
              options={[
                { value: ALL_ACCOUNTS, label: t('email-todas-contas') },
                ...accounts.map((a) => ({ value: a.id, label: a.name })),
              ]}
            />
          </Field>
          <Field label={t('email-regra-ordem')}>
            <Input
              value={priority}
              inputMode="numeric"
              onChange={(e) => setPriority(e.target.value)}
            />
          </Field>
        </div>
        <Switch checked={enabled} onChange={setEnabled} label={t('email-regra-ligada')} />
        <Field label={t('email-regra-de-label')} hint={t('email-regra-de-dica')}>
          <TextArea rows={2} value={from} onChange={(e) => setFrom(e.target.value)} />
        </Field>
        <Field label={t('email-regra-bloqueados')}>
          <Input value={fromBlock} onChange={(e) => setFromBlock(e.target.value)} />
        </Field>
        <div className="grid gap-3 @min-[640px]:grid-cols-2">
          <Field label={t('email-assunto-contem')} hint={t('email-assunto-contem-dica')}>
            <Input
              value={contains}
              placeholder={t('email-assunto-contem-exemplo')}
              onChange={(e) => setContains(e.target.value)}
            />
          </Field>
          <Field label={t('email-assunto-nao-contem')}>
            <Input
              value={notContains}
              placeholder={t('email-assunto-nao-contem-exemplo')}
              onChange={(e) => setNotContains(e.target.value)}
            />
          </Field>
        </div>
        <Checkbox checked={all} onChange={setAll} label={t('email-todos-termos')} />
        <Field label={t('email-assunto-regex')} hint={t('email-assunto-regex-dica')}>
          <Input value={regex} onChange={(e) => setRegex(e.target.value)} />
        </Field>
        <details className="rounded-card border border-line px-3 py-2">
          <summary className="cursor-pointer text-[13px] text-fg">{t('email-avancadas')}</summary>
          <div className="mt-3 flex flex-col gap-3">
            <Field label={t('email-so-pastas')} hint={t('email-so-pastas-dica')}>
              <Input value={folders} onChange={(e) => setFolders(e.target.value)} />
            </Field>
            {simple.other ? (
              <p className="text-[12.5px] text-dim">{t('email-avancadas-combinadas')}</p>
            ) : (
              <div className="grid gap-3 @min-[640px]:grid-cols-3">
                <Field label={t('email-anexo-nome')} hint={t('email-anexo-nome-exemplo')}>
                  <Input value={attachments} onChange={(e) => setAttachments(e.target.value)} />
                </Field>
                <Field label={t('email-corpo-contem')}>
                  <Input value={body} onChange={(e) => setBody(e.target.value)} />
                </Field>
                <Field label={t('email-ate-kb')}>
                  <Input
                    value={maxKb}
                    inputMode="numeric"
                    onChange={(e) => setMaxKb(e.target.value)}
                  />
                </Field>
              </div>
            )}
          </div>
        </details>
        <Field label={t('email-o-que-fazer')}>
          <Segmented
            label={t('email-acao-regra')}
            fill
            value={action}
            onChange={setAction}
            options={(Object.keys(ACTION_LABEL) as MailRuleAction[]).map((a) => ({
              value: a,
              label: ACTION_LABEL[a],
            }))}
          />
        </Field>
        {action === 'auto' ? (
          <p className="text-[12.5px] text-dim">{t('email-auto-aviso')}</p>
        ) : null}
        <Field label={t('email-ia')} hint={t('email-ia-dica')}>
          <Select
            value={ai}
            onValueChange={(v) => setAi(v as MailAiMode)}
            options={(Object.keys(AI_LABEL) as MailAiMode[]).map((m) => ({
              value: m,
              label: AI_LABEL[m],
            }))}
          />
        </Field>
        <div className="grid gap-3 @min-[640px]:grid-cols-3">
          <Field label={t('email-retencao')}>
            <Select
              value={retention}
              onValueChange={(v) => setRetention(v as RetentionMode | typeof ACCOUNT_DEFAULT)}
              options={[
                { value: ACCOUNT_DEFAULT, label: t('email-retencao-conta') },
                ...(Object.keys(RETENTION_LABEL) as RetentionMode[]).map((m) => ({
                  value: m,
                  label: RETENTION_LABEL[m],
                })),
              ]}
            />
          </Field>
          {retention !== ACCOUNT_DEFAULT && retention !== 'never' ? (
            <>
              <Field label={t('email-carencia')}>
                <Input
                  value={grace}
                  inputMode="numeric"
                  onChange={(e) => setGrace(e.target.value)}
                />
              </Field>
              <Field label={t('email-depois-de')}>
                <Input value={days} inputMode="numeric" onChange={(e) => setDays(e.target.value)} />
              </Field>
            </>
          ) : null}
        </div>
        <details className="rounded-card border border-line px-3 py-2">
          <summary className="cursor-pointer text-[13px] text-fg">
            {t('email-modelo-corpo')}
          </summary>
          <div className="mt-3 flex flex-col gap-3">
            <Field
              label={t('email-modelo-expressao')}
              hint={t('email-modelo-exemplo', { exemplo: TEMPLATE_EXAMPLE })}
            >
              <Input value={pattern} onChange={(e) => setPattern(e.target.value)} />
            </Field>
            <div className="grid gap-3 @min-[640px]:grid-cols-2">
              <Field label={t('email-modelo-tipo')}>
                <Select
                  value={direction}
                  onValueChange={setDirection}
                  options={[
                    { value: 'expense', label: t('email-modelo-saida') },
                    { value: 'income', label: t('email-modelo-entrada') },
                  ]}
                />
              </Field>
              <Field label={t('email-modelo-conta')}>
                <Select
                  value={templateAccount}
                  onValueChange={setTemplateAccount}
                  placeholder={t('email-modelo-escolher-revisao')}
                  options={ledgerAccounts.map((a) => ({ value: a.id, label: a.name }))}
                />
              </Field>
            </div>
          </div>
        </details>
        <details className="rounded-card border border-line px-3 py-2" open={Boolean(tested)}>
          <summary className="cursor-pointer text-[13px] text-fg">
            {t('email-testar-exemplo')}
          </summary>
          <div className="mt-3 flex flex-col gap-3">
            <div className="grid gap-3 @min-[640px]:grid-cols-2">
              <Field label={t('email-remetente')}>
                <Input
                  value={sampleFrom}
                  placeholder="faturas@nubank.com.br"
                  onChange={(e) => setSampleFrom(e.target.value)}
                />
              </Field>
              <Field label={t('email-assunto')}>
                <Input
                  value={sampleSubject}
                  placeholder={t('email-assunto-exemplo')}
                  onChange={(e) => setSampleSubject(e.target.value)}
                />
              </Field>
            </div>
            {tested ? (
              <div className="flex flex-col gap-1 text-[12.5px]">
                {tested.sample ? (
                  <p className={tested.sample.passed ? 'text-positive' : 'text-dim'}>
                    {tested.sample.passed
                      ? t('email-exemplo-passa', { motivo: tested.sample.reason })
                      : t('email-exemplo-nao-passa', { motivo: tested.sample.reason })}
                  </p>
                ) : null}
                <p className="text-dim">
                  {t('email-teste-recentes', {
                    total: tested.recent.length,
                    pegaria: tested.recent.filter((r) => r.matches).length,
                  })}
                </p>
                <ul className="flex flex-col gap-0.5">
                  {tested.recent
                    .filter((r) => r.matches)
                    .slice(0, 8)
                    .map((r) => (
                      <li key={r.message_id} className="truncate text-fg">
                        {r.subject ?? t('email-sem-assunto')}{' '}
                        <span className="text-dim">{`· ${r.sender}`}</span>
                      </li>
                    ))}
                </ul>
              </div>
            ) : (
              <p className="text-[12.5px] text-dim">{t('email-testar-nada')}</p>
            )}
          </div>
        </details>
        <InlineError error={save.error ?? testRule.error} />
      </div>
    </Dialog>
  );
}
