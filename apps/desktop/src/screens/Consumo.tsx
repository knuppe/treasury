/**
 * 06 · Consumo (docs/consumption.md): tudo o que a família compra, item a item, em qualquer país.
 * Visão geral do mês por segmento (Alimentação, Vestuário, Eletrônicos…), com a comida em casa e
 * fora; as compras (cadastradas à mão ou lidas do cupom fiscal, no Brasil); e os produtos com o
 * preço por kg, L ou unidade ao longo do tempo; e a inflação da sua cesta contra a oficial
 * (docs/gtin-inflation.md); e a saúde da cesta pelo Open Food Facts. O segmento escolhido vale para as abas de compras e produtos.
 */
import { useSearchParams } from 'react-router';

import { screenMeta } from '@/app/screens';
import { ScreenHeader, Tabs, TabsContent, TabsList, TabsTrigger } from '@/design/components';
import { usePageContext } from '@/features/companion/pageContext';
import { BasketCard, BasketPanel } from '@/features/food/BasketPanel';
import { ConsumoOverview, FoodProductsPanel } from '@/features/food/FoodPanels';
import { FoodHealthPanel } from '@/features/food/HealthPanel';
import { PurchasesPanel } from '@/features/food/PurchasesPanel';
import { MonthPicker } from '@/features/reports/controls';
import { t } from '@/i18n';
import { todayIso } from '@/lib/format';
import { monthOf } from '@/lib/months';

const meta = screenMeta('consumo');

type Tab = 'visao' | 'compras' | 'produtos' | 'precos' | 'saude';
const TABS: readonly Tab[] = ['visao', 'compras', 'produtos', 'precos', 'saude'];

export default function Consumo() {
  const [params, setParams] = useSearchParams();
  const current = monthOf(todayIso());
  const m = params.get('mes');
  const month = m && /^\d{4}-\d{2}$/.test(m) ? m : current;
  const aba = params.get('aba') as Tab | null;
  const tab: Tab = aba && TABS.includes(aba) ? aba : 'visao';
  const segment = params.get('seg');
  const update = (patch: Record<string, string | null>) => {
    const p = new URLSearchParams(params);
    for (const [k, v] of Object.entries(patch)) {
      if (v == null) p.delete(k);
      else p.set(k, v);
    }
    setParams(p, { replace: true });
  };
  const setSegment = (id: string | null) => update({ seg: id });
  usePageContext({
    screen: 'consumo',
    title: `${meta.index} · ${meta.title}`,
    view: tab,
    period: { month },
  });
  return (
    <div className="flex flex-col gap-6">
      <ScreenHeader
        index={meta.index}
        title={meta.title}
        description={meta.summary}
        actions={
          tab === 'produtos' || tab === 'precos' || tab === 'saude' ? undefined : (
            <MonthPicker
              month={month}
              max={current}
              onChange={(next) => update({ mes: next === current ? null : next })}
            />
          )
        }
      />
      <Tabs value={tab} onValueChange={(v) => update({ aba: v === 'visao' ? null : v })}>
        <TabsList aria-label={t('alimentacao-abas')}>
          <TabsTrigger value="visao">{t('alimentacao-aba-visao')}</TabsTrigger>
          <TabsTrigger value="compras">{t('alimentacao-aba-compras')}</TabsTrigger>
          <TabsTrigger value="produtos">{t('alimentacao-aba-produtos')}</TabsTrigger>
          <TabsTrigger value="precos">{t('alimentacao-aba-precos')}</TabsTrigger>
          <TabsTrigger value="saude">{t('alimentacao-aba-saude')}</TabsTrigger>
        </TabsList>
        <TabsContent value="visao">
          <div className="flex flex-col gap-4">
            <BasketCard onOpen={() => update({ aba: 'precos' })} />
            <ConsumoOverview month={month} segment={segment} onSegment={setSegment} />
          </div>
        </TabsContent>
        <TabsContent value="compras">
          <PurchasesPanel month={month} segment={segment} onSegment={setSegment} />
        </TabsContent>
        <TabsContent value="produtos">
          <FoodProductsPanel segment={segment} onSegment={setSegment} />
        </TabsContent>
        <TabsContent value="precos">
          <BasketPanel />
        </TabsContent>
        <TabsContent value="saude">
          <FoodHealthPanel />
        </TabsContent>
      </Tabs>
    </div>
  );
}
