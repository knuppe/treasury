import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { explanation, findTerm, GLOSSARY, showsAt, type GlossaryId } from '@/content/glossary';

import { InstrumentCard, KeyValue, Measure } from './components';
import { HelpProvider, type HelpSettings } from './help';

const iniciante: HelpSettings = { level: 'iniciante', tone: 'simples' };

describe('glossário', () => {
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
  ])('“%s” aponta para %s', (label, id) => {
    expect(findTerm(label)).toBe(id);
  });

  it('rótulos comuns e frases longas não ganham explicação', () => {
    for (const label of ['Descrição', 'Data', 'Conta', 'Categoria', 'Valor']) {
      expect(findTerm(label), label).toBeNull();
    }
    expect(findTerm('Patrimônio líquido de toda a família, somando contas e bens')).toBeNull();
  });

  it('todo termo tem texto simples e casa com o próprio nome', () => {
    for (const [id, g] of Object.entries(GLOSSARY)) {
      expect(g.simples.length, id).toBeGreaterThan(30);
      expect(g.match?.length, id).toBeGreaterThan(0);
    }
  });

  it('o tom muda o texto; sem versão equilibrada, vale a simples', () => {
    expect(explanation('cdi', 'simples')).toBe(GLOSSARY.cdi.simples);
    expect(explanation('cdi', 'tecnico')).toBe(GLOSSARY.cdi.equilibrado);
    expect(explanation('sobra', 'tecnico')).toBe(GLOSSARY.sobra.simples);
  });

  it('termos do dia a dia só no Iniciante; técnicos também no Intermediário; nada no Avançado', () => {
    const basic: GlossaryId = 'fatura';
    const technical: GlossaryId = 'xirr';
    expect(showsAt(basic, 'iniciante')).toBe(true);
    expect(showsAt(basic, 'intermediario')).toBe(false);
    expect(showsAt(technical, 'intermediario')).toBe(true);
    expect(showsAt(technical, 'avancado')).toBe(false);
  });
});

describe('ícones ? nos componentes', () => {
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

  it('no Iniciante, cada termo técnico ganha um ? que explica ao passar o mouse', async () => {
    const user = userEvent.setup();
    ui(iniciante);
    const tips = screen.getAllByRole('button', { name: /^O que é/ });
    expect(tips.map((t) => t.getAttribute('data-help'))).toEqual([
      'patrimonio_liquido',
      'taxa_poupanca',
      'xirr',
    ]);
    // O título do card continua sendo só o título (o ? fica fora do heading).
    expect(screen.getByRole('heading', { name: 'Taxa de poupança' })).toBeTruthy();
    await user.hover(tips[0]!);
    const tip = await screen.findByRole('tooltip');
    expect(tip.textContent).toContain(GLOSSARY.patrimonio_liquido.simples);
  });

  it('no Intermediário, só os técnicos; no Avançado ou sem perfil, nenhum', () => {
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
