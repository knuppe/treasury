/**
 * 16 · CALCULADORAS: contas soltas para testar uma ideia sem mexer no cofre (docs/calculators.md):
 * juros compostos, viver de renda, à vista × parcelado e alugar × comprar. As taxas de referência
 * oficiais (juro básico, CDI, inflação de 12 meses, poupança) aparecem com a data e só entram
 * num campo quando a pessoa pede.
 *
 * Estado na URL: `?aba=` (juros | renda | parcelado | aluguel).
 */
import { useSearchParams } from 'react-router';

import { screenMeta } from '@/app/screens';
import { ScreenHeader, Tabs, TabsContent, TabsList, TabsTrigger } from '@/design/components';
import {
  CompoundCalculator,
  IncomeCalculator,
  InstallmentsCalculator,
  RentBuyCalculator,
} from '@/features/calculators/Calculators';
import { ReferencesPanel, useRateReferences } from '@/features/calculators/shared';
import { usePageContext } from '@/features/companion/pageContext';
import { t } from '@/i18n';

const meta = screenMeta('calculadoras');

export const CALC_TABS = [
  { id: 'juros', label: 'calc-aba-juros' },
  { id: 'renda', label: 'calc-aba-renda' },
  { id: 'parcelado', label: 'calc-aba-parcelado' },
  { id: 'aluguel', label: 'calc-aba-aluguel' },
] as const;

export default function Calculadoras() {
  const [params, setParams] = useSearchParams();
  const tab = CALC_TABS.some((tb) => tb.id === params.get('aba')) ? params.get('aba')! : 'juros';
  const refs = useRateReferences().data;
  usePageContext({ screen: 'calculadoras', title: `${meta.index} · ${meta.title}`, view: tab });
  return (
    <div className="flex flex-col gap-6">
      <ScreenHeader index={meta.index} title={meta.title} description={meta.summary} />
      <ReferencesPanel refs={refs} />
      <Tabs
        value={tab}
        onValueChange={(v) =>
          setParams((prev) => {
            const next = new URLSearchParams(prev);
            if (v === 'juros') next.delete('aba');
            else next.set('aba', v);
            return next;
          })
        }
      >
        <TabsList aria-label={t('calc-secoes')}>
          {CALC_TABS.map((tb) => (
            <TabsTrigger key={tb.id} value={tb.id}>
              {t(tb.label)}
            </TabsTrigger>
          ))}
        </TabsList>
        <TabsContent value="juros">
          <CompoundCalculator refs={refs} />
        </TabsContent>
        <TabsContent value="renda">
          <IncomeCalculator refs={refs} />
        </TabsContent>
        <TabsContent value="parcelado">
          <InstallmentsCalculator refs={refs} />
        </TabsContent>
        <TabsContent value="aluguel">
          <RentBuyCalculator refs={refs} />
        </TabsContent>
      </Tabs>
      <p className="text-[12px] text-dim">{t('calc-aviso')}</p>
    </div>
  );
}
