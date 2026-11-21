/** Ajustes › Cofres (docs/vaults.md): a lista de cofres com trocar, criar, abrir arquivo,
 * renomear e tirar da lista (o arquivo nunca é apagado). */
import { InstrumentCard } from '@/design/components';
import { t } from '@/i18n';
import { VaultSwitcher } from '@/vault/VaultSwitcher';

export function VaultsSection() {
  return (
    <InstrumentCard index="12.C" title={t('ajustes-secao-cofres')}>
      <div className="flex flex-col gap-3">
        <p className="text-[12.5px] leading-snug text-dim">{t('ajustes-cofres-texto')}</p>
        <VaultSwitcher manage />
      </div>
    </InstrumentCard>
  );
}
