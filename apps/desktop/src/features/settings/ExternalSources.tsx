/**
 * 11.10 · Fontes externas: liga e desliga cada site ou órgão que o Treasury consulta pela
 * rede. Desligada, a fonte não é usada em nenhuma coleta, rotina ou agente; o que já foi
 * baixado continua no cofre. A busca e o filtro por país rodam só na tela, sobre a lista
 * que `sources.list` já devolve.
 */
import { useDeferredValue, useMemo, useState } from 'react';

import type { SourceInfo } from '@/api/generated/SourceInfo';
import { useSetSource, useSources } from '@/api/hooks/dataSources';
import { Icon, Input, InstrumentCard, toast } from '@/design/components';
import { InlineError, QueryState, Segmented, Switch } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

/** Bandeira e nome de cada país das fontes (`INT` = fonte global). */
const COUNTRIES: Record<string, { flag: string; readonly label: string; readonly short: string }> =
  {
    BR: {
      flag: '🇧🇷',
      get label() {
        return t('ajustes-pais-br');
      },
      get short() {
        return t('ajustes-pais-br');
      },
    },
    US: {
      flag: '🇺🇸',
      get label() {
        return t('ajustes-pais-us');
      },
      get short() {
        return t('ajustes-pais-us-curto');
      },
    },
    EU: {
      flag: '🇪🇺',
      get label() {
        return t('ajustes-pais-eu');
      },
      get short() {
        return t('ajustes-pais-eu-curto');
      },
    },
    GB: {
      flag: '🇬🇧',
      get label() {
        return t('ajustes-pais-gb');
      },
      get short() {
        return t('ajustes-pais-gb');
      },
    },
    CA: {
      flag: '🇨🇦',
      get label() {
        return t('ajustes-pais-ca');
      },
      get short() {
        return t('ajustes-pais-ca');
      },
    },
    INT: {
      flag: '🌐',
      get label() {
        return t('ajustes-pais-int');
      },
      get short() {
        return t('ajustes-pais-int-curto');
      },
    },
  };
const ORDER = ['BR', 'US', 'EU', 'GB', 'CA', 'INT'];

export const countryOf = (code: string) =>
  COUNTRIES[code] ?? { flag: '🏳️', label: code, short: code };

const fold = (s: string) => s.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase();

/** Todas as palavras da busca aparecem no nome, no uso, no site ou no país da fonte. */
export function matchesSource(s: SourceInfo, query: string): boolean {
  const words = fold(query).split(/\s+/).filter(Boolean);
  if (!words.length) return true;
  const c = countryOf(s.country);
  const hay = fold(
    [s.name, s.used_for, s.terms, s.homepage, c.label, c.short, s.country].join(' '),
  );
  return words.every((w) => hay.includes(w));
}

/** "Nenhuma fonte encontrada", com a busca e o país quando houver. */
function emptyText(query: string, country: string): string {
  const pais = countryOf(country).label;
  if (query && country !== 'all')
    return t('ajustes-externas-nenhuma-busca-pais', { busca: query, pais });
  if (query) return t('ajustes-externas-nenhuma-busca', { busca: query });
  if (country !== 'all') return t('ajustes-externas-nenhuma-pais', { pais });
  return t('ajustes-externas-nenhuma');
}

export function ExternalSourcesCard() {
  const { data = [], isLoading, error } = useSources();
  const set = useSetSource();
  const [text, setText] = useState('');
  const [country, setCountry] = useState('all');
  const query = useDeferredValue(text);
  const on = data.filter((s) => s.enabled).length;
  const countries = useMemo(() => {
    const present = new Set(data.map((s) => s.country));
    const known = ORDER.filter((c) => present.has(c));
    return [...known, ...[...present].filter((c) => !ORDER.includes(c)).sort()];
  }, [data]);
  const shown = data.filter(
    (s) => (country === 'all' || s.country === country) && matchesSource(s, query),
  );
  return (
    <InstrumentCard
      index="12.10"
      title={t('ajustes-externas-titulo')}
      actions={
        data.length ? (
          <span className="label-caps text-dim">
            {t('ajustes-externas-ligadas', { ligadas: on, total: data.length })}
          </span>
        ) : undefined
      }
    >
      <div className="flex flex-col gap-3">
        <p className="max-w-[72ch] text-[13px] leading-relaxed text-dim">
          {t('ajustes-externas-texto')}
        </p>
        <QueryState loading={isLoading} error={error}>
          <div className="flex flex-wrap items-center gap-3">
            <div className="w-72 max-w-full">
              <Input
                type="search"
                aria-label={t('ajustes-externas-buscar')}
                placeholder={t('ajustes-externas-buscar-placeholder')}
                value={text}
                onChange={(e) => setText(e.target.value)}
                prefix={<Icon name="search" size={13} />}
              />
            </div>
            {countries.length > 1 ? (
              <Segmented
                label={t('ajustes-externas-filtrar')}
                size="sm"
                className="flex-wrap"
                value={country}
                onChange={setCountry}
                options={[
                  { value: 'all', label: t('ajustes-externas-todas') },
                  ...countries.map((c) => {
                    const info = countryOf(c);
                    return {
                      value: c,
                      label: (
                        <span title={info.label}>
                          <span aria-hidden>{info.flag}</span> {info.short}
                        </span>
                      ),
                    };
                  }),
                ]}
              />
            ) : null}
            {shown.length !== data.length ? (
              <span role="status" className="label-caps text-dim">
                {t('ajustes-externas-mostrando', { n: shown.length, total: data.length })}
              </span>
            ) : null}
          </div>
          {data.length && !shown.length ? (
            <p className="text-[13px] text-dim">{emptyText(text.trim(), country)}</p>
          ) : null}
          <ul className="grid gap-x-8 gap-y-3 @min-[880px]:grid-cols-2">
            {shown.map((s) => (
              <li key={s.id} className="border-t border-line pt-3">
                <Switch
                  checked={s.enabled}
                  disabled={set.isPending}
                  label={
                    <>
                      <span aria-hidden title={countryOf(s.country).label} className="mr-1.5">
                        {countryOf(s.country).flag}
                      </span>
                      {s.name}
                    </>
                  }
                  description={
                    <>
                      {s.used_for}{' '}
                      <a
                        className="text-accent underline"
                        href={s.homepage}
                        target="_blank"
                        rel="noreferrer"
                      >
                        {s.homepage.replace(/^https:\/\/(www\.)?/, '')}
                      </a>
                      <span className="mt-1 block text-[12px] text-dim">{s.terms}</span>
                      {s.health ? (
                        <span
                          role="status"
                          className="mt-1 block text-[12px] font-medium text-warning"
                        >
                          {s.health.auto_disabled
                            ? t('ajustes-externas-desligada-app', { dias: s.health.failing_days })
                            : t('ajustes-externas-falhando', { dias: s.health.failing_days })}{' '}
                          {s.health.last_error
                            ? t('ajustes-externas-ultimo-erro', { erro: s.health.last_error })
                            : null}
                        </span>
                      ) : null}
                    </>
                  }
                  onChange={(enabled) =>
                    set.mutate(
                      { id: s.id, enabled },
                      {
                        onSuccess: () =>
                          toast({
                            tone: 'success',
                            title: enabled
                              ? t('ajustes-externas-ligada', { fonte: s.name })
                              : t('ajustes-externas-desligada', { fonte: s.name }),
                          }),
                      },
                    )
                  }
                />
              </li>
            ))}
          </ul>
        </QueryState>
        <InlineError error={set.error} />
      </div>
    </InstrumentCard>
  );
}
