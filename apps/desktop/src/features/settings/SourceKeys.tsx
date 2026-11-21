/**
 * Chaves das fontes de cotação que a própria pessoa cadastra (docs/data-sources.md): Tiingo
 * (Estados Unidos, gratuita) e EODHD (bolsas do mundo todo). A chave é de quem usa, fica cifrada
 * no cofre e nunca volta para a tela; os dados são de uso pessoal e não saem do cofre.
 */
import { useState } from 'react';

import { useDataSources, useSetEodhdToken, useSetTiingoToken } from '@/api/hooks/dataSources';
import { Button, Field, InstrumentCard, KeyValue, PasswordInput, toast } from '@/design/components';
import { InlineError } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

import { tRich } from './richText';

type Setter = ReturnType<typeof useSetTiingoToken>;

function KeyCard({
  index,
  title,
  text,
  site,
  href,
  isSet,
  setter,
}: {
  index: string;
  title: string;
  text: 'ajustes-tiingo-texto' | 'ajustes-eodhd-texto';
  site: string;
  href: string;
  isSet: boolean;
  setter: Setter;
}) {
  const [key, setKey] = useState('');
  const valid = /^[A-Za-z0-9._-]{10,200}$/.test(key.trim());
  return (
    <InstrumentCard index={index} title={title}>
      <div className="flex flex-col gap-3">
        <p className="max-w-[64ch] text-[13px] leading-relaxed text-dim">
          {tRich(text, {
            link: (
              <a className="text-accent underline" href={href} target="_blank" rel="noreferrer">
                {site}
              </a>
            ),
          })}
        </p>
        <KeyValue
          items={[
            {
              label: t('ajustes-chave-fonte'),
              value: isSet ? (
                <span className="text-positive">{t('ajustes-chave-fonte-cadastrada')}</span>
              ) : (
                t('ajustes-chave-fonte-nao-cadastrada')
              ),
            },
          ]}
        />
        <div className="flex flex-wrap items-end gap-2">
          <Field
            label={
              isSet
                ? t('ajustes-chave-fonte-trocar')
                : t('ajustes-chave-fonte-campo', { fonte: site })
            }
            className="w-80"
          >
            <PasswordInput
              value={key}
              onChange={(e) => setKey(e.target.value.trim())}
              autoComplete="off"
            />
          </Field>
          <Button
            variant="secondary"
            disabled={!valid}
            loading={setter.isPending}
            onClick={() =>
              setter.mutate(
                { key },
                {
                  onSuccess: () => (
                    toast({
                      tone: 'success',
                      title: t('ajustes-chave-fonte-salva', { fonte: site }),
                    }),
                    setKey('')
                  ),
                },
              )
            }
          >
            {t('ajustes-chave-fonte-salvar')}
          </Button>
          {isSet ? (
            <Button
              variant="ghost"
              onClick={() =>
                setter.mutate(
                  { key: '' },
                  {
                    onSuccess: () =>
                      toast({
                        tone: 'success',
                        title: t('ajustes-chave-fonte-removida', { fonte: site }),
                      }),
                  },
                )
              }
            >
              {t('ajustes-remover')}
            </Button>
          ) : null}
        </div>
        <InlineError error={setter.error} />
      </div>
    </InstrumentCard>
  );
}

export function SourceKeyCards() {
  const { data } = useDataSources();
  const tiingo = useSetTiingoToken();
  const eodhd = useSetEodhdToken();
  return (
    <>
      <KeyCard
        index="12.13b"
        title={t('ajustes-tiingo-titulo')}
        text="ajustes-tiingo-texto"
        site="Tiingo"
        href="https://www.tiingo.com/account/api/token"
        isSet={Boolean(data?.tiingo_token_set)}
        setter={tiingo}
      />
      <KeyCard
        index="12.13c"
        title={t('ajustes-eodhd-titulo')}
        text="ajustes-eodhd-texto"
        site="EODHD"
        href="https://eodhd.com/cp/dashboard"
        isSet={Boolean(data?.eodhd_token_set)}
        setter={eodhd}
      />
    </>
  );
}
