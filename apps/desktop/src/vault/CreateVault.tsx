import { useRef, useState, type FormEvent } from 'react';
import { flushSync } from 'react-dom';

import { useCountryChoices } from '@/api/hooks/country';
import type { VaultStatus } from '@/api/vault';
import { useCreateVault } from '@/api/vaultQueries';
import {
  Button,
  Field,
  Icon,
  IconButton,
  Input,
  InstrumentCard,
  PasswordInput,
  toast,
} from '@/design/components';
import { cx } from '@/design/cx';
import { CountryPicker, supportText } from '@/features/country/CountryPicker';
import { DemoQuick } from '@/features/demo/DemoQuick';
import { t } from '@/i18n';
import { estimatePasswordStrength } from '@/lib/passwordStrength';

import { FormError } from './FormError';
import { StrengthMeter } from './StrengthMeter';
import { validateNewPassword } from './validation';
import { VaultLayout } from './VaultLayout';
import { OtherVaults } from './VaultSwitcher';

interface Member {
  key: number;
  name: string;
}

/** Enquanto a lista não chega: o Brasil, como sempre foi. */
const FALLBACK = [{ code: 'BR', name: 'Brasil', currency: 'BRL', support: 'full' as const }];

type Choice = 'real' | 'demo';

/**
 * Criar um cofre (primeira execução ou "Novo cofre"): primeiro a escolha entre o cofre real, com
 * senha mestra, e a demonstração, que sai pronta em poucos cliques e abre sem senha.
 */
export function CreateVault({ status }: { status: VaultStatus }) {
  const [choice, setChoice] = useState<Choice | null>(null);
  if (choice === 'real') return <RealVault status={status} onBack={() => setChoice(null)} />;
  const kicker =
    status.vault_count > 1
      ? t('cofre-criar-kicker-nomeado', { nome: status.vault_name })
      : t('cofre-criar-kicker');
  if (choice === 'demo')
    return (
      <VaultLayout
        kicker={kicker}
        heading={t('cofre-escolha-demo-titulo')}
        lead={t('cofre-escolha-demo-lead')}
        dbPath={status.db_path}
        footer={<OtherVaults count={status.vault_count} />}
      >
        <InstrumentCard index="01" title={t('cofre-escolha-demo')}>
          <div className="flex flex-col gap-4">
            <DemoQuick />
            <Button variant="ghost" className="self-start" onClick={() => setChoice(null)}>
              {t('comum-voltar')}
            </Button>
          </div>
        </InstrumentCard>
      </VaultLayout>
    );
  return (
    <VaultLayout
      kicker={kicker}
      heading={t('cofre-criar-titulo')}
      lead={t('cofre-escolha-lead')}
      dbPath={status.db_path}
      footer={<OtherVaults count={status.vault_count} />}
    >
      <div
        role="radiogroup"
        aria-label={t('cofre-escolha-rotulo')}
        className="grid gap-3 sm:grid-cols-2"
      >
        {(
          [
            ['real', t('cofre-escolha-real'), t('cofre-escolha-real-dica'), 'lock'],
            ['demo', t('cofre-escolha-demo'), t('cofre-escolha-demo-dica'), 'sliders'],
          ] as const
        ).map(([id, title, hint, icon]) => (
          <button
            key={id}
            type="button"
            role="radio"
            aria-checked={false}
            onClick={() => setChoice(id)}
            className={cx(
              'flex flex-col gap-2 rounded-card border border-line bg-surface px-5 py-4 text-left transition-colors focus-visible:outline-2 focus-visible:outline-accent',
              id === 'demo' ? 'hover:border-demo' : 'hover:border-accent',
            )}
          >
            <span className="flex items-center gap-2 text-[15px] font-medium text-fg">
              <Icon name={icon} size={16} className={id === 'demo' ? 'text-demo' : 'text-accent'} />
              {title}
            </span>
            <span className="text-[12.5px] leading-snug text-dim">{hint}</span>
          </button>
        ))}
      </div>
    </VaultLayout>
  );
}

/** O cofre real: nome da família, país, membros e a senha mestra. */
function RealVault({ status, onBack }: { status: VaultStatus; onBack: () => void }) {
  const create = useCreateVault();
  const formRef = useRef<HTMLFormElement>(null);
  const nextKey = useRef(1);

  const [householdName, setHouseholdName] = useState('');
  const [country, setCountry] = useState('BR');
  const countries = useCountryChoices().data ?? [];
  const chosen = countries.find((c) => c.code === country);
  const [members, setMembers] = useState<Member[]>([{ key: 0, name: '' }]);
  const [focusKey, setFocusKey] = useState<number | null>(null);
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [acknowledged, setAcknowledged] = useState(false);
  const [submitted, setSubmitted] = useState(false);

  const memberNames = members.map((m) => m.name.trim()).filter(Boolean);
  // Barato o bastante para recalcular a cada renderização.
  const strength = estimatePasswordStrength(password, [
    householdName,
    ...members.map((m) => m.name),
  ]);

  const errors = {
    householdName: householdName.trim() ? undefined : t('cofre-criar-erro-nome-familia'),
    members:
      memberNames.length === 0
        ? t('cofre-criar-erro-sem-membros')
        : new Set(memberNames.map((n) => n.toLocaleLowerCase('pt-BR'))).size !== memberNames.length
          ? t('cofre-criar-erro-membros-repetidos')
          : undefined,
    ...validateNewPassword(password, confirm, strength),
    acknowledged: acknowledged ? undefined : t('cofre-criar-erro-aviso'),
  };
  const valid = Object.values(errors).every((e) => !e);
  const show = (error: string | undefined) => (submitted ? error : undefined);

  const addMember = () => {
    const key = nextKey.current++;
    setMembers((list) => [...list, { key, name: '' }]);
    setFocusKey(key);
  };

  const onSubmit = (event: FormEvent) => {
    event.preventDefault();
    if (create.isPending) return;
    if (!valid) {
      flushSync(() => setSubmitted(true));
      formRef.current?.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus();
      return;
    }
    create.mutate(
      { password, household_name: householdName.trim(), members: memberNames, country },
      {
        onSuccess: () => {
          setPassword('');
          setConfirm('');
          toast({
            tone: 'success',
            title: t('cofre-criar-toast-titulo'),
            description: t('cofre-criar-toast-descricao'),
          });
        },
      },
    );
  };

  return (
    <VaultLayout
      kicker={
        status.vault_count > 1
          ? t('cofre-criar-kicker-nomeado', { nome: status.vault_name })
          : t('cofre-criar-kicker')
      }
      heading={t('cofre-criar-titulo')}
      lead={t('cofre-criar-lead')}
      dbPath={status.db_path}
      footer={<OtherVaults count={status.vault_count} />}
    >
      <InstrumentCard
        index="01"
        title={t('cofre-criar-card-titulo')}
        footer={[
          { label: 'KDF', value: 'Argon2id' },
          { label: t('cofre-ficha-cifra'), value: 'AES-256' },
          { label: t('cofre-ficha-arquivo'), value: t('cofre-criar-arquivo-unico') },
        ]}
      >
        <form ref={formRef} noValidate onSubmit={onSubmit} className="flex flex-col gap-5">
          <Field label={t('cofre-criar-nome-familia')} error={show(errors.householdName)}>
            <Input
              autoFocus
              autoComplete="off"
              placeholder={t('cofre-criar-nome-familia-exemplo')}
              value={householdName}
              onChange={(e) => setHouseholdName(e.target.value)}
            />
          </Field>

          <Field
            label={t('cofre-pais-rotulo')}
            hint={
              chosen
                ? t('cofre-criar-pais-dica', {
                    suporte: supportText(chosen.support),
                    moeda: chosen.currency,
                  })
                : undefined
            }
          >
            <CountryPicker
              choices={countries.length ? countries : FALLBACK}
              value={country}
              onChange={setCountry}
            />
          </Field>

          <fieldset className="flex flex-col gap-2" aria-describedby="membros-ajuda">
            <legend className="label-caps mb-1.5">{t('cofre-criar-membros')}</legend>
            <p id="membros-ajuda" className="-mt-1 text-[12px] leading-snug text-dim">
              {t('cofre-criar-membros-ajuda')}
            </p>
            <ul className="flex flex-col gap-2">
              {members.map((member, i) => (
                <li key={member.key} className="flex items-center gap-2">
                  <span aria-hidden className="num w-5 text-[11px] text-dim">
                    {String(i + 1).padStart(2, '0')}
                  </span>
                  <Input
                    aria-label={t('cofre-criar-membro-n', { n: i + 1 })}
                    autoComplete="off"
                    placeholder={
                      i === 0
                        ? t('cofre-criar-membro-exemplo')
                        : t('cofre-criar-membro-placeholder')
                    }
                    autoFocus={member.key === focusKey}
                    invalid={Boolean(show(errors.members)) && i === 0}
                    value={member.name}
                    onChange={(e) => {
                      const name = e.target.value;
                      setMembers((list) =>
                        list.map((m) => (m.key === member.key ? { ...m, name } : m)),
                      );
                    }}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        e.preventDefault();
                        addMember();
                      }
                    }}
                  />
                  <IconButton
                    label={t('cofre-criar-remover-membro', { n: i + 1 })}
                    icon={<Icon name="x" size={14} />}
                    disabled={members.length === 1}
                    tooltip={false}
                    onClick={() => setMembers((list) => list.filter((m) => m.key !== member.key))}
                  />
                </li>
              ))}
            </ul>
            <div className="flex items-center justify-between">
              <Button
                size="sm"
                variant="ghost"
                icon={<Icon name="plus" size={13} />}
                onClick={addMember}
                className="-ml-2.5"
              >
                {t('cofre-criar-adicionar-membro')}
              </Button>
              {show(errors.members) ? (
                <p className="text-[12px] text-negative">{errors.members}</p>
              ) : null}
            </div>
          </fieldset>

          <div className="flex flex-col gap-2">
            <Field label={t('cofre-senha-mestra')} error={show(errors.password)}>
              <PasswordInput
                autoComplete="new-password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </Field>
            <StrengthMeter strength={strength} />
          </div>

          <Field label={t('cofre-confirmar-senha')} error={show(errors.confirm)}>
            <PasswordInput
              autoComplete="new-password"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
            />
          </Field>

          <div
            className="flex flex-col gap-3 rounded-card border border-warning/45 bg-warning/8 p-3.5"
            id="aviso-recuperacao"
          >
            <p className="flex items-start gap-2.5 text-[13px] leading-relaxed text-fg">
              <Icon name="alert" className="mt-0.5 shrink-0 text-warning" />
              <span>
                <strong className="font-semibold">{t('cofre-criar-aviso-forte')}</strong>{' '}
                {t('cofre-criar-aviso')}
              </span>
            </p>
            <label className="flex cursor-pointer items-start gap-2.5 text-[13px] text-fg">
              <input
                type="checkbox"
                className="mt-0.5 size-4 shrink-0"
                checked={acknowledged}
                aria-invalid={show(errors.acknowledged) ? true : undefined}
                aria-describedby="aviso-recuperacao"
                onChange={(e) => setAcknowledged(e.target.checked)}
              />
              <span>{t('cofre-criar-aviso-entendi')}</span>
            </label>
            {show(errors.acknowledged) ? (
              <p className="text-[12px] text-negative">{errors.acknowledged}</p>
            ) : null}
          </div>

          <FormError error={create.error} />

          <Button
            type="submit"
            variant="primary"
            size="lg"
            fullWidth
            loading={create.isPending}
            loadingText={t('cofre-criar-carregando')}
            icon={<Icon name="shield" size={15} />}
          >
            {t('cofre-criar-botao')}
          </Button>
          <Button variant="ghost" className="self-start" onClick={onBack}>
            {t('comum-voltar')}
          </Button>
        </form>
      </InstrumentCard>
    </VaultLayout>
  );
}
