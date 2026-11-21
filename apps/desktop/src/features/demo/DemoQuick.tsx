/**
 * Demonstração em poucos cliques (docs/demo.md): país (sugerido pela família aberta ou pelo
 * idioma do app) e um perfil pronto, e o cofre de demonstração é criado e aberto, sem senha.
 * "Personalizar antes de criar" abre o assistente completo com essas escolhas.
 */
import { useState } from 'react';

import type { DemoKind } from '@/api/generated/DemoKind';
import type { ProfileLevel } from '@/api/generated/ProfileLevel';
import { useCountryChoices, useCountryInfo } from '@/api/hooks/country';
import { useNewDemoVault } from '@/api/hooks/vaults';
import { Button, Dialog, Field, Icon, toast } from '@/design/components';
import { CountryPicker } from '@/features/country/CountryPicker';
import { InlineError } from '@/features/transactions/ui/controls';
import { t, useLocale } from '@/i18n';
import { formatDate } from '@/lib/format';

import { DemoWizard, fromPreset, initialDraft, ProfileCards, toProfile } from './DemoWizard';
import { localeCountry, type PresetKind } from './presets';

const FALLBACK = [{ code: 'BR', name: 'Brasil', currency: 'BRL', support: 'full' as const }];

export function DemoQuick({
  level = 'avancado',
  onDone,
}: {
  /** Nível da interface sugerido para o cofre novo. */
  level?: ProfileLevel;
  /** Depois de criar (o cofre de demonstração já está aberto). */
  onDone?: () => void;
}) {
  const locale = useLocale();
  const home = useCountryInfo()?.code ?? localeCountry(locale);
  const countries = useCountryChoices().data ?? [];
  const [country, setCountry] = useState(home);
  const [kind, setKind] = useState<PresetKind>('comfortable');
  const [custom, setCustom] = useState(false);
  const create = useNewDemoVault();

  const run = () => {
    const currency = countries.find((c) => c.code === country)?.currency ?? 'BRL';
    const d = fromPreset(kind, country, currency, initialDraft(level));
    create.mutate(
      { name: d.name, household_name: d.household, profile: toProfile(d) },
      {
        onSuccess: (r) => {
          toast({
            tone: 'success',
            title: t('demo-criado', { nome: r.status.vault_name }),
            description: t('demo-criado-dica', {
              n: r.summary.transactions,
              data: formatDate(r.summary.from),
            }),
          });
          onDone?.();
        },
      },
    );
  };

  return (
    <div className="flex flex-col gap-4">
      <Field label={t('demo-pais')} hint={t('demo-pais-dica')}>
        <CountryPicker
          choices={countries.length ? countries : FALLBACK}
          value={country}
          onChange={setCountry}
        />
      </Field>
      <ProfileCards
        value={kind}
        custom={false}
        onChange={(k: DemoKind) => {
          if (k !== 'custom') setKind(k);
        }}
      />
      <p
        role="note"
        className="flex items-start gap-2 rounded-card border border-warning/60 bg-warning/5 px-3 py-2 text-[12.5px] text-warning"
      >
        <Icon name="info" size={14} className="mt-0.5 shrink-0" />
        <span>{t('demo-sem-senha')}</span>
      </p>
      <InlineError error={create.error} />
      <div className="flex flex-wrap items-center justify-between gap-2">
        <Button variant="ghost" disabled={create.isPending} onClick={() => setCustom(true)}>
          {t('demo-rapido-personalizar')}
        </Button>
        <Button variant="primary" loading={create.isPending} onClick={run}>
          {t('demo-rapido-criar')}
        </Button>
      </div>
      <DemoWizard
        open={custom}
        level={level}
        initial={{ kind, country }}
        onOpenChange={(o) => {
          setCustom(o);
          if (!o) onDone?.();
        }}
      />
    </div>
  );
}

/** A criação rápida numa janela (lista de cofres, boas-vindas). */
export function DemoQuickDialog({
  open,
  onOpenChange,
  level,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  level?: ProfileLevel;
}) {
  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      size="lg"
      index="★"
      title={t('demo-titulo')}
      description={t('demo-rapido-descricao')}
    >
      {open ? <DemoQuick level={level} onDone={() => onOpenChange(false)} /> : null}
    </Dialog>
  );
}
