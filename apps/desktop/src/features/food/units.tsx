/**
 * Unidades da tela Consumo pelo perfil do país (docs/design/consumption-review.md, seção 4): o
 * preço por unidade fica sempre guardado por kg, L ou unidade; onde se usa libra e galão
 * (EUA), a tela mostra por libra e por galão. A conversão é só de exibição e fica aqui.
 */
import type { ConsumptionProfile } from '@/api/generated/ConsumptionProfile';
import { useCore } from '@/api/hooks/coreQuery';
import { Money } from '@/design/components';
import { t } from '@/i18n';

/** O perfil de consumo do país da família (imposto no cupom, unidades, leitor, série oficial). */
export const useConsumptionProfile = () =>
  useCore<ConsumptionProfile>('food', 'profile', {}, { staleTime: 60_000 });

const KG_PER_LB = 0.45359237;
const L_PER_GAL = 3.785411784;

/** O preço por kg ou L convertido para a unidade que a tela mostra, e o rótulo dela. */
export function shownUnitPrice(
  cents: number,
  unit: string | null | undefined,
  units: ConsumptionProfile['units'] | undefined,
): { cents: number; label: string } {
  if (units === 'us' && unit === 'kg')
    return { cents: Math.round(cents * KG_PER_LB), label: t('alimentacao-por-lb') };
  if (units === 'us' && unit === 'l')
    return { cents: Math.round(cents * L_PER_GAL), label: t('alimentacao-por-gal') };
  const label =
    unit === 'kg'
      ? t('alimentacao-por-kg')
      : unit === 'l'
        ? t('alimentacao-por-l')
        : unit === 'un'
          ? t('alimentacao-por-un')
          : '';
  return { cents, label };
}

/** Preço por unidade com o rótulo ("R$ 4,98/kg", "$0.59/lb"). */
export function UnitPrice({ cents, unit }: { cents: number; unit: string | null | undefined }) {
  const profile = useConsumptionProfile().data;
  const shown = shownUnitPrice(cents, unit, profile?.units);
  return (
    <>
      <Money cents={shown.cents} />
      <span className="text-dim">{shown.label}</span>
    </>
  );
}
