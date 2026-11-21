/**
 * Painel da família: um quadro por membro (docs/members.md) com o que entrou, saiu e sobrou
 * no mês, o patrimônio, o nível do perfil e os atalhos "Ver como" e "Fazer o questionário".
 */
import { useNavigate } from 'react-router';

import { useMembersOverview } from '@/api/hooks/household';
import { LEVEL_LABEL } from '@/app/profile';
import { useViewAs } from '@/app/viewAs';
import { Button, InstrumentCard, Money } from '@/design/components';
import { tRich } from '@/features/settings/richText';
import { t } from '@/i18n';
import { formatPercent } from '@/lib/format';

export function MembersCard({ month }: { month: string }) {
  const overview = useMembersOverview(month);
  const { setViewAs } = useViewAs();
  const navigate = useNavigate();
  const o = overview.data;
  if (!o || o.members.length < 2) return null;
  const sharedAny = o.shared.income !== 0 || o.shared.expense !== 0;
  return (
    <InstrumentCard
      index="★"
      title={t('perfil-membros-titulo')}
      className="@min-[1040px]:col-span-12"
    >
      <div className="flex flex-col gap-3">
        <ul
          aria-label={t('perfil-membros')}
          className="grid gap-3 @min-[720px]:grid-cols-2 @min-[1040px]:grid-cols-3"
        >
          {o.members.map((m) => (
            <li
              key={m.member.id}
              className="flex flex-col gap-2 rounded-control border border-line px-3.5 py-3"
            >
              <div className="flex items-baseline justify-between gap-2">
                <span className="text-[14px] font-medium text-fg">{m.member.name}</span>
                <span className="text-[11.5px] text-dim">
                  {m.has_own_profile
                    ? LEVEL_LABEL[m.level]
                    : t('perfil-membro-nivel-familia', { nivel: LEVEL_LABEL[m.level] })}
                </span>
              </div>
              <dl className="grid grid-cols-3 gap-2 text-[12px]">
                <div>
                  <dt className="text-dim">{t('perfil-membro-entrou')}</dt>
                  <dd>
                    <Money cents={m.month.income} currency={o.currency} tone="positive" />
                  </dd>
                </div>
                <div>
                  <dt className="text-dim">{t('perfil-membro-saiu')}</dt>
                  <dd>
                    <Money cents={m.month.expense} currency={o.currency} tone="negative" />
                  </dd>
                </div>
                <div>
                  <dt className="text-dim">{t('perfil-membro-sobrou')}</dt>
                  <dd>
                    <Money cents={m.month.net} currency={o.currency} tone="auto" />
                  </dd>
                </div>
              </dl>
              <p className="text-[12px] text-dim">
                {tRich('perfil-membro-patrimonio', {
                  valor: <Money cents={m.net_worth} currency={o.currency} />,
                })}
                {[
                  m.month.savings_rate_12m != null
                    ? t('perfil-membro-guardou', { pct: formatPercent(m.month.savings_rate_12m) })
                    : null,
                  t('perfil-membro-contas', { n: m.accounts }),
                  m.loans ? t('perfil-membro-dividas', { n: m.loans }) : null,
                ]
                  .filter(Boolean)
                  .map((part) => ` · ${part}`)
                  .join('')}
              </p>
              <div className="flex flex-wrap gap-2">
                <Button size="sm" variant="secondary" onClick={() => setViewAs(m.member.id)}>
                  {t('perfil-ver-como', { nome: m.member.name })}
                </Button>
                {m.has_own_profile ? null : (
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => {
                      setViewAs(m.member.id);
                      void navigate('/boas-vindas?refazer=1');
                    }}
                  >
                    {t('perfil-fazer-questionario')}
                  </Button>
                )}
              </div>
            </li>
          ))}
        </ul>
        {sharedAny ? (
          <p className="text-[12px] text-dim">
            {tRich('perfil-membros-familia', {
              entrou: <Money cents={o.shared.income} currency={o.currency} />,
              saiu: <Money cents={o.shared.expense} currency={o.currency} />,
            })}
          </p>
        ) : null}
      </div>
    </InstrumentCard>
  );
}
