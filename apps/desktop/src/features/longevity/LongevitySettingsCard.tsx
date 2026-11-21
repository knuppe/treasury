/**
 * Ajustes › Perfil › Linha da vida (docs/life-expectancy.md, seção 8): liga e desliga a
 * função da família, esconde a barra de um membro ("mostrar a minha barra" em "Ver como"),
 * mostra o final da barra para crianças e destaca a longevidade avançada. Desligar não apaga
 * nada: nada é guardado além da preferência.
 */
import type { LongevitySettings } from '@/api/generated/LongevitySettings';
import { useLongevitySettings, useSetLongevitySettings } from '@/api/hooks/longevity';
import { useViewAs } from '@/app/viewAs';
import { InstrumentCard, toast } from '@/design/components';
import { Checkbox, InlineError, QueryState, Switch } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

export function LongevitySettingsCard() {
  const q = useLongevitySettings();
  const save = useSetLongevitySettings();
  const { member, members } = useViewAs();
  const s = q.data;
  const set = (patch: Partial<LongevitySettings>, done?: () => void) =>
    s && save.mutate({ ...s, ...patch }, done ? { onSuccess: done } : undefined);
  const showBar = (id: string, show: boolean) =>
    s &&
    set({
      hidden_members: show
        ? s.hidden_members.filter((x) => x !== id)
        : [...new Set([...s.hidden_members, id])],
    });
  return (
    <InstrumentCard index="12.P3" title={t('longevidade-titulo')}>
      <QueryState loading={q.isPending} error={q.error}>
        {s ? (
          <div className="flex flex-col gap-3">
            <Switch
              checked={s.enabled}
              disabled={save.isPending}
              onChange={(enabled) =>
                set({ enabled }, () =>
                  toast({
                    tone: 'success',
                    title: enabled
                      ? t('longevidade-ajustes-ligada')
                      : t('longevidade-ajustes-desligada'),
                  }),
                )
              }
              label={t('longevidade-ajustes-ligar')}
              description={t('longevidade-ajustes-ligar-dica')}
            />
            {s.enabled ? (
              <>
                {member ? (
                  <Switch
                    checked={!s.hidden_members.includes(member.id)}
                    disabled={save.isPending}
                    onChange={(v) => showBar(member.id, v)}
                    label={t('longevidade-ajustes-minha')}
                    description={t('longevidade-ajustes-minha-dica')}
                  />
                ) : members.length ? (
                  <fieldset className="flex flex-col gap-1.5">
                    <legend className="mb-1 text-[13.5px] text-fg">
                      {t('longevidade-ajustes-barras')}
                    </legend>
                    {members.map((m) => (
                      <Checkbox
                        key={m.id}
                        checked={!s.hidden_members.includes(m.id)}
                        disabled={save.isPending}
                        onChange={(v) => showBar(m.id, v)}
                        label={t('longevidade-ajustes-barra-de', { nome: m.name })}
                      />
                    ))}
                  </fieldset>
                ) : null}
                <Switch
                  checked={s.show_children_end}
                  disabled={save.isPending}
                  onChange={(v) => set({ show_children_end: v })}
                  label={t('longevidade-ajustes-criancas')}
                  description={t('longevidade-ajustes-criancas-dica')}
                />
                <Switch
                  checked={s.show_advanced_later_life}
                  disabled={save.isPending}
                  onChange={(v) => set({ show_advanced_later_life: v })}
                  label={t('longevidade-ajustes-avancada')}
                  description={t('longevidade-ajustes-avancada-dica')}
                />
              </>
            ) : null}
            <p className="text-[12px] text-dim">{t('longevidade-ajustes-nada-guardado')}</p>
            <InlineError error={save.error} />
          </div>
        ) : null}
      </QueryState>
    </InstrumentCard>
  );
}
