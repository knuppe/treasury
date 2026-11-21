import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { explanation, findTerm, GLOSSARY, showsAt, type GlossaryId } from '@/content/glossary';

import { InstrumentCard, KeyValue, Measure } from './components';
import { HelpProvider, type HelpSettings } from './help';

const iniciante: HelpSettings = { level: 'iniciante', tone: 'simples' };

describe('glossary', () => {
  it.each([
    ['Patrimônio líquido', 'patrimonio_liquido'],
    ['XIRR real', 'xirr'],
    ['Rentabilidade real', 'real'],
    ['Taxa de poupança', 'taxa_poupanca'],
    ['Resultado não realizado', 'nao_realizado'],
    ['Preço médio', 'preco_medio'],
    ['CDI', 'cdi'],
    ['Come-cotas', 'come_cotas'],
    ['Competência', 'competencia'],
    ['Fatura', 'fatura'],
  ])('“%s” points to %s', (label, id) => {
    expect(findTerm(label)).toBe(id);
  });

  it('common labels and long phrases get no explanation', () => {
    for (const label of ['Descrição', 'Data', 'Conta', 'Categoria', 'Valor']) {
      expect(findTerm(label), label).toBeNull();
    }
    expect(findTerm('Patrimônio líquido de toda a família, somando contas e bens')).toBeNull();
  });

  it('every term has plain text and matches its own name', () => {
    for (const [id, g] of Object.entries(GLOSSARY)) {
      expect(g.simples.length, id).toBeGreaterThan(30);
      expect(g.match?.length, id).toBeGreaterThan(0);
    }
  });

  it('the tone changes the text; without a balanced version, the plain one applies', () => {
    expect(explanation('cdi', 'simples')).toBe(GLOSSARY.cdi.simples);
    expect(explanation('cdi', 'tecnico')).toBe(GLOSSARY.cdi.equilibrado);
    expect(explanation('sobra', 'tecnico')).toBe(GLOSSARY.sobra.simples);
  });

  it('everyday terms only at Beginner; technical ones also at Intermediate; none at Advanced', () => {
    const basic: GlossaryId = 'fatura';
    const technical: GlossaryId = 'xirr';
    expect(showsAt(basic, 'iniciante')).toBe(true);
    expect(showsAt(basic, 'intermediario')).toBe(false);
    expect(showsAt(technical, 'intermediario')).toBe(true);
    expect(showsAt(technical, 'avancado')).toBe(false);
  });
});

describe('? icons in the components', () => {
  const ui = (settings?: HelpSettings) => {
    const body = (
      <>
        <Measure label="Patrimônio líquido" value="R$ 10" />
        <InstrumentCard title="Taxa de poupança">corpo</InstrumentCard>
        <KeyValue items={[{ label: 'XIRR nominal', value: '11%' }]} />
        <Measure label="Data" value="hoje" />
      </>
    );
    return render(settings ? <HelpProvider value={settings}>{body}</HelpProvider> : body);
  };

  it('at Beginner, each technical term gets a ? that explains on hover', async () => {
    const user = userEvent.setup();
    ui(iniciante);
    const tips = screen.getAllByRole('button', { name: /^O que é/ });
    expect(tips.map((t) => t.getAttribute('data-help'))).toEqual([
      'patrimonio_liquido',
      'taxa_poupanca',
      'xirr',
    ]);
    // The card title stays just the title (the ? stays outside the heading).
    expect(screen.getByRole('heading', { name: 'Taxa de poupança' })).toBeTruthy();
    await user.hover(tips[0]!);
    const tip = await screen.findByRole('tooltip');
    expect(tip.textContent).toContain(GLOSSARY.patrimonio_liquido.simples);
  });

  it('at Intermediate, only the technical ones; at Advanced or with no profile, none', () => {
    const { unmount } = ui({ level: 'avancado', tone: 'tecnico' });
    expect(screen.queryAllByRole('button', { name: /^O que é/ })).toHaveLength(0);
    unmount();
    const r2 = ui();
    expect(screen.queryAllByRole('button', { name: /^O que é/ })).toHaveLength(0);
    r2.unmount();
    render(
      <HelpProvider value={{ level: 'intermediario', tone: 'equilibrado' }}>
        <Measure label="Fatura" value="R$ 1" />
        <Measure label="CDI" value="100%" />
      </HelpProvider>,
    );
    expect(
      screen.getAllByRole('button', { name: /^O que é/ }).map((b) => b.getAttribute('data-help')),
    ).toEqual(['cdi']);
  });
});
