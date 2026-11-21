/** Menu lateral: o cofre em uso e o atalho para trocar, criar ou abrir outro (docs/vaults.md). */
import { useState } from 'react';

import { useVaultStatus } from '@/api/vaultQueries';
import { Dialog } from '@/design/components';
import { t } from '@/i18n';
import { VaultSwitcher } from '@/vault/VaultSwitcher';

export function VaultMenu() {
  const { data: status } = useVaultStatus();
  const [open, setOpen] = useState(false);
  if (!status) return null;
  const many = status.vault_count > 1;
  return (
    <Dialog
      open={open}
      onOpenChange={setOpen}
      title={t('cofre-menu-titulo')}
      description={t('cofre-menu-descricao')}
      trigger={
        <button
          type="button"
          className="max-w-[60%] truncate rounded-control px-1.5 py-0.5 text-[11px] text-dim hover:bg-surface-2 hover:text-fg"
          title={
            many ? t('cofre-menu-cofre-atual', { nome: status.vault_name }) : t('cofre-menu-trocar')
          }
        >
          {many ? status.vault_name : t('cofre-menu-titulo')}
          {' ▾'}
        </button>
      }
    >
      <VaultSwitcher onDone={() => setOpen(false)} />
    </Dialog>
  );
}
