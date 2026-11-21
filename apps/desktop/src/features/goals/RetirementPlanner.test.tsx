import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

describe('05 · Orçamento › Aposentadoria', () => {
  it('asks for the age, simulates with the portfolio and saves the plan', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/orcamento?aba=aposentadoria');
    expect(await screen.findByText('Informe a idade de hoje para simular.')).toBeTruthy();
    await user.type(screen.getByLabelText('Idade hoje'), '40');
    expect(await screen.findByText('Chance de o dinheiro durar')).toBeTruthy();
    expect(screen.getByRole('table', { name: 'Patrimônio por idade' })).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Salvar plano' }));
    expect(await screen.findByText('Plano de aposentadoria salvo')).toBeTruthy();
    await waitFor(async () => {
      const r = (await mockCall('goals.retirement', {})) as { plan: { current_age: number } };
      expect(r.plan.current_age).toBe(40);
    });
  });

  it('computes INSS by the rules, saves scenarios and links the goal to the plan', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/orcamento?aba=aposentadoria');
    await user.type(await screen.findByLabelText('Idade hoje'), '45');
    expect(await screen.findByText('Chance de o dinheiro durar')).toBeTruthy();
    // INSS by the rules: the rules table shows up with the one used.
    await user.click(screen.getByRole('combobox', { name: 'INSS' }));
    await user.click(await screen.findByRole('option', { name: 'Calcular pelas regras' }));
    await user.type(screen.getByLabelText('Contribuição até hoje (anos)'), '20');
    const rules = await screen.findByRole('table', { name: 'Regras: INSS' });
    expect(within(rules).getByText('usada')).toBeTruthy();
    expect(screen.queryByLabelText('Valor mensal estimado (INSS)')).toBeNull();
    expect(screen.getByText('Necessário ao parar (90% de chance)')).toBeTruthy();

    // Named scenario: shows in the comparison and can be deleted.
    await user.type(screen.getByLabelText('Novo cenário'), 'Aos 60');
    await user.click(screen.getByRole('button', { name: 'Salvar como cenário' }));
    expect(await screen.findByText('Cenário “Aos 60” salvo')).toBeTruthy();
    const cmp = await screen.findByRole('table', { name: 'Comparar cenários' });
    expect(within(cmp).getByRole('button', { name: 'Aos 60' })).toBeTruthy();

    // Goal linked to the saved plan.
    await user.click(screen.getByRole('button', { name: 'Usar como plano principal' }));
    expect(await screen.findByText('Plano de aposentadoria salvo')).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Criar meta ligada ao plano' }));
    expect(await screen.findByText('Meta ligada ao plano')).toBeTruthy();
    expect(await screen.findByText(/Meta ligada:/)).toBeTruthy();
    const goals = (await mockCall('goals.list', {})) as {
      kind: string;
      linked_to_plan?: boolean;
    }[];
    expect(goals.some((g) => g.kind === 'retirement' && g.linked_to_plan)).toBe(true);

    await user.click(screen.getByRole('button', { name: 'Apagar cenário' }));
    await waitFor(() =>
      expect(screen.queryByRole('table', { name: 'Comparar cenários' })).toBeNull(),
    );
  });

  it('in the US it computes Social Security; in Portugal, the Segurança Social pension', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', { country: 'US' }, '#/orcamento?aba=aposentadoria');
    await user.type(await screen.findByLabelText('Idade hoje'), '45');
    await user.click(await screen.findByRole('combobox', { name: 'Social Security' }));
    await user.click(await screen.findByRole('option', { name: 'Calcular pelas regras' }));
    await user.type(screen.getByLabelText('Benefício na idade plena (PIA)'), '2500');
    const us = await screen.findByRole('table', { name: 'Regras: Social Security' });
    expect(within(us).getByText('Idade plena (FRA)')).toBeTruthy();
    expect(within(us).getByText('Pedido adiado (70 anos)')).toBeTruthy();
    expect(screen.queryByText('Contribuição em 13/11/2019')).toBeNull();
  });

  it('in Portugal it computes the old-age pension; in other countries only the entered amount', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', { country: 'PT' }, '#/orcamento?aba=aposentadoria');
    await user.type(await screen.findByLabelText('Idade hoje'), '45');
    await user.click(await screen.findByRole('combobox', { name: 'Segurança Social' }));
    await user.click(await screen.findByRole('option', { name: 'Calcular pelas regras' }));
    await user.type(screen.getByLabelText('Anos com descontos até hoje'), '20');
    await user.type(screen.getByLabelText('Remuneração de referência'), '1500');
    const pt = await screen.findByRole('table', { name: 'Regras: Segurança Social' });
    expect(within(pt).getByText('Idade pessoal de reforma')).toBeTruthy();

    renderApp('unlocked', { country: 'MX' }, '#/orcamento?aba=aposentadoria');
    await user.type(await screen.findByLabelText('Idade hoje'), '45');
    expect(
      await screen.findByLabelText('Valor mensal estimado (previdência pública)'),
    ).toBeTruthy();
  });
});
