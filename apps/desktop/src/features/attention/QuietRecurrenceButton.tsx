/**
 * "Não avisar mais desta": no aviso de vencimento de um previsto que veio de uma recorrência
 * (diarista, mesada), desliga o aviso daquela recorrência. A previsão continua igual; o aviso
 * volta pelo sino da lista de recorrências (Lançamentos › Recorrências).
 */
import { useSetRecurrenceReminder } from '@/api/hooks/attention';
import { Button, toast, Tooltip } from '@/design/components';
import { errorText } from '@/features/transactions/ui/errors';
import { t } from '@/i18n';

export function QuietRecurrenceButton({ recurrenceId }: { recurrenceId: string }) {
  const reminder = useSetRecurrenceReminder();
  return (
    <Tooltip content={t('atencao-nao-avisar-dica')}>
      <Button
        size="sm"
        variant="ghost"
        disabled={reminder.isPending}
        onClick={() =>
          reminder.mutate(
            { recurrence_id: recurrenceId, remind: false },
            {
              onSuccess: () => toast({ tone: 'success', title: t('atencao-nao-avisar-feito') }),
              onError: (e) =>
                toast({
                  tone: 'error',
                  title: t('atencao-nao-avisar-erro'),
                  description: errorText(e),
                }),
            },
          )
        }
      >
        {t('atencao-nao-avisar')}
      </Button>
    </Tooltip>
  );
}
