/**
 * Recorte do universo (`UniverseFilter`): classes, setor, segmento, liquidez mínima e
 * "só carteira e watchlist". Usado por rankings e screener. Os textos digitados ficam
 * aqui; o pai recebe o filtro já convertido (liquidez em unidades da moeda).
 */
import { useState } from 'react';

import type { AssetClass } from '@/api/generated/AssetClass';
import type { UniverseFilter } from '@/api/generated/UniverseFilter';
import { Field, Input } from '@/design/components';
import { t } from '@/i18n';

import { Check } from '../portfolio/components/Fields';
import { CLASS_LABEL } from '../portfolio/labels';
import { FUNDAMENTAL_CLASSES } from './labels';
import { numberToInput, parseNumberInput } from './query';

const list = (text: string) =>
  text
    .split(',')
    .map((s) => s.trim())
    .filter(Boolean);

export function UniverseFilters({
  value,
  onChange,
}: {
  value: UniverseFilter;
  onChange: (f: UniverseFilter) => void;
}) {
  const [sector, setSector] = useState((value.sectors ?? []).join(', '));
  const [segment, setSegment] = useState((value.segments ?? []).join(', '));
  const [liq, setLiq] = useState(
    value.min_liquidity != null ? numberToInput(value.min_liquidity, false) : '',
  );
  const classes = value.classes ?? [];
  const toggle = (c: AssetClass, on: boolean) => {
    const next = on ? [...classes, c] : classes.filter((x) => x !== c);
    onChange({ ...value, classes: next.length ? next : undefined });
  };
  const liqParsed = liq.trim() ? parseNumberInput(liq, false) : null;
  const liqError = liq.trim() && (liqParsed == null || liqParsed < 0);
  return (
    <fieldset className="flex flex-col gap-3" aria-label={t('pesquisa-recorte')}>
      <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
        <span className="label-caps">{t('pesquisa-classes')}</span>
        {FUNDAMENTAL_CLASSES.map((c) => (
          <Check
            key={c}
            label={CLASS_LABEL[c]}
            checked={classes.includes(c)}
            onChange={(on) => toggle(c, on)}
          />
        ))}
        <Check
          className="ml-auto"
          label={t('pesquisa-so-carteira')}
          checked={Boolean(value.only_mine)}
          onChange={(on) => onChange({ ...value, only_mine: on || undefined })}
        />
      </div>
      <div className="grid gap-3 @min-[400px]:grid-cols-3">
        <Field label={t('pesquisa-setor-contem')} hint={t('pesquisa-varios-virgula')}>
          <Input
            value={sector}
            placeholder={t('pesquisa-setor-exemplo')}
            onChange={(e) => {
              setSector(e.target.value);
              const l = list(e.target.value);
              onChange({ ...value, sectors: l.length ? l : undefined });
            }}
          />
        </Field>
        <Field label={t('pesquisa-segmento-contem')} hint={t('pesquisa-segmento-dica')}>
          <Input
            value={segment}
            placeholder={t('pesquisa-segmento-exemplo')}
            onChange={(e) => {
              setSegment(e.target.value);
              const l = list(e.target.value);
              onChange({ ...value, segments: l.length ? l : undefined });
            }}
          />
        </Field>
        <Field
          label={t('pesquisa-liquidez-minima')}
          hint={t('pesquisa-liquidez-dica')}
          error={liqError ? t('pesquisa-liquidez-invalida') : undefined}
        >
          <Input
            numeric
            inputMode="decimal"
            value={liq}
            placeholder="0"
            onChange={(e) => {
              setLiq(e.target.value);
              const n = e.target.value.trim() ? parseNumberInput(e.target.value, false) : null;
              onChange({ ...value, min_liquidity: n != null && n > 0 ? n : undefined });
            }}
          />
        </Field>
      </div>
    </fieldset>
  );
}
