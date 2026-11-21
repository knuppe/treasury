/**
 * 02 · Família (docs/income.md e docs/members.md): cada membro com o cadastro, as rendas
 * (formais e informais), os holerites importados (bruto, INSS, IRRF, líquido) e as contas e
 * cartões dele (os lançamentos delas herdam o membro), mais a previsão da renda da família nos
 * próximos 12 meses, com 13º e férias. Sem renda cadastrada, um começo guiado pelo perfil do
 * questionário.
 */
import { useCallback, useMemo, useState } from 'react';

import type { AccountView } from '@/api/generated/AccountView';
import type { IncomeSource } from '@/api/generated/IncomeSource';
import type { IncomeSourceInput } from '@/api/generated/IncomeSourceInput';
import type { IncomeSourceKind } from '@/api/generated/IncomeSourceKind';
import type { Member } from '@/api/generated/Member';
import type { MemberProfile } from '@/api/generated/MemberProfile';
import type { MemberRelationship } from '@/api/generated/MemberRelationship';
import type { MemberSex } from '@/api/generated/MemberSex';
import type { PayslipPreview } from '@/api/generated/PayslipPreview';
import { useAccounts, useUpdateAccount } from '@/api/hooks/accounts';
import { useCreateMember, useMembers, useUpdateMember } from '@/api/hooks/household';
import { useCountryInfo } from '@/api/hooks/country';
import { useProfile } from '@/api/hooks/profile';
import {
  previewPayslip,
  useCreateIncomeSource,
  useDeleteIncomeSource,
  useImportPayslip,
  useIncomeSummary,
  usePayslips,
  useUpdateIncomeSource,
} from '@/api/hooks/income';
import { ChartFrame, EChart, type ChartTheme } from '@/charts';
import {
  Button,
  DateInput,
  Dialog,
  Field,
  Icon,
  Input,
  InstrumentCard,
  KeyValue,
  Money,
  MoneyInput,
  Select,
  TechTable,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { MemberLifeBar } from '@/features/longevity/MemberLifeBar';
import { sexOptions } from '@/features/longevity/sex';
import { fileToBase64 } from '@/features/transactions/ui/hooks';
import {
  Checkbox,
  FileInput,
  InlineError,
  QueryState,
  Switch,
} from '@/features/transactions/ui/controls';
import { ACCOUNT_KIND } from '@/features/transactions/ui/labels';
import { t } from '@/i18n';
import { formatLocale, formatMonthShort, monthName } from '@/lib/format';

import { incomeOption, incomeRows } from './incomeChart';
import { TerminationDialog } from './TerminationDialog';
import { MemberDietField, useMemberDietDraft } from './MemberDietField';
import { cap, type IncomeTerms, useIncomeTerms } from './terms';

const RELATIONS: readonly MemberRelationship[] = [
  'titular',
  'conjuge',
  'filho',
  'pai_mae',
  'outro',
];

function relationLabel(r: MemberRelationship): string {
  switch (r) {
    case 'titular':
      return t('familia-relacao-titular');
    case 'conjuge':
      return t('familia-relacao-conjuge');
    case 'filho':
      return t('familia-relacao-filho');
    case 'pai_mae':
      return t('familia-relacao-pai-mae');
    case 'outro':
      return t('familia-relacao-outro');
  }
}

/** De onde vem a previsão de uma renda (o último holerite, os valores informados, a média). */
function basisLabel(basis: string | undefined, terms: IncomeTerms): string | undefined {
  switch (basis ?? 'none') {
    case 'payslip':
      return t('familia-base-holerite', { holerite: terms.payslip });
    case 'declared':
      return t('familia-base-informados');
    case 'average':
      return t('familia-base-media');
    case 'none':
      return t('familia-base-nenhum');
    default:
      return undefined;
  }
}

/** Nomes numa lista ("Ana e Bruno"), no idioma do app. */
function nameList(names: string[]): string {
  const locale = formatLocale();
  return new Intl.ListFormat(locale === 'xx' ? 'pt-BR' : locale, { type: 'conjunction' }).format(
    names,
  );
}

// ── Cadastro do membro ───────────────────────────────────────────

function ProfileDialog({ member, onClose }: { member: Member; onClose: () => void }) {
  const terms = useIncomeTerms();
  const save = useUpdateMember();
  const diet = useMemberDietDraft(member.id);
  const [p, setP] = useState<MemberProfile>(member.profile);
  const set = <K extends keyof MemberProfile>(k: K, v: MemberProfile[K]) =>
    setP((x) => ({ ...x, [k]: v }));
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      title={t('familia-cadastro-de', { nome: member.name })}
      size="md"
    >
      <div className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-2">
          <Field label={t('familia-relacao')}>
            <Select
              value={p.relationship ?? ''}
              onValueChange={(v) => set('relationship', (v || null) as MemberRelationship | null)}
              options={RELATIONS.map((value) => ({ value, label: relationLabel(value) }))}
              placeholder={t('familia-escolha')}
            />
          </Field>
          <Field label={t('familia-nascimento')}>
            <DateInput value={p.birth_date} onValueChange={(v) => set('birth_date', v)} />
          </Field>
          <Field label={t('familia-sexo')} hint={t('familia-sexo-dica')}>
            <Select
              value={p.sex ?? ''}
              onValueChange={(v) => set('sex', (v || null) as MemberSex | null)}
              options={sexOptions()}
              placeholder={t('familia-sexo-nao-informado')}
            />
          </Field>
          <Field label={t('familia-ocupacao')}>
            <Input
              value={p.occupation ?? ''}
              onChange={(e) => set('occupation', e.target.value || null)}
            />
          </Field>
          <Field label={t('familia-observacoes')}>
            <Input value={p.notes ?? ''} onChange={(e) => set('notes', e.target.value || null)} />
          </Field>
        </div>
        <Switch
          checked={p.irpf_dependent}
          onChange={(v) => set('irpf_dependent', v)}
          label={t('familia-dependente-ir')}
          description={terms.dependentHint}
        />
        <MemberDietField value={diet.value} onToggle={diet.toggle} />
        <p className="text-[12px] text-dim">{t('familia-sem-cpf')}</p>
        {/* A barra do membro pelo cadastro salvo (só com a linha da vida ligada). */}
        <MemberLifeBar memberId={member.id} />
        <InlineError error={save.error ?? diet.mutation.error} />
        <div className="flex justify-end gap-2 border-t border-line pt-3">
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={save.isPending || diet.mutation.isPending}
            onClick={() =>
              // Os erros aparecem no InlineError acima.
              void Promise.all([save.mutateAsync({ id: member.id, profile: p }), diet.save()]).then(
                () => {
                  toast({ tone: 'success', title: t('familia-cadastro-salvo') });
                  onClose();
                },
                () => undefined,
              )
            }
          >
            {t('comum-salvar')}
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

// ── Renda ───────────────────────────────────────────────────────

function sourceInput(
  memberId: string,
  s?: IncomeSource,
  kind?: IncomeSourceKind,
  extraPay = true,
): IncomeSourceInput {
  return {
    member_id: memberId,
    name: s?.name ?? '',
    kind: s?.kind ?? kind ?? 'clt',
    ...(s?.employer ? { employer: s.employer } : {}),
    ...(s?.employer_cnpj ? { employer_cnpj: s.employer_cnpj } : {}),
    ...(s?.gross_monthly != null ? { gross_monthly: s.gross_monthly } : {}),
    ...(s?.net_monthly != null ? { net_monthly: s.net_monthly } : {}),
    variable: s?.variable ?? kind === 'informal',
    ...(s?.pay_day ? { pay_day: s.pay_day } : {}),
    has_thirteenth:
      s?.has_thirteenth ??
      (extraPay && ['clt', 'public_servant', 'retirement', 'pension'].includes(kind ?? 'clt')),
    ...(s?.vacation_month ? { vacation_month: s.vacation_month } : {}),
    dependents: s?.dependents ?? 0,
    ...(s?.start_date ? { start_date: s.start_date } : {}),
    ...(s?.end_date ? { end_date: s.end_date } : {}),
    ...(s?.notes ? { notes: s.notes } : {}),
  };
}

function SourceDialog({
  memberId,
  source,
  kind,
  onClose,
}: {
  memberId: string;
  source?: IncomeSource;
  kind?: IncomeSourceKind;
  onClose: () => void;
}) {
  const create = useCreateIncomeSource();
  const update = useUpdateIncomeSource();
  const terms = useIncomeTerms();
  const [i, setI] = useState<IncomeSourceInput>(
    sourceInput(memberId, source, kind, terms.christmas != null),
  );
  const set = <K extends keyof IncomeSourceInput>(k: K, v: IncomeSourceInput[K]) =>
    setI((x) => ({ ...x, [k]: v }));
  const formal = ['clt', 'public_servant', 'retirement', 'pension'].includes(i.kind);
  const save = () => {
    const done = () => {
      toast({
        tone: 'success',
        title: source ? t('familia-renda-alterada') : t('familia-renda-cadastrada'),
      });
      onClose();
    };
    if (source) update.mutate({ id: source.id, source: i }, { onSuccess: done });
    else create.mutate(i, { onSuccess: done });
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      title={source ? t('familia-editar-nome', { nome: source.name }) : t('familia-nova-renda')}
      description={t('familia-renda-descricao', { holerite: terms.payslip })}
      size="lg"
    >
      <div className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-2">
          <Field label={t('familia-nome')}>
            <Input
              value={i.name}
              onChange={(e) => set('name', e.target.value)}
              placeholder={t('familia-nome-exemplo')}
            />
          </Field>
          <Field label={t('familia-tipo')}>
            <Select
              value={i.kind}
              onValueChange={(v) => {
                const k = v as IncomeSourceKind;
                setI((x) => ({
                  ...x,
                  kind: k,
                  has_thirteenth:
                    terms.christmas != null &&
                    ['clt', 'public_servant', 'retirement', 'pension'].includes(k),
                  variable: k === 'informal',
                }));
              }}
              options={Object.entries(terms.kind).map(([value, label]) => ({ value, label }))}
            />
          </Field>
          <Field
            label={t('familia-bruto-mes')}
            hint={
              formal ? t('familia-bruto-dica', { descontos: terms.both }) : t('familia-opcional')
            }
          >
            <MoneyInput
              value={i.gross_monthly ?? null}
              onValueChange={(v) => set('gross_monthly', v ?? undefined)}
            />
          </Field>
          <Field label={t('familia-liquido-mes')} hint={t('familia-liquido-dica')}>
            <MoneyInput
              value={i.net_monthly ?? null}
              onValueChange={(v) => set('net_monthly', v ?? undefined)}
            />
          </Field>
          {formal || i.kind === 'pj' ? (
            <>
              <Field label={t('familia-empregador-fonte')}>
                <Input
                  value={i.employer ?? ''}
                  onChange={(e) => set('employer', e.target.value || undefined)}
                />
              </Field>
              <Field
                label={terms.employerId ?? t('familia-registro-empresa')}
                hint={t('familia-registro-dica', { holerites: terms.payslips })}
              >
                <Input
                  className="num"
                  value={i.employer_cnpj ?? ''}
                  onChange={(e) => set('employer_cnpj', e.target.value || undefined)}
                  placeholder={terms.country === 'BR' ? '00.000.000/0001-00' : undefined}
                />
              </Field>
            </>
          ) : null}
          <Field label={t('familia-dia-pagamento')}>
            <Input
              numeric
              inputMode="numeric"
              value={i.pay_day ? String(i.pay_day) : ''}
              onChange={(e) =>
                set('pay_day', Number(e.target.value.replace(/\D/g, '')) || undefined)
              }
            />
          </Field>
          {formal && terms.vacation ? (
            <Field
              label={t('familia-mes-ferias')}
              hint={t('familia-mes-ferias-dica', { ferias: terms.vacation })}
            >
              <Select
                value={i.vacation_month ? String(i.vacation_month) : ''}
                onValueChange={(v) => set('vacation_month', v ? Number(v) : undefined)}
                options={Array.from({ length: 12 }, (_, k) => ({
                  value: String(k + 1),
                  label: monthName(k + 1),
                }))}
                placeholder={t('familia-sem-ferias')}
              />
            </Field>
          ) : null}
          <Field label={t('familia-comeca-em')}>
            <DateInput
              value={i.start_date ?? null}
              onValueChange={(v) => set('start_date', v ?? undefined)}
            />
          </Field>
          <Field label={t('familia-termina-em')} hint={t('familia-termina-dica')}>
            <DateInput
              value={i.end_date ?? null}
              onValueChange={(v) => set('end_date', v ?? undefined)}
            />
          </Field>
        </div>
        <div className="flex flex-col gap-2">
          {terms.christmas ? (
            <Switch
              checked={i.has_thirteenth ?? false}
              onChange={(v) => set('has_thirteenth', v)}
              label={t('familia-recebe', { extra: terms.christmas })}
            />
          ) : null}
          <Switch
            checked={i.variable ?? false}
            onChange={(v) => set('variable', v)}
            label={t('familia-valor-variavel')}
            description={t('familia-valor-variavel-dica')}
          />
        </div>
        <InlineError error={create.error ?? update.error} />
        <div className="flex justify-end gap-2 border-t border-line pt-3">
          <Button variant="ghost" onClick={onClose}>
            {t('comum-cancelar')}
          </Button>
          <Button
            variant="primary"
            loading={create.isPending || update.isPending}
            disabled={!i.name.trim()}
            onClick={save}
          >
            {t('comum-salvar')}
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

// ── Holerite ────────────────────────────────────────────────────

function PayslipDialog({ memberId, onClose }: { memberId: string; onClose: () => void }) {
  const terms = useIncomeTerms();
  const save = useImportPayslip();
  const [file, setFile] = useState<{ filename: string; content_base64: string } | null>(null);
  const [password, setPassword] = useState('');
  const [month, setMonth] = useState('');
  const [preview, setPreview] = useState<PayslipPreview | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const args = () => ({
    files: file ? [file] : [],
    member_id: memberId,
    ...(password ? { password } : {}),
    ...(/^\d{4}-\d{2}$/.test(month) ? { reference_month: month } : {}),
  });
  const read = async (f?: { filename: string; content_base64: string }) => {
    const fl = f ?? file;
    if (!fl) return;
    setBusy(true);
    setError(null);
    try {
      setPreview(await previewPayslip({ ...args(), files: [fl] }));
    } catch (e) {
      setError(e);
      setPreview(null);
    } finally {
      setBusy(false);
    }
  };
  const d = preview?.data;
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      title={t('familia-importar-holerite', { holerite: terms.payslip })}
      description={t('familia-importar-descricao', { holerite: terms.payslip })}
      size="lg"
    >
      <div className="flex flex-col gap-4">
        <div className="grid gap-3 sm:grid-cols-[1fr_auto_auto]">
          <Field label={t('familia-holerite-pdf', { holerite: cap(terms.payslip) })}>
            <FileInput
              accept=".pdf,application/pdf"
              disabled={busy}
              onChange={(e) => {
                const f = e.target.files?.[0];
                if (!f) return;
                void fileToBase64(f).then((c) => {
                  const fl = { filename: f.name, content_base64: c };
                  setFile(fl);
                  void read(fl);
                });
              }}
            />
          </Field>
          <Field label={t('familia-senha-se-tiver')}>
            <Input
              type="password"
              autoComplete="off"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="w-36"
            />
          </Field>
          <div className="self-end">
            <Button variant="secondary" disabled={!file || busy} onClick={() => void read()}>
              {t('familia-ler-de-novo')}
            </Button>
          </div>
        </div>
        {busy ? (
          <p className="label-caps text-accent">
            {t('familia-lendo', { holerite: terms.payslip })}
          </p>
        ) : null}
        <InlineError error={error ?? save.error} />
        {d && preview ? (
          <>
            <KeyValue
              items={[
                {
                  label: t('familia-competencia'),
                  value: d.reference_month ? formatMonthShort(d.reference_month) : '—',
                },
                { label: t('familia-empregador'), value: d.employer ?? '—' },
                {
                  label: t('familia-renda'),
                  value:
                    preview.source_name ??
                    (preview.creates_source
                      ? t('familia-renda-nova', { tipo: terms.kind.clt })
                      : '—'),
                },
                { label: t('familia-bruto'), value: <Money cents={d.gross} /> },
                { label: terms.contribution, value: <Money cents={d.social_security} /> },
                { label: terms.tax, value: <Money cents={d.income_tax} /> },
                {
                  label: t('familia-outros-descontos'),
                  value: <Money cents={d.other_deductions} />,
                },
                { label: t('familia-liquido'), value: <Money cents={d.net} /> },
                {
                  label: t('familia-salario-conta'),
                  value: preview.txn
                    ? `${preview.txn.account_name}, ${preview.txn.date}`
                    : t('familia-nao-achado'),
                },
              ]}
            />
            {!d.reference_month ? (
              <Field
                label={t('familia-competencia-mes')}
                hint={t('familia-sem-mes', { holerite: terms.payslip })}
              >
                <Input
                  value={month}
                  onChange={(e) => setMonth(e.target.value)}
                  placeholder="2026-09"
                  className="w-32"
                />
              </Field>
            ) : null}
            {preview.warnings.length ? (
              <ul className="flex flex-col gap-1 rounded-card border border-warning/50 bg-warning/5 px-3 py-2 text-[12.5px] text-warning">
                {preview.warnings.map((w) => (
                  <li key={w} className="first-letter:uppercase">
                    {w}.
                  </li>
                ))}
              </ul>
            ) : null}
            {preview.duplicate ? (
              <p className="text-[12.5px] text-warning">
                {t('familia-ja-importado', { holerite: terms.payslip })}
              </p>
            ) : null}
            <div className="flex justify-end gap-2 border-t border-line pt-3">
              <Button variant="ghost" onClick={onClose}>
                {t('comum-cancelar')}
              </Button>
              <Button
                variant="primary"
                loading={save.isPending}
                onClick={() =>
                  save.mutate(
                    { ...args(), replace: preview.duplicate },
                    {
                      onSuccess: () => {
                        toast({
                          tone: 'success',
                          title: t('familia-holerite-importado', {
                            holerite: cap(terms.payslip),
                          }),
                        });
                        onClose();
                      },
                    },
                  )
                }
              >
                {t('familia-gravar', { holerite: terms.payslip })}
              </Button>
            </div>
          </>
        ) : null}
      </div>
    </Dialog>
  );
}

// ── Painel ──────────────────────────────────────────────────────

// ── Contas e cartões de cada membro ─────────────────────────────

function AccountsDialog({
  member,
  accounts,
  onClose,
}: {
  member: Member;
  accounts: readonly AccountView[];
  onClose: () => void;
}) {
  const update = useUpdateAccount();
  const names = new Map((useMembers().data ?? []).map((m) => [m.id, m.name]));
  const [pending, setPending] = useState<string | null>(null);
  const toggle = (a: AccountView, on: boolean) => {
    const ids = on
      ? [...new Set([...a.member_ids, member.id])]
      : a.member_ids.filter((x) => x !== member.id);
    setPending(a.id);
    update.mutate({ id: a.id, member_ids: ids }, { onSettled: () => setPending(null) });
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      title={t('familia-contas-de', { nome: member.name })}
      description={t('familia-contas-descricao')}
      size="md"
    >
      <div className="flex flex-col gap-2">
        {accounts.map((a) => {
          const on = a.member_ids.includes(member.id);
          const otherNames = a.member_ids
            .filter((x) => x !== member.id)
            .map((x) => names.get(x) ?? '?');
          const others = otherNames.length ? nameList(otherNames) : '';
          return (
            <div key={a.id} className="flex items-center gap-3">
              <Checkbox
                checked={on}
                disabled={pending === a.id}
                onChange={(v) => toggle(a, v)}
                label={a.name}
              />
              <span className="text-[12px] text-dim">
                {ACCOUNT_KIND[a.kind] ?? a.kind}
                {others
                  ? ` · ${on ? t('familia-conjunta-com', { nomes: others }) : t('familia-de-outros', { nomes: others })}`
                  : ''}
              </span>
            </div>
          );
        })}
        <InlineError error={update.error} />
        <div className="flex justify-end border-t border-line pt-3">
          <Button variant="primary" onClick={onClose}>
            {t('familia-pronto')}
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

// ── Começo guiado pelo perfil ───────────────────────────────────

type Start = { kind: IncomeSourceKind; text: string; tip: string };

/** O começo pela resposta de renda do questionário, nos termos do país da família. */
function startFor(income: string | null, terms: IncomeTerms): Start {
  // O que o app calcula: "INSS e IRRF, 13º salário e terço de férias" (lista no idioma do app).
  const items = [terms.both, terms.christmas, terms.vacation].filter((x): x is string =>
    Boolean(x),
  );
  const fixed = terms.payroll
    ? t('familia-inicio-fixa-calcula', { holerite: terms.payslip, itens: nameList(items) })
    : t('familia-inicio-fixa-recibo', { holerite: terms.payslip });
  const benefits =
    terms.country === 'BR'
      ? t('familia-beneficios-br')
      : terms.country === 'PT'
        ? t('familia-beneficios-pt')
        : terms.country === 'US'
          ? t('familia-beneficios-us')
          : t('familia-beneficios-outros');
  switch (income) {
    case 'fixa':
      return { kind: 'clt', text: fixed, tip: t('familia-dica-fixa') };
    case 'variavel':
      return {
        kind: 'informal',
        text: t('familia-inicio-variavel'),
        tip: t('familia-dica-variavel'),
      };
    case 'beneficios':
      return {
        kind: 'benefit',
        text: t('familia-inicio-beneficios', { exemplos: benefits }),
        tip: t('familia-dica-beneficios'),
      };
    case 'mista':
      return { kind: 'clt', text: t('familia-inicio-mista'), tip: t('familia-dica-mista') };
    default:
      return { kind: 'clt', text: t('familia-inicio-padrao'), tip: t('familia-dica-padrao') };
  }
}

function StartCard({ onStart }: { onStart: (kind: IncomeSourceKind) => void }) {
  const income = useProfile().data?.income ?? null;
  const terms = useIncomeTerms();
  const info = useCountryInfo();
  const s = startFor(income, terms);
  // Fora do Brasil, as boas práticas de lá (docs/countries.md).
  const tips = info && info.code !== 'BR' ? info.tips.slice(0, 3) : [];
  return (
    <section
      aria-label={t('familia-comece')}
      className="flex flex-col gap-2 rounded-card border border-accent/50 bg-accent-soft px-4 py-3.5"
    >
      <h2 className="text-[15px] font-medium text-fg">{t('familia-comece')}</h2>
      <p className="text-[13px] text-fg">{s.text}</p>
      <p className="text-[12.5px] text-dim">{s.tip}</p>
      {tips.length ? (
        <ul
          aria-label={t('familia-boas-praticas-em', { pais: info!.name })}
          className="flex list-disc flex-col gap-1 pl-4 text-[12.5px] text-dim"
        >
          {tips.map((tip) => (
            <li key={tip}>{tip}</li>
          ))}
        </ul>
      ) : null}
      <div>
        <Button
          variant="primary"
          icon={<Icon name="plus" size={12} />}
          onClick={() => onStart(s.kind)}
        >
          {t('familia-cadastrar-primeira')}
        </Button>
      </div>
    </section>
  );
}

function NewMemberForm() {
  const create = useCreateMember();
  const [name, setName] = useState('');
  return (
    <form
      className="flex flex-wrap items-end gap-2"
      onSubmit={(e) => {
        e.preventDefault();
        if (!name.trim()) return;
        create.mutate(
          { name: name.trim() },
          {
            onSuccess: () => {
              toast({
                tone: 'success',
                title: t('familia-membro-adicionado'),
                description: name.trim(),
              });
              setName('');
            },
          },
        );
      }}
    >
      <Field label={t('familia-novo-membro')}>
        <Input
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder={t('familia-nome')}
          className="w-56"
        />
      </Field>
      <Button type="submit" variant="secondary" loading={create.isPending} disabled={!name.trim()}>
        {t('familia-adicionar')}
      </Button>
      <InlineError error={create.error} />
    </form>
  );
}

export function IncomePanel() {
  const terms = useIncomeTerms();
  const q = useIncomeSummary(12);
  const members = useMembers().data ?? [];
  const slips = usePayslips();
  const remove = useDeleteIncomeSource();
  const [profile, setProfile] = useState<Member | null>(null);
  const [editing, setEditing] = useState<{ memberId: string; source?: IncomeSource } | null>(null);
  const [importFor, setImportFor] = useState<string | null>(null);
  const [accountsFor, setAccountsFor] = useState<Member | null>(null);
  const [startKind, setStartKind] = useState<IncomeSourceKind | null>(null);
  const [dismissal, setDismissal] = useState<IncomeSource | null>(null);
  const brazil = (useCountryInfo()?.code ?? 'BR') === 'BR';
  const accounts = (useAccounts().data ?? []).filter((a) => !a.archived);
  const update = useUpdateAccount();
  const s = q.data;
  const noIncome = s ? s.members.every((m) => m.sources.length === 0) : false;
  const unassigned = accounts.filter((a) => a.member_ids.length === 0);

  const rows = useMemo(() => (s ? incomeRows(s) : []), [s]);
  const baseMonth = s?.forecast_base_month ?? null;
  const option = useCallback(
    (theme: ChartTheme) =>
      s
        ? incomeOption(theme, s, rows, { taxes: terms.both, payslip: terms.payslip, baseMonth })
        : {},
    [s, rows, terms.both, terms.payslip, baseMonth],
  );

  return (
    <QueryState loading={q.isPending} error={q.error} onRetry={() => void q.refetch()}>
      {s ? (
        <div className="flex flex-col gap-4">
          {noIncome && members[0] ? (
            <StartCard
              onStart={(k) => {
                setStartKind(k);
                setEditing({ memberId: members.find((m) => m.is_primary)?.id ?? members[0]!.id });
              }}
            />
          ) : null}
          <div className="grid gap-3 @min-[560px]:grid-cols-2 @min-[1100px]:grid-cols-4">
            {[
              { label: t('familia-bruto-12m'), cents: s.next.gross },
              { label: t('familia-liquido-12m'), cents: s.next.net },
              {
                label: t('familia-descontos-previstos', { descontos: terms.both }),
                cents: s.next.social_security + s.next.income_tax,
              },
              {
                label: t('familia-impostos-holerites', { holerites: terms.payslips }),
                cents: s.last_12m.social_security + s.last_12m.income_tax,
              },
            ].map((c) => (
              <div
                key={c.label}
                className="flex flex-col gap-1 rounded-card border border-line bg-surface px-4 py-3"
              >
                <span className="text-[12.5px] text-dim">{c.label}</span>
                <Money cents={c.cents} currency={s.currency} className="text-[22px]" />
              </div>
            ))}
          </div>
          <ChartFrame
            index="02.R1"
            title={s.actual.length ? t('familia-renda-real-prevista') : t('familia-renda-prevista')}
            source={
              baseMonth
                ? t('familia-renda-fonte-base', {
                    holerite: terms.payslip,
                    holerites: terms.payslips,
                    mes: formatMonthShort(baseMonth),
                    descontos: terms.both,
                  })
                : t('familia-renda-fonte', { holerite: terms.payslip, descontos: terms.both })
            }
            chart={
              <EChart
                option={option}
                height={260}
                ariaLabel={
                  s.actual.length ? t('familia-renda-real-prevista') : t('familia-renda-prevista')
                }
              />
            }
            table={
              <TechTable
                label={t('familia-renda-prevista')}
                prefsId="Renda prevista por mês"
                dense
                rows={rows.filter((r) => r.kind !== 'empty')}
                getRowId={(r) => r.month}
                columns={[
                  {
                    key: 'month',
                    header: t('familia-mes'),
                    cell: (r) => formatMonthShort(r.month),
                  },
                  {
                    key: 'kind',
                    header: t('familia-renda-tipo'),
                    cell: (r) =>
                      r.kind === 'real' ? t('familia-renda-real') : t('familia-renda-previsto'),
                  },
                  {
                    key: 'gross',
                    header: t('familia-bruto'),
                    numeric: true,
                    cell: (r) => <Money cents={r.value.gross} />,
                  },
                  {
                    key: 'tax',
                    header: terms.both,
                    numeric: true,
                    cell: (r) => <Money cents={r.value.social_security + r.value.income_tax} />,
                  },
                  {
                    key: 'net',
                    header: t('familia-liquido'),
                    numeric: true,
                    cell: (r) => <Money cents={r.value.net} />,
                  },
                  {
                    key: 'diff',
                    header: t('familia-renda-diferenca-coluna'),
                    numeric: true,
                    cell: (r) =>
                      r.kind === 'real' && r.expected ? (
                        <Money cents={r.value.net - r.expected.net} tone="auto" />
                      ) : (
                        ''
                      ),
                  },
                  {
                    key: 'extras',
                    header: t('familia-extras'),
                    cell: (r) => r.value.extras.join(', '),
                  },
                ]}
              />
            }
          />
          {s.members.map((mi) => {
            const m = members.find((x) => x.id === mi.member_id);
            const prof = m?.profile;
            return (
              <InstrumentCard
                key={mi.member_id}
                index="02.R"
                title={mi.member_name}
                actions={
                  <span className="flex gap-1">
                    {m ? (
                      <Button size="sm" variant="ghost" onClick={() => setProfile(m)}>
                        {t('familia-cadastro')}
                      </Button>
                    ) : null}
                    <Button size="sm" variant="ghost" onClick={() => setImportFor(mi.member_id)}>
                      {t('familia-importar-holerite', { holerite: terms.payslip })}
                    </Button>
                    <Button
                      size="sm"
                      variant="secondary"
                      icon={<Icon name="plus" size={12} />}
                      onClick={() => setEditing({ memberId: mi.member_id })}
                    >
                      {t('familia-nova-renda')}
                    </Button>
                  </span>
                }
                padded={false}
              >
                <p className="px-4 py-2 text-[12.5px] text-dim">
                  {[
                    prof?.relationship ? relationLabel(prof.relationship) : null,
                    prof?.occupation,
                    prof?.irpf_dependent ? t('familia-dependente-ir-curto') : null,
                  ]
                    .filter(Boolean)
                    .join(' · ') || t('familia-cadastro-incompleto')}
                  {mi.effective_tax_rate != null
                    ? ` · ${t('familia-aliquota-efetiva', { aliquota: mi.effective_tax_rate * 100 })}`
                    : ''}
                </p>
                <div className="flex flex-wrap items-center gap-2 border-t border-line/60 px-4 py-2 text-[12.5px]">
                  <span className="text-dim">{t('familia-contas-cartoes')}</span>
                  {accounts.filter((a) => a.member_ids.includes(mi.member_id)).length ? (
                    accounts
                      .filter((a) => a.member_ids.includes(mi.member_id))
                      .map((a) => (
                        <span
                          key={a.id}
                          className="rounded-control border border-line px-1.5 py-px text-fg"
                          title={
                            a.member_ids.length > 1 ? t('familia-conta-conjunta-dica') : undefined
                          }
                        >
                          {a.name}
                          {a.member_ids.length > 1 ? ` ${t('familia-conjunta')}` : ''}
                        </span>
                      ))
                  ) : (
                    <span className="text-dim">{t('familia-nenhuma')}</span>
                  )}
                  {m ? (
                    <Button size="sm" variant="ghost" onClick={() => setAccountsFor(m)}>
                      {t('familia-escolher')}
                    </Button>
                  ) : null}
                </div>
                <TechTable
                  label={t('familia-rendas-de', { nome: mi.member_name })}
                  prefsId={`Rendas de ${mi.member_name}`}
                  dense
                  rows={mi.sources}
                  getRowId={(x) => x.id}
                  empty={<p className="p-4 text-[13px] text-dim">{t('familia-nenhuma-renda')}</p>}
                  columns={[
                    {
                      key: 'name',
                      header: t('familia-renda'),
                      cell: (x) => (
                        <span className="flex flex-col">
                          <span className="text-fg">{x.name}</span>
                          <span className="text-[11.5px] text-dim">
                            {`${terms.kind[x.kind]} · `}
                            <span className={cx(x.formal ? 'text-accent' : '')}>
                              {x.formal ? t('familia-formal') : t('familia-informal')}
                            </span>
                            {x.last_payslip
                              ? ` · ${t('familia-holerite-de', {
                                  holerite: terms.payslip,
                                  mes: formatMonthShort(x.last_payslip),
                                })}`
                              : ''}
                          </span>
                        </span>
                      ),
                    },
                    {
                      key: 'gross',
                      header: t('familia-bruto'),
                      numeric: true,
                      cell: (x) =>
                        x.gross_monthly != null ? <Money cents={x.gross_monthly} /> : '—',
                    },
                    {
                      key: 'net',
                      header: t('familia-liquido'),
                      numeric: true,
                      cell: (x) => (x.net_monthly != null ? <Money cents={x.net_monthly} /> : '—'),
                    },
                    {
                      key: 'basis',
                      header: t('familia-previsao'),
                      cell: (x) => {
                        const p = mi.projection.find((y) => y.source_id === x.id);
                        return (
                          <span className="text-[12px] text-dim">
                            {basisLabel(p?.basis, terms)}
                          </span>
                        );
                      },
                    },
                    {
                      key: 'actions',
                      header: '',
                      cell: (x) => (
                        <span className="flex justify-end gap-1">
                          {brazil && x.kind === 'clt' ? (
                            <Button size="sm" variant="ghost" onClick={() => setDismissal(x)}>
                              {t('familia-rescisao-botao')}
                            </Button>
                          ) : null}
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() => setEditing({ memberId: mi.member_id, source: x })}
                          >
                            {t('comum-editar')}
                          </Button>
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() => remove.mutate({ id: x.id })}
                          >
                            {t('comum-apagar')}
                          </Button>
                        </span>
                      ),
                    },
                  ]}
                />
              </InstrumentCard>
            );
          })}
          {unassigned.length ? (
            <InstrumentCard index="02.C" title={t('familia-sem-membro')}>
              <div className="flex flex-col gap-2">
                <p className="text-[12.5px] text-dim">{t('familia-sem-membro-explicacao')}</p>
                {unassigned.map((a) => (
                  <div key={a.id} className="flex flex-wrap items-center gap-3">
                    <span className="min-w-40 text-[13px] text-fg">{a.name}</span>
                    <span className="text-[12px] text-dim">{ACCOUNT_KIND[a.kind] ?? a.kind}</span>
                    <div className="w-48">
                      <Select
                        aria-label={t('familia-membro-de', { nome: a.name })}
                        value=""
                        onValueChange={(v) => v && update.mutate({ id: a.id, member_ids: [v] })}
                        options={members.map((m) => ({ value: m.id, label: m.name }))}
                        placeholder={t('familia-da-familia')}
                      />
                    </div>
                  </div>
                ))}
                <InlineError error={update.error} />
              </div>
            </InstrumentCard>
          ) : null}
          <NewMemberForm />
          {slips.data?.length ? (
            <InstrumentCard
              index="02.R2"
              title={t('familia-holerites-importados', { holerites: cap(terms.payslips) })}
              padded={false}
            >
              <TechTable
                label={t('familia-holerites-importados', { holerites: cap(terms.payslips) })}
                prefsId="Holerites importados"
                dense
                rows={slips.data}
                getRowId={(p) => p.id}
                columns={[
                  {
                    key: 'month',
                    header: t('familia-competencia'),
                    cell: (p) => formatMonthShort(p.reference_month),
                  },
                  {
                    key: 'kind',
                    header: t('holerite-tipo-coluna'),
                    cell: (p) =>
                      p.kind === 'plr' || p.kind === 'other' ? (
                        <span title={t('holerite-tipo-fora-previsao')}>
                          {t(`holerite-tipo-${p.kind}`)}
                        </span>
                      ) : (
                        t(`holerite-tipo-${p.kind}`)
                      ),
                  },
                  { key: 'member', header: t('familia-membro'), cell: (p) => p.member_name },
                  { key: 'source', header: t('familia-renda'), cell: (p) => p.source_name },
                  {
                    key: 'gross',
                    header: t('familia-bruto'),
                    numeric: true,
                    cell: (p) => <Money cents={p.gross} />,
                  },
                  {
                    key: 'inss',
                    header: terms.contribution,
                    numeric: true,
                    cell: (p) => <Money cents={p.social_security} />,
                  },
                  {
                    key: 'irrf',
                    header: terms.tax,
                    numeric: true,
                    cell: (p) => <Money cents={p.income_tax} />,
                  },
                  {
                    key: 'net',
                    header: t('familia-liquido'),
                    numeric: true,
                    cell: (p) => <Money cents={p.net} />,
                  },
                  {
                    key: 'txn',
                    header: t('familia-na-conta'),
                    cell: (p) => (p.txn_id ? t('familia-ligado') : '—'),
                  },
                ]}
              />
            </InstrumentCard>
          ) : null}
          <ul className="flex flex-col gap-1 text-[12px] text-dim">
            {s.notes.map((n) => (
              <li key={n}>{n}</li>
            ))}
          </ul>
        </div>
      ) : null}
      {profile ? <ProfileDialog member={profile} onClose={() => setProfile(null)} /> : null}
      {editing ? (
        <SourceDialog
          memberId={editing.memberId}
          source={editing.source}
          {...(startKind ? { kind: startKind } : {})}
          onClose={() => (setEditing(null), setStartKind(null))}
        />
      ) : null}
      {importFor ? <PayslipDialog memberId={importFor} onClose={() => setImportFor(null)} /> : null}
      {accountsFor ? (
        <AccountsDialog
          member={accountsFor}
          accounts={accounts}
          onClose={() => setAccountsFor(null)}
        />
      ) : null}
      {dismissal ? (
        <TerminationDialog source={dismissal} onClose={() => setDismissal(null)} />
      ) : null}
    </QueryState>
  );
}
