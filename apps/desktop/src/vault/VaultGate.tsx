import { useQueryClient } from '@tanstack/react-query';
import { useEffect, type ReactNode } from 'react';

import { clearSensitiveCache, useVaultStatus } from '@/api/vaultQueries';
import { Brand } from '@/app/Brand';
import { Button, Icon, InstrumentCard, ScannerSweep, LiveIndicator } from '@/design/components';
import { TermsGate } from '@/features/terms/TermsGate';
import { t } from '@/i18n';
import { LanguageGate } from '@/i18n/LanguageGate';

import { CreateVault } from './CreateVault';
import { UnlockVault } from './UnlockVault';

function Splash() {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center gap-5 bg-canvas">
      <Brand motion="loop" />
      <ScannerSweep variant="inline" />
      <LiveIndicator label={t('cofre-lendo')} tone="accent" />
    </div>
  );
}

function CoreUnavailable({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <div className="flex min-h-screen items-center justify-center overflow-y-auto bg-canvas px-6">
      <InstrumentCard
        index="!"
        title={t('cofre-nucleo-indisponivel')}
        className="w-full max-w-[460px]"
      >
        <div role="alert" className="flex flex-col gap-4">
          <p className="text-[14px] leading-relaxed text-fg">{t('cofre-nucleo-erro')}</p>
          <p className="num text-[12px] text-dim">{message}</p>
          <Button variant="secondary" icon={<Icon name="refresh" size={14} />} onClick={onRetry}>
            {t('cofre-nucleo-tentar')}
          </Button>
        </div>
      </InstrumentCard>
    </div>
  );
}

/**
 * Porta do cofre: decide entre criar, desbloquear ou mostrar o app. Sempre que
 * o cofre deixa de estar aberto (bloqueio pelo usuário, por inatividade ou pelo
 * núcleo), o cache de dados em memória é descartado.
 */
export function VaultGate({ children }: { children: ReactNode }) {
  const client = useQueryClient();
  const { data: status, error, isPending, refetch, isRefetching } = useVaultStatus();
  const state = status?.state;

  useEffect(() => {
    if (state && state !== 'unlocked') {
      clearSensitiveCache(client);
      document.title = 'Treasury';
    }
  }, [state, client]);

  if (isPending) return <Splash />;
  if (!status) {
    return (
      <CoreUnavailable
        message={error?.message ?? t('cofre-erro-desconhecido')}
        onRetry={() => {
          if (!isRefetching) void refetch();
        }}
      />
    );
  }
  // Idioma e termos de uso antes de qualquer cofre (docs/i18n-design.md, docs/terms.md).
  const first = (node: ReactNode) => (
    <LanguageGate>
      <TermsGate>{node}</TermsGate>
    </LanguageGate>
  );
  if (status.state === 'no_vault') return first(<CreateVault status={status} />);
  if (status.state === 'locked') return first(<UnlockVault status={status} />);
  return first(children);
}
