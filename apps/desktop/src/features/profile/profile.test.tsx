import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { tipOfTheDay } from '@/content/tips';
import { renderApp } from '@/test/renderApp';

const BEGINNER = [
  /Nada seguro/,
  /nunca controlei/,
  /Saber para onde o dinheiro está indo/,
  /Não sei o que é/,
  /Benefícios do governo/,
  /Não entendo e fico perdido/,
  /Bem simples/,
  // 8th question, no points: the life line (docs/life-expectancy.md).
  /Decidir depois/,
];

describe('profile: welcome, level and simple dashboard', () => {
  it('beginner questionnaire: simple dashboard and advanced screens under "Mais telas"', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/boas-vindas');
    await user.click(
      await screen.findByRole('button', { name: /Quero personalizar o app para mim/ }),
    );
    for (const [i, option] of BEGINNER.entries()) {
      expect(await screen.findByText(`Pergunta ${i + 1} de 8`)).toBeTruthy();
      await user.click(screen.getByRole(i === 2 ? 'checkbox' : 'radio', { name: option }));
      await user.click(screen.getByRole('button', { name: i === 7 ? 'Concluir' : 'Continuar' }));
    }
    expect(await screen.findByText('Preparamos o app no modo Iniciante')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Começar' }));

    const summary = await screen.findByRole('region', { name: 'Resumo do mês' });
    expect(within(summary).getByText('Quanto entrou')).toBeTruthy();
    expect(within(summary).getByText('O que sobrou')).toBeTruthy();
    expect(screen.getByLabelText('Dica do dia')).toBeTruthy();
    expect(screen.getByText('Sua reserva de emergência')).toBeTruthy();
    // Carteira, Ativos and Impostos leave the main menu and stay under "Mais telas".
    const nav = screen.getByRole('navigation', { name: 'Telas' });
    const more = within(nav).getByRole('list', { name: 'Mais telas' });
    expect(within(more).getByRole('link', { name: /Carteira/ })).toBeTruthy();
    expect(within(more).getByRole('link', { name: /Relatórios/ })).toBeTruthy();

    // "Ver painel completo" and back.
    await user.click(screen.getByRole('button', { name: 'Ver painel completo' }));
    await user.click(await screen.findByRole('button', { name: 'Voltar ao painel simples' }));
    expect(await screen.findByLabelText('Dica do dia')).toBeTruthy();

    // The "Anotar um gasto" shortcut opens the new transaction.
    await user.click(screen.getByRole('button', { name: 'Anotar um gasto' }));
    expect(await screen.findByRole('dialog', { name: 'Novo lançamento' })).toBeTruthy();
  });

  it('skipping goes straight to advanced mode; Settings changes the level', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/boas-vindas');
    await user.click(await screen.findByRole('button', { name: /Já sei o que quero/ }));
    await waitFor(() => expect(window.location.hash).toContain('/painel'));
    const nav = await screen.findByRole('navigation', { name: 'Telas' });
    expect(within(nav).queryByRole('list', { name: 'Mais telas' })).toBeNull();
    expect(screen.queryByRole('region', { name: 'Resumo do mês' })).toBeNull();

    window.location.hash = '#/ajustes?secao=perfil';
    const levels = await screen.findByRole('radiogroup', { name: 'Nível de experiência' });
    expect(
      within(levels)
        .getByRole('radio', { name: /Avançado/ })
        .getAttribute('aria-checked'),
    ).toBe('true');
    await user.click(within(levels).getByRole('radio', { name: /Intermediário/ }));
    await waitFor(() =>
      expect(
        within(levels)
          .getByRole('radio', { name: /Intermediário/ })
          .getAttribute('aria-checked'),
      ).toBe('true'),
    );
    window.location.hash = '#/painel';
    expect(await screen.findByRole('region', { name: 'Resumo do mês' })).toBeTruthy();
    expect(
      screen.getByRole('button', { name: 'Ver painel completo (patrimônio e carteira)' }),
    ).toBeTruthy();
  });

  it('tip of the day: changes with the day and respects goal and income', () => {
    const p = { level: 'iniciante' as const, goals: ['dividas'], income: 'beneficios' };
    const days = new Set(
      ['2026-10-01', '2026-10-02', '2026-10-03', '2026-10-04'].map((d) => tipOfTheDay(p, d)),
    );
    expect(days.size).toBeGreaterThan(1);
    expect(tipOfTheDay(p, '2026-10-01')).toBe(tipOfTheDay(p, '2026-10-01'));
    const advanced = tipOfTheDay({ level: 'avancado', goals: [], income: null }, '2026-10-01');
    expect(advanced).not.toMatch(/R\$ 50/);
  });
});
