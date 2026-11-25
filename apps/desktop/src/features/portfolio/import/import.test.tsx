import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Trade } from '@/api/generated/Trade';
import { mockCall } from '@/api/mock';
import { SAMPLE_NOTE } from '@/api/mock/notes';
import { renderApp } from '@/test/renderApp';

import { PORTFOLIO_UI_DEFAULTS, usePortfolioUi } from '../state';
import { fixToday, quietCanvas } from '../testing';

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
  usePortfolioUi.setState({ ...PORTFOLIO_UI_DEFAULTS });
});
afterEach(() => {
  vi.useRealTimers();
});

const trades = async (args: Record<string, unknown> = {}) =>
  (await mockCall('portfolio.list_trades', { limit: 5000, ...args })) as Trade[];

describe('B3 Área do Investidor import', () => {
  it('detects, chooses accounts, reviews the preview, saves and undoes', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/carteira?aba=operacoes');
    await user.click(await screen.findByRole('button', { name: 'Importar da B3' }));
    const dialog = await screen.findByRole('dialog', { name: 'Importar da B3' });
    const before = (await trades()).length;

    await user.upload(within(dialog).getByLabelText('Relatórios da Área do Investidor'), [
      new File(['x'], 'negociacao-2026-10.xlsx'),
      new File(['x'], 'movimentacao-2026-10.xlsx'),
    ]);
    const list = await within(dialog).findByRole('list', { name: 'Relatórios escolhidos' });
    expect(within(list).getByText('negociacao-2026-10.xlsx')).toBeTruthy();
    expect(within(list).getAllByText(/NU INVEST CORRETORA/).length).toBeGreaterThan(0);
    await user.click(within(dialog).getByRole('button', { name: 'Continuar' }));

    // Accounts: XP and Inter suggested by name; Nu Invest has no account and goes to Inter.
    const xp = await within(dialog).findByRole('combobox', { name: /Conta para XP INVESTIMENTOS/ });
    expect(xp.textContent).toContain('XP Investimentos');
    const nu = within(dialog).getByRole('combobox', { name: /Conta para NU INVEST/ });
    expect(nu.textContent).toContain('Não importar');
    await user.click(nu);
    await user.click(await screen.findByRole('option', { name: /Inter Invest/ }));
    await user.click(within(dialog).getByRole('button', { name: 'Gerar prévia' }));

    // Preview: operations with statuses and filters.
    const table = await within(dialog).findByRole('table', { name: 'Operações da importação' });
    expect(within(table).getByText('Possível duplicada')).toBeTruthy();
    // A sale without its purchase is imported by default, flagged as a pending item.
    expect(within(table).getByText(/Venda de 300 maior que a posição/)).toBeTruthy();
    const filters = within(dialog).getByRole('group', { name: 'Filtrar operações por situação' });
    await user.click(within(filters).getByRole('button', { name: /Com pendência 1/ }));
    expect(within(table).getAllByRole('row')).toHaveLength(2);
    const pendingSale = within(table).getByRole('checkbox', { name: /Importar operação/ });
    expect((pendingSale as HTMLInputElement).checked).toBe(true);
    expect(pendingSale.hasAttribute('disabled')).toBe(false);
    await user.click(within(filters).getByRole('button', { name: /Todas/ }));

    const possible = within(table).getByRole('checkbox', { name: 'Importar operação 1' });
    expect(possible.hasAttribute('disabled')).toBe(true);
    await user.click(within(dialog).getByRole('switch', { name: /possíveis duplicados/ }));
    expect(possible.hasAttribute('disabled')).toBe(false);
    // Skip the September PETR4 purchase.
    await user.click(within(table).getByRole('checkbox', { name: 'Importar operação 2' }));

    // New asset with a presumed class.
    await user.click(within(dialog).getByRole('tab', { name: /Ativos/ }));
    const assets = await within(dialog).findByRole('table', { name: 'Ativos da importação' });
    expect(within(assets).getByText('KLBN11')).toBeTruthy();
    expect(within(assets).getByText('Conferir')).toBeTruthy();

    // Event under review: approves with the factor.
    await user.click(within(dialog).getByRole('tab', { name: /Eventos/ }));
    const events = await within(dialog).findByRole('table', {
      name: 'Eventos corporativos da importação',
    });
    expect(within(events).getByText('Revisar')).toBeTruthy();
    await user.click(within(events).getByRole('checkbox', { name: 'Aprovar' }));
    await user.type(within(events).getByLabelText(/Fator do evento/), '0,1');

    await user.click(within(dialog).getByRole('tab', { name: /Ignoradas/ }));
    expect(await within(dialog).findByText('Opção de Compra · PETRA300 · 100')).toBeTruthy();
    expect(within(dialog).getByText('Liquidação já importada')).toBeTruthy();

    const commit = within(dialog).getByRole('button', { name: /^Importar \d+ lançamento/ });
    await user.click(commit);
    expect(await within(dialog).findByText(/Importados da B3: 6 operação\(ões\)/)).toBeTruthy();
    expect(within(dialog).getByText(/1 evento\(s\); 1 ativo\(s\) cadastrado/)).toBeTruthy();
    // The result says what was left to explain and where to solve it.
    expect(within(dialog).getByText('1 venda ficou sem compra registrada')).toBeTruthy();
    expect(within(dialog).getByRole('link', { name: 'Ver em Carteira › Vendas' })).toBeTruthy();
    const after = await trades();
    expect(after.length).toBe(before + 6);
    expect(after.some((t) => t.ticker === 'KLBN11')).toBe(true);
    expect(after.some((t) => t.date === '2026-09-03' && t.ticker === 'PETR4')).toBe(false);

    await user.click(within(dialog).getByRole('button', { name: 'Desfazer importação' }));
    await waitFor(async () => expect((await trades()).length).toBe(before));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Importar da B3' })).toBeNull(),
    );
  });
});

describe('B3 import: accounts step', () => {
  it('creates a new account from the pre-filled form and shows a checking account by name', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/carteira?aba=operacoes');
    await user.click(await screen.findByRole('button', { name: 'Importar da B3' }));
    const dialog = await screen.findByRole('dialog', { name: 'Importar da B3' });
    const accountsBefore = ((await mockCall('accounts.list', {})) as unknown[]).length;
    await user.upload(within(dialog).getByLabelText('Relatórios da Área do Investidor'), [
      new File(['x'], 'negociacao-2026-10.xlsx'),
      new File(['x'], 'movimentacao-2026-10.xlsx'),
    ]);
    await within(dialog).findByRole('list', { name: 'Relatórios escolhidos' });
    await user.click(within(dialog).getByRole('button', { name: 'Continuar' }));

    // Nu Invest has no account: "Create a new account" opens the form, pre-filled.
    const nu = await within(dialog).findByRole('combobox', { name: /Conta para NU INVEST/ });
    await user.click(nu);
    await user.click(await screen.findByRole('option', { name: 'Criar nova conta…' }));
    const form = within(dialog).getByRole('group', { name: /Conta nova para NU INVEST/ });
    expect(within(form).getByLabelText('Nome da conta')).toHaveProperty('value', 'Nu Invest');
    expect(within(form).getByLabelText('Instituição')).toHaveProperty('value', 'Nu Invest');
    expect(within(form).getByRole('combobox', { name: 'Tipo' }).textContent).toContain('Corretora');

    // A checking account chosen for Inter shows its name, never the id.
    const inter = within(dialog).getByRole('combobox', { name: /Conta para INTER/ });
    await user.click(inter);
    await user.click(await screen.findByRole('option', { name: /Nubank Conta/ }));
    expect(inter.textContent).toContain('Nubank Conta');
    expect(inter.textContent).not.toMatch(/acc-/);

    await user.click(within(dialog).getByRole('button', { name: 'Gerar prévia' }));
    await within(dialog).findByRole('table', { name: 'Operações da importação' });
    expect(within(dialog).getByText(/NU → Nu Invest/)).toBeTruthy();
    expect(within(dialog).getByText(/INTER → Nubank Conta/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: /^Importar \d+ lançamento/ }));
    expect(await within(dialog).findByText(/Importados da B3/)).toBeTruthy();
    const accounts = (await mockCall('accounts.list', {})) as { name: string }[];
    expect(accounts.length).toBe(accountsBefore + 1);
    expect(accounts.some((a) => a.name === 'Nu Invest')).toBe(true);
  });
});

describe('B3 import: already registered asset', () => {
  it('links the proposed asset to an already registered one instead of creating another', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/carteira?aba=operacoes');
    await user.click(await screen.findByRole('button', { name: 'Importar da B3' }));
    const dialog = await screen.findByRole('dialog', { name: 'Importar da B3' });
    await user.upload(within(dialog).getByLabelText('Relatórios da Área do Investidor'), [
      new File(['x'], 'negociacao-2026-10.xlsx'),
    ]);
    await within(dialog).findByRole('list', { name: 'Relatórios escolhidos' });
    await user.click(within(dialog).getByRole('button', { name: 'Continuar' }));
    await user.click(await within(dialog).findByRole('button', { name: 'Gerar prévia' }));
    await user.click(await within(dialog).findByRole('tab', { name: /Ativos/ }));
    const assets = await within(dialog).findByRole('table', { name: 'Ativos da importação' });
    await user.click(
      within(assets).getByRole('combobox', { name: /Ativo cadastrado para KLBN11/ }),
    );
    await user.click(await screen.findByRole('option', { name: /^WEGE3/ }));
    expect(within(assets).queryByRole('combobox', { name: /Classe de KLBN11/ })).toBeNull();
    await user.click(within(dialog).getByRole('button', { name: /^Importar \d+ lançamento/ }));
    expect(await within(dialog).findByText(/Importados da B3/)).toBeTruthy();
    const after = await trades();
    expect(after.some((t) => t.ticker === 'KLBN11')).toBe(false);
  });
});

describe('brokerage note import', () => {
  it('asks for the password, chooses the asset, skips, fills in costs and undoes', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/carteira?importar=notas');
    // The HGLG11 sale already came from the B3 report, with no costs.
    const b3 = (await mockCall('portfolio.create_trade', {
      asset_id: 'ast-hglg11',
      account_id: 'acc-xp',
      date: '2026-10-13',
      side: 'sell',
      quantity: '10',
      price: '160',
      cash_movement: false,
    })) as Trade;
    const dialog = await screen.findByRole('dialog', { name: 'Importar notas de corretagem' });
    const account = within(dialog).getByRole('combobox', { name: 'Conta da corretora' });
    if (!account.textContent?.includes('XP Investimentos')) {
      await user.click(account);
      await user.click(await screen.findByRole('option', { name: /XP Investimentos/ }));
    }
    await user.upload(
      within(dialog).getByLabelText('Notas de corretagem (PDF)'),
      new File([SAMPLE_NOTE], 'nota-123456-senha.pdf', { type: 'application/pdf' }),
    );

    // Protected PDF: the password is requested and explained.
    expect(await within(dialog).findByText('o PDF é protegido por senha')).toBeTruthy();
    expect(within(dialog).getByText(/3 primeiros dígitos do CPF/)).toBeTruthy();
    expect(
      within(dialog)
        .getByRole('button', { name: 'Gravar 0 operação(ões)' })
        .hasAttribute('disabled'),
    ).toBe(true);
    await user.type(within(dialog).getByLabelText('Senha do PDF'), '123');
    await user.click(within(dialog).getByRole('button', { name: 'Abrir com a senha' }));

    const note = await within(dialog).findByRole('region', { name: 'Nota 123456' });
    expect(within(note).getByText('Líquido da nota')).toBeTruthy();
    expect(within(dialog).queryByText(/Líquido da nota ≠/)).toBeNull();
    const table = within(dialog).getByRole('table', { name: 'Operações das notas' });
    expect(within(table).getByText('Nova')).toBeTruthy();
    expect(within(table).getByText('Sem ativo')).toBeTruthy();
    expect(within(table).getByText('Completar custos')).toBeTruthy();
    expect(within(dialog).getByRole('button', { name: 'Gravar 2 operação(ões)' })).toBeTruthy();

    // ITAUSA asset chosen by hand; the PETR4 purchase is left out.
    const fix = within(dialog).getByRole('group', { name: 'Ativo para ITAUSA PN N1' });
    await user.click(within(fix).getByRole('combobox', { name: 'Ativo cadastrado' }));
    await user.click(await screen.findByRole('option', { name: /ITUB4 · Itaú Unibanco/ }));
    expect(await within(table).findByText('corrigido por você')).toBeTruthy();
    expect(within(table).queryByText('Sem ativo')).toBeNull();
    expect(
      await within(dialog).findByRole('button', { name: 'Gravar 3 operação(ões)' }),
    ).toBeTruthy();
    await user.click(within(table).getByRole('checkbox', { name: 'Gravar operação 1' }));
    expect(
      within(dialog).getByRole('switch', { name: 'Lançar no caixa da corretora' }),
    ).toBeTruthy();
    expect(
      within(dialog)
        .getByRole('switch', { name: 'Lançar no caixa da corretora' })
        .getAttribute('aria-checked'),
    ).toBe('false');
    await user.click(within(dialog).getByRole('button', { name: 'Gravar 2 operação(ões)' }));

    expect(
      await within(dialog).findByText(/1 operação\(ões\) nova\(s\) e 1 completada\(s\)/),
    ).toBeTruthy();
    const filled = (await mockCall('portfolio.get_trade', { id: b3.id })) as Trade;
    expect(filled).toMatchObject({ fees: 42, taxes: 8, note_number: '123456' });
    const xp = await trades({ account_id: 'acc-xp' });
    expect(xp.some((t) => t.ticker === 'ITUB4' && t.date === '2026-10-13')).toBe(true);
    expect(xp.some((t) => t.ticker === 'PETR4' && t.date === '2026-10-13')).toBe(false);
    // The password does not stay in the browser.
    expect(JSON.stringify({ ...localStorage, ...sessionStorage })).not.toContain('123');

    await user.click(within(dialog).getByRole('button', { name: 'Desfazer importação' }));
    await waitFor(async () =>
      expect(await mockCall('portfolio.get_trade', { id: b3.id })).toMatchObject({
        fees: 0,
        note_number: null,
      }),
    );
  });

  it("warns when the note's net amount does not match the calculated one", async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/carteira?importar=notas');
    const dialog = await screen.findByRole('dialog', { name: 'Importar notas de corretagem' });
    await waitFor(() =>
      expect(
        within(dialog).getByRole('combobox', { name: 'Conta da corretora' }).textContent,
      ).toContain('Investimentos'),
    );
    await user.upload(
      within(dialog).getByLabelText('Notas de corretagem (PDF)'),
      new File(['%PDF-1.4 binário'], 'nota-incompleta.pdf', { type: 'application/pdf' }),
    );
    expect(await within(dialog).findByText(/Líquido da nota ≠ líquido calculado/)).toBeTruthy();
    expect(within(dialog).getByText(/não bate com as operações e custos lidos/)).toBeTruthy();
  });

  it("groups the executions of the same order and fixes a line's asset by code", async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/carteira?importar=notas');
    const dialog = await screen.findByRole('dialog', { name: 'Importar notas de corretagem' });
    await waitFor(() =>
      expect(
        within(dialog).getByRole('combobox', { name: 'Conta da corretora' }).textContent,
      ).toContain('Investimentos'),
    );
    const text = [
      'NOTA DE NEGOCIAÇÃO',
      'Nr. nota Folha Data pregão',
      '555 1 14/10/2026',
      '1-BOVESPA C VISTA VALE ON NM 10 60,00 600,00 D',
      '1-BOVESPA C VISTA VALE ON NM @ 30 62,00 1.860,00 D',
      'Taxa de liquidação 0,50 D',
      'Líquido para 16/10/2026 2.460,50 D',
    ].join('\n');
    await user.upload(
      within(dialog).getByLabelText('Notas de corretagem (PDF)'),
      new File([text], 'nota-555.txt', { type: 'text/plain' }),
    );
    const table = await within(dialog).findByRole('table', { name: 'Operações das notas' });
    expect(within(table).getByText('2 execuções')).toBeTruthy();
    expect(within(table).getAllByRole('checkbox')).toHaveLength(1);

    // Recognized as VALE3; the user corrects it to another code.
    await user.click(within(table).getByRole('button', { name: 'Corrigir ativo' }));
    await user.type(within(table).getByLabelText('Código para VALE ON NM'), 'vale5');
    await user.click(within(table).getByRole('button', { name: 'Usar código' }));
    expect(await within(table).findByText('corrigido por você')).toBeTruthy();
    expect(within(table).getByText(/cadastra VALE5/)).toBeTruthy();

    // Without grouping: one line per execution.
    await user.click(within(dialog).getByRole('switch', { name: /Agrupar execuções/ }));
    await waitFor(() => expect(within(table).getAllByRole('checkbox')).toHaveLength(2));
  });
});
