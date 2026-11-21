import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { Asset } from '@/api/generated/Asset';
import type { PensionView } from '@/api/generated/PensionView';
import type { PortfolioSummary } from '@/api/generated/PortfolioSummary';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

async function seedPlan(): Promise<string> {
  const a = (await mockCall('assets.create', {
    name: 'VGBL Icatu Multimercado',
    class: 'pension',
  })) as Asset;
  for (const [date, qty] of [
    ['2014-03-10', '10000'],
    ['2026-01-10', '5000'],
  ] as const)
    await mockCall('portfolio.create_trade', {
      asset_id: a.id,
      account_id: 'acc-xp',
      date,
      side: 'buy',
      quantity: qty,
      price: '1',
      cash_movement: false,
    });
  await mockCall('market.set_price', { asset_id: a.id, date: '2026-10-15', close: '2' });
  return a.id;
}

describe('VGBL pension', () => {
  it('shows contributions by IR bracket, saves the plan and simulates the withdrawal', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/ativos');
    await screen.findByRole('grid', { name: 'Ativos cadastrados' });
    const id = await seedPlan();
    await queryClient.invalidateQueries();
    window.location.hash = `#/ativos?ativo=${id}&secao=previdencia`;
    const lots = await screen.findByRole('grid', { name: 'Aportes da previdência' });
    expect(within(lots).getByText('10%')).toBeTruthy();
    expect(within(lots).getByText('35%')).toBeTruthy();
    // No plan: warns and assumes regressive VGBL.
    expect(screen.getByLabelText('Avisos da previdência')).toBeTruthy();
    // The plan is the asset; the screen says which account holds the shares.
    expect(
      within(screen.getByLabelText('Conta das cotas do plano')).getByText(
        /As cotas deste plano ficam na conta/,
      ),
    ).toBeTruthy();

    const form = screen.getByRole('form', { name: 'Plano de previdência' });
    await user.click(within(form).getByRole('radio', { name: 'PGBL' }));
    await user.type(within(form).getByLabelText('Seguradora'), 'Icatu');
    await user.click(within(form).getByRole('button', { name: 'Salvar plano' }));
    await waitFor(async () =>
      expect(((await mockCall('pension.get', { asset_id: id })) as PensionView).plan).toMatchObject(
        {
          plan_type: 'pgbl',
          insurer: 'Icatu',
        },
      ),
    );

    const sim = screen.getByRole('form', { name: 'Simular resgate' });
    await user.type(within(sim).getByLabelText('Valor bruto a resgatar'), '20.000,00');
    await user.click(within(sim).getByRole('button', { name: 'Simular' }));
    const res = await screen.findByRole('status', { name: 'Resultado da simulação' });
    // PGBL: the base is the whole value; the oldest contribution (10%) comes out first.
    expect(within(res).getByText(/2\.000,00/)).toBeTruthy();
  });

  it('plan without a public share price: asks for the balance and the entered balance applies', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/ativos');
    await screen.findByRole('grid', { name: 'Ativos cadastrados' });
    const a = (await mockCall('assets.create', {
      name: 'Plano Fechado Exemplo',
      class: 'pension',
    })) as Asset;
    await mockCall('portfolio.create_trade', {
      asset_id: a.id,
      account_id: 'acc-xp',
      date: '2025-01-10',
      side: 'buy',
      quantity: '1000',
      price: '1',
      cash_movement: false,
    });
    const summary = async () =>
      (await mockCall('portfolio.summary', { deflator: 'none' })) as PortfolioSummary;
    let s = await summary();
    // No balance: a reminder, not the valued-at-cost warning.
    expect(s.balance_due.map((b) => b.name)).toContain('Plano Fechado Exemplo');
    expect(s.missing_prices).not.toContain('Plano Fechado Exemplo');
    await queryClient.invalidateQueries();
    window.location.hash = `#/ativos?ativo=${a.id}&secao=previdencia`;
    const form = await screen.findByRole('form', { name: 'Informar saldo' });
    await user.type(within(form).getByLabelText('Saldo'), '1.234,56');
    await user.click(within(form).getByRole('button', { name: 'Salvar saldo' }));
    expect(await screen.findByText('Saldo salvo')).toBeTruthy();
    s = await summary();
    expect(s.balance_due.map((b) => b.name)).not.toContain('Plano Fechado Exemplo');
    const pos = s.positions.find((p) => p.asset.id === a.id)!;
    expect(pos.market_value_base).toBe(123_456);
  });

  it('editing the pension shows the saved plan and saves its changes', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/ativos');
    await screen.findByRole('grid', { name: 'Ativos cadastrados' });
    const a = (await mockCall('assets.create', {
      name: 'VGBL Brasilprev',
      class: 'pension',
      pension: { plan_type: 'vgbl', tax_regime: 'regressive', insurer: 'Brasilprev' },
    })) as Asset;
    await queryClient.invalidateQueries();
    window.location.hash = `#/ativos?ativo=${a.id}`;
    await user.click(await screen.findByRole('button', { name: 'Editar' }));
    const dialog = await screen.findByRole('dialog', { name: /Editar VGBL Brasilprev/ });
    const plan = within(dialog).getByRole('group', { name: 'Plano de previdência' });
    expect(within(plan).getByLabelText<HTMLInputElement>('Seguradora').value).toBe('Brasilprev');
    expect(within(dialog).queryByLabelText('Ticker')).toBeNull();
    await user.click(within(plan).getByRole('radio', { name: 'Progressivo' }));
    // The record name is kept; the screen keeps the chosen name.
    await user.type(within(dialog).getByLabelText('Nome oficial'), 'BRASILPREV FIC PREV RF');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar ativo' }));
    expect(await screen.findByText('Nome oficial: BRASILPREV FIC PREV RF')).toBeTruthy();
    const found = (await mockCall('assets.list', { text: 'fic prev' })) as Asset[];
    expect(found.map((x) => x.name)).toEqual(['VGBL Brasilprev']);
    await waitFor(async () =>
      expect(
        ((await mockCall('pension.get', { asset_id: a.id })) as PensionView).plan,
      ).toMatchObject({ plan_type: 'vgbl', tax_regime: 'progressive', insurer: 'Brasilprev' }),
    );
    // A plan outside the pension category is not accepted.
    await expect(
      mockCall('assets.create', {
        name: 'Ação',
        class: 'stock_br',
        pension: { plan_type: 'vgbl', tax_regime: 'regressive' },
      }),
    ).rejects.toMatchObject({ code: 'invalid' });
  });

  it('the pension registration asks for the plan and opens its tab directly', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ativos?novo=ativo');
    const dialog = await screen.findByRole('dialog', { name: 'Cadastrar ativo' });
    await user.click(within(dialog).getByRole('radio', { name: 'Previdência' }));
    // Only what the pension uses: no ticker, exchange, sector or segment.
    for (const label of ['Ticker', 'Bolsa', 'Setor', 'Segmento'])
      expect(within(dialog).queryByLabelText(label)).toBeNull();
    const plan = within(dialog).getByRole('group', { name: 'Plano de previdência' });
    await user.type(within(dialog).getByLabelText('Nome do plano'), 'PGBL Icatu');
    await user.type(within(dialog).getByLabelText(/CNPJ do fundo/), '12.345.678/0001-90');
    await user.click(within(plan).getByRole('radio', { name: 'PGBL' }));
    await user.click(within(plan).getByRole('radio', { name: 'Progressivo' }));
    await user.type(within(plan).getByLabelText('Seguradora'), 'Icatu');
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar ativo' }));
    const form = await screen.findByRole('form', { name: 'Plano de previdência' });
    expect(window.location.hash).toContain('secao=previdencia');
    expect(within(form).getByRole('radio', { name: 'PGBL' }).getAttribute('aria-checked')).toBe(
      'true',
    );
    expect(
      within(form).getByRole('radio', { name: 'Progressivo' }).getAttribute('aria-checked'),
    ).toBe('true');
    expect(within(form).getByLabelText<HTMLInputElement>('Seguradora').value).toBe('Icatu');
  });

  it('imports the plan statement: preview, saving and nothing duplicated', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/ativos');
    await screen.findByRole('grid', { name: 'Ativos cadastrados' });
    const id = await seedPlan();
    await queryClient.invalidateQueries();
    window.location.hash = `#/ativos?ativo=${id}&secao=previdencia`;
    await user.click(await screen.findByRole('button', { name: 'Importar extrato (PDF)' }));
    const dialog = await screen.findByRole('dialog', { name: 'Importar extrato da previdência' });
    await user.upload(
      within(dialog).getByLabelText('Extrato em PDF'),
      new File(['%PDF-1.4 extrato'], 'extrato.pdf', { type: 'application/pdf' }),
    );
    const grid = await within(dialog).findByRole('grid', { name: 'Linhas do extrato' });
    expect(within(grid).getByText('Extra do empregador')).toBeTruthy();
    expect(within(grid).getByText('Só informativo')).toBeTruthy();
    expect(within(dialog).getByText('PLANOPREV-Arrojado')).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Importar 2 linha(s)' }));
    expect(await screen.findByText('Extrato importado')).toBeTruthy();
    const v = (await mockCall('pension.get', { asset_id: id })) as PensionView;
    expect(v.plan?.admin_fee_pct).toBe('0.25');

    // Again: the contributions are already in the app.
    await user.click(screen.getByRole('button', { name: 'Importar extrato (PDF)' }));
    const again = await screen.findByRole('dialog', { name: 'Importar extrato da previdência' });
    await user.upload(
      within(again).getByLabelText('Extrato em PDF'),
      new File(['%PDF-1.4 extrato'], 'extrato.pdf', { type: 'application/pdf' }),
    );
    expect(
      await within(again).findByRole('button', { name: 'Nada novo para importar' }),
    ).toHaveProperty('disabled', true);
  });

  it('the plan page shows neither Income nor Corporate events', async () => {
    const { queryClient } = renderApp('unlocked', {}, '#/ativos');
    await screen.findByRole('grid', { name: 'Ativos cadastrados' });
    const id = await seedPlan();
    await queryClient.invalidateQueries();
    // An old link to the income tab falls back to the Summary.
    window.location.hash = `#/ativos?ativo=${id}&secao=proventos`;
    const tabs = await screen.findByRole('tablist', { name: 'Seções do ativo' });
    expect(within(tabs).getByRole('tab', { name: 'Previdência' })).toBeTruthy();
    expect(
      within(tabs).getByRole('tab', { name: 'Visão geral' }).getAttribute('aria-selected'),
    ).toBe('true');
    expect(within(tabs).queryByRole('tab', { name: 'Proventos' })).toBeNull();
    expect(within(tabs).queryByRole('tab', { name: 'Eventos corporativos' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Provento' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Evento' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Nova operação' })).toBeTruthy();
  });
});
