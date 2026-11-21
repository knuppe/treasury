/**
 * Correção na hora quando o score de uma ação dos EUA ou BDR para por falta do e-mail de
 * contato exigido pela SEC (`missing.key = 'sec_contact'`): grava o User-Agent com o e-mail
 * (`research.set_settings`) e roda a coleta só da SEC (`research.sync`), que refaz o score.
 */
import { useState } from 'react';

import { useResearchSync, useSetResearchSettings } from '@/api/hooks/research';
import { APP_VERSION } from '@/app/Brand';
import { Button, Field, Input, toast } from '@/design/components';
import { InlineError } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

const EMAIL = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

export function SecContactFix() {
  const [email, setEmail] = useState('');
  const save = useSetResearchSettings();
  const sync = useResearchSync();
  const valid = EMAIL.test(email.trim());
  const busy = save.isPending || sync.isPending;
  const submit = () => {
    if (!valid) return;
    save.mutate(
      { sec_user_agent: `Treasury/${APP_VERSION} ${email.trim()}` },
      {
        onSuccess: () =>
          sync.mutate(
            { sources: ['sec'] },
            {
              onSuccess: () => toast({ tone: 'success', title: t('score-sec-baixadas') }),
            },
          ),
      },
    );
  };
  return (
    <form
      className="flex flex-col gap-2"
      onSubmit={(e) => {
        e.preventDefault();
        submit();
      }}
    >
      <p>{t('score-sec-texto')}</p>
      <div className="flex flex-wrap items-end gap-2">
        <Field label={t('score-sec-email')} className="w-72">
          <Input
            type="email"
            value={email}
            autoComplete="email"
            placeholder={t('score-sec-email-exemplo')}
            onChange={(e) => setEmail(e.target.value)}
          />
        </Field>
        <Button type="submit" variant="primary" disabled={!valid} loading={busy}>
          {sync.isPending ? t('score-sec-baixando') : t('score-sec-salvar')}
        </Button>
      </div>
      <InlineError error={save.error ?? sync.error} />
    </form>
  );
}
