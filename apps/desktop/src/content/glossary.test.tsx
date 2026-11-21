/**
 * Glossário e dicas nos outros idiomas (docs/i18n-design.md): os textos vêm do catálogo do
 * idioma e a busca dos `?` casa com os rótulos já traduzidos.
 */
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it } from 'vitest';

import { Measure } from '@/design/components';
import { HelpProvider } from '@/design/help';
import { applyLocale, type AppLocale } from '@/i18n';

import { explanation, findTerm, GLOSSARY, type GlossaryId } from './glossary';
import { tipOfTheDay, TIPS } from './tips';

afterEach(() => applyLocale('pt-BR'));

const IDS = Object.keys(GLOSSARY) as GlossaryId[];

describe('glossário em inglês e espanhol', () => {
  it('em inglês, acha o termo pelos rótulos em inglês e explica em inglês', async () => {
    await applyLocale('en');
    expect(findTerm('Emergency fund')).toBe('reserva');
    expect(findTerm('Savings rate')).toBe('taxa_poupanca');
    expect(findTerm('Net worth')).toBe('patrimonio_liquido');
    expect(findTerm('Real XIRR')).toBe('xirr');
    expect(findTerm('Card bill')).toBe('fatura');
    expect(findTerm('Balance owed')).toBe('saldo_devedor');
    // Contas a pagar não são fatura de cartão; rótulos comuns não ganham `?`.
    expect(findTerm('Bills still due this month')).toBeNull();
    expect(findTerm('Description')).toBeNull();
    expect(GLOSSARY.patrimonio_liquido.term).toBe('Net worth');
    expect(explanation('patrimonio_liquido', 'simples')).toMatch(/^Everything you have/);
    expect(explanation('cdi', 'tecnico')).toMatch(/^Certificado de Depósito Interbancário:/);
  });

  it('em espanhol, segue os rótulos em espanhol', async () => {
    await applyLocale('es');
    expect(findTerm('Fondo de emergencia')).toBe('reserva');
    expect(findTerm('Tasa de ahorro')).toBe('taxa_poupanca');
    expect(findTerm('Patrimonio neto')).toBe('patrimonio_liquido');
    expect(findTerm('Resumen de la tarjeta')).toBe('fatura');
    expect(findTerm('XIRR real')).toBe('xirr');
    // "Real" do orçamento (realizado) e "Cambio" do histórico (alteração) não são esses termos.
    expect(findTerm('Real')).toBeNull();
    expect(findTerm('Cambio')).toBeNull();
    expect(explanation('reserva', 'simples')).toMatch(/^Dinero guardado/);
  });

  it('volta para o pt-BR com as buscas de antes', async () => {
    await applyLocale('en');
    expect(findTerm('Patrimônio líquido')).toBeNull();
    await applyLocale('pt-BR');
    expect(findTerm('Patrimônio líquido')).toBe('patrimonio_liquido');
    expect(findTerm('Reserva')).toBe('reserva');
  });

  it.each<AppLocale>(['en', 'es'])(
    'em %s, todo termo tem nome, texto e busca próprios, sem R$',
    async (code) => {
      await applyLocale('pt-BR');
      const pt = Object.fromEntries(IDS.map((id) => [id, GLOSSARY[id].simples]));
      await applyLocale(code);
      for (const id of IDS) {
        const g = GLOSSARY[id];
        expect(g.term.length, id).toBeGreaterThan(1);
        expect(g.simples.length, id).toBeGreaterThan(30);
        expect(g.simples, id).not.toBe(pt[id]);
        expect(g.match?.length, id).toBe(1);
        expect(`${g.simples} ${g.equilibrado ?? ''}`, id).not.toMatch(/R\$/);
      }
      for (const tip of TIPS) expect(tip.text).not.toMatch(/R\$|^dicas-/);
    },
  );
});

describe('? e dica do dia em inglês', () => {
  it('o ? do rótulo traduzido explica em inglês', async () => {
    await applyLocale('en');
    const user = userEvent.setup();
    render(
      <HelpProvider value={{ level: 'iniciante', tone: 'simples' }}>
        <Measure label="Net worth" value="10" />
        <Measure label="Description" value="x" />
      </HelpProvider>,
    );
    const tip = screen.getByRole('button', { name: 'What is net worth?' });
    expect(screen.getAllByRole('button', { name: /^What is/ })).toHaveLength(1);
    await user.hover(tip);
    expect((await screen.findByRole('tooltip')).textContent).toContain(
      GLOSSARY.patrimonio_liquido.simples,
    );
  });

  it('a dica do dia sai no idioma escolhido', async () => {
    await applyLocale('en');
    const p = { level: 'iniciante' as const, goals: ['reserva'], income: null };
    const texts = ['2026-10-01', '2026-10-02', '2026-10-03'].map((d) => tipOfTheDay(p, d));
    expect(texts.join(' ')).toMatch(/emergency fund/);
    expect(texts.join(' ')).not.toMatch(/R\$/);
  });
});
