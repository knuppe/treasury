/**
 * 02 · FAMÍLIA: os membros, a renda de cada um (salário, holerites, benefícios, renda informal),
 * as contas e cartões de cada um (os lançamentos herdam o membro) e a previsão da renda da
 * família nos próximos 12 meses e, com a função ligada, a linha da vida de cada membro. Ver
 * docs/income.md, docs/members.md e docs/life-expectancy.md.
 */
import { Link } from 'react-router';

import { useIncomeSummary } from '@/api/hooks/income';
import { screenMeta } from '@/app/screens';
import { ScreenHeader } from '@/design/components';
import { money, usePageContext } from '@/features/companion/pageContext';
import { IncomePanel } from '@/features/income/IncomePanel';
import { FamilyLifeCard } from '@/features/longevity/FamilyLifeCard';
import { t } from '@/i18n';

const meta = screenMeta('familia');

export default function Familia() {
  const s = useIncomeSummary(12).data;
  // Contexto para o agente: a renda de cada membro e a previsão do próximo mês.
  usePageContext({
    screen: 'familia',
    title: `${meta.index} · ${meta.title}`,
    figures: s
      ? {
          next_month_net: money(s.next.net, s.currency),
          last_12m_net: money(s.last_12m.net, s.currency),
        }
      : undefined,
    items: s?.members.map((m) => ({
      id: m.member_id,
      name: m.member_name,
      incomes: m.sources.length,
      next_month_net: money(m.next.net, s.currency),
    })),
  });
  return (
    <div className="flex flex-col gap-5">
      <ScreenHeader
        index={meta.index}
        title={meta.title}
        description={meta.summary}
        actions={
          <Link to="/ajustes?secao=familia" className="label-caps hover:text-fg">
            {t('familia-nome-instituicoes')}
          </Link>
        }
      />
      {/* Linha da vida: só com a função ligada (docs/life-expectancy.md). */}
      <FamilyLifeCard />
      <IncomePanel />
    </div>
  );
}
