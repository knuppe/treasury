import type { PasswordStrength } from '@/lib/passwordStrength';
import { cx } from '@/design/cx';
import { t } from '@/i18n';

const SEGMENT_TONE = ['bg-negative', 'bg-negative', 'bg-warning', 'bg-positive', 'bg-positive'];
const TEXT_TONE = [
  'text-negative',
  'text-negative',
  'text-warning',
  'text-positive',
  'text-positive',
];

/**
 * Medidor de força da senha: quatro segmentos + rótulo + bits estimados. A cor
 * nunca é a única pista: o rótulo diz a força por extenso.
 */
export function StrengthMeter({
  strength,
  showSuggestions = true,
}: {
  strength: PasswordStrength;
  showSuggestions?: boolean;
}) {
  const { score, bits, label, suggestions } = strength;
  const empty = bits === 0;
  return (
    <div className="flex flex-col gap-2">
      <div
        role="meter"
        aria-label={t('cofre-forca-rotulo')}
        aria-valuemin={0}
        aria-valuemax={4}
        aria-valuenow={score}
        aria-valuetext={
          empty ? t('cofre-forca-sem-senha') : t('cofre-forca-descricao', { nivel: label, bits })
        }
        className="flex items-center gap-3"
      >
        <div aria-hidden className="grid flex-1 grid-cols-4 gap-1">
          {[0, 1, 2, 3].map((i) => (
            <span
              key={i}
              className={cx(
                'h-1 rounded-full transition-colors duration-200',
                !empty && i < Math.max(score, 1) ? SEGMENT_TONE[score] : 'bg-line',
              )}
            />
          ))}
        </div>
        <span
          aria-hidden
          className={cx('label-caps w-28 text-right whitespace-nowrap', !empty && TEXT_TONE[score])}
        >
          {empty ? t('cofre-forca-sem-senha') : label}
        </span>
        <span aria-hidden className="num w-20 text-right text-[11px] whitespace-nowrap text-dim">
          {empty ? '' : t('cofre-forca-bits', { bits })}
        </span>
      </div>
      {showSuggestions && !empty && suggestions.length > 0 ? (
        <ul className="flex flex-col gap-1 text-[12px] leading-snug text-dim">
          {suggestions.slice(0, 3).map((s) => (
            <li key={s} className="flex gap-2">
              <span aria-hidden className="num text-accent">
                ›
              </span>
              {s}
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}
