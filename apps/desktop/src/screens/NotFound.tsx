import { useNavigate } from 'react-router';

import { Button, EmptyState, Icon, ScreenHeader } from '@/design/components';
import { t } from '@/i18n';

export default function NotFound() {
  const navigate = useNavigate();
  return (
    <div className="flex flex-col gap-6">
      <ScreenHeader index="—" title={t('casca-nao-encontrada')} />
      <EmptyState
        kicker={t('casca-sem-sinal')}
        title={t('casca-nao-encontrada-titulo')}
        description={t('casca-nao-encontrada-texto')}
        actions={
          <Button
            variant="secondary"
            icon={<Icon name="arrowRight" size={14} />}
            onClick={() => void navigate('/painel')}
          >
            {t('casca-ir-painel')}
          </Button>
        }
      />
    </div>
  );
}
