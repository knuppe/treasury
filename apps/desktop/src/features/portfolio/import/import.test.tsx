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

describe('importação da Área do Investidor B3', () => {
  it('detecta, escolhe contas, revisa a prévia, grava e desfaz', async () => {
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

    // Contas: XP e Inter sugeridas pelo nome; a Nu Invest não tem conta e vai para a Inter.
    const xp = await within(dialog).findByRole('combobox', { name: /Conta para XP INVESTIMENTOS/ });
    expect(xp.textContent).toContain('XP Investimentos');
    const nu = within(dialog).getByRole('combobox', { name: /Conta para NU INVEST/ });
    expect(nu.textContent).toContain('Não importar');
    await user.click(nu);
    await user.click(await screen.findByRole('option', { name: /Inter Invest/ }));
    await user.click(within(dialog).getByRole('button', { name: 'Gerar prévia' }));

    // Prévia: operações com situações e filtros.
    const table = await within(dialog).findByRole('table', { name: 'Operações da importação' });
    expect(within(table).getByText('Possível duplicada')).toBeTruthy();
    expect(within(table).getByText(/venda de 300 maior que a posição/)).toBeTruthy();
    const filters = within(dialog).getByRole('group', { name: 'Filtrar operações por situação' });
    await user.click(within(filters).getByRole('button', { name: /Com erro 1/ }));
    expect(within(table).getAllByRole('row')).toHaveLength(2);
    await user.click(within(filters).getByRole('button', { name: /Todas/ }));

    const possible = within(table).getByRole('checkbox', { name: 'Importar operação 1' });
    expect(possible.hasAttribute('disabled')).toBe(true);
    await user.click(within(dialog).getByRole('switch', { name: /possíveis duplicados/ }));
    expect(possible.hasAttribute('disabled')).toBe(false);
    // Pula a compra de PETR4 de setembro.
    await user.click(within(table).getByRole('checkbox', { name: 'Importar operação 2' }));

    // Ativo novo com classe presumida.
    await user.click(within(dialog).getByRole('tab', { name: /Ativos/ }));
    const assets = await within(dialog).findByRole('table', { name: 'Ativos da importação' });
    expect(within(assets).getByText('KLBN11')).toBeTruthy();
    expect(within(assets).getByText('Conferir')).toBeTruthy();

    // Evento em revisão: aprova com o fator.
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
    expect(await within(dialog).findByText(/Importados da B3: 5 operação\(ões\)/)).toBeTruthy();
    expect(within(dialog).getByText(/1 evento\(s\); 1 ativo\(s\) cadastrado/)).toBeTruthy();
    const after = await trades();
    expect(after.length).toBe(before + 5);
    expect(after.some((t) => t.ticker === 'KLBN11')).toBe(true);
    expect(after.some((t) => t.date === '2026-09-03' && t.ticker === 'PETR4')).toBe(false);

    await user.click(within(dialog).getByRole('button', { name: 'Desfazer importação' }));
    await waitFor(async () => expect((await trades()).length).toBe(before));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Importar da B3' })).toBeNull(),
    );
  });
});

describe('importação da B3: ativo já cadastrado', () => {
  it('liga o ativo proposto a um já cadastrado em vez de criar outro', async () => {
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

describe('importação de notas de corretagem', () => {
  it('pede a senha, escolhe o ativo, pula, completa custos e desfaz', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/carteira?importar=notas');
    // A venda de HGLG11 já veio do relatório da B3, sem custos.
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

    // PDF protegido: a senha é pedida e explicada.
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

    // Ativo da ITAUSA escolhido à mão; a compra de PETR4 fica de fora.
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
    // A senha não fica no navegador.
    expect(JSON.stringify({ ...localStorage, ...sessionStorage })).not.toContain('123');

    await user.click(within(dialog).getByRole('button', { name: 'Desfazer importação' }));
    await waitFor(async () =>
      expect(await mockCall('portfolio.get_trade', { id: b3.id })).toMatchObject({
        fees: 0,
        note_number: null,
      }),
    );
  });

  it('avisa quando o líquido da nota não bate com o calculado', async () => {
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

  it('agrupa as execuções da mesma ordem e corrige o ativo de uma linha pelo código', async () => {
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

    // Reconhecido como VALE3; o usuário corrige para outro código.
    await user.click(within(table).getByRole('button', { name: 'Corrigir ativo' }));
    await user.type(within(table).getByLabelText('Código para VALE ON NM'), 'vale5');
    await user.click(within(table).getByRole('button', { name: 'Usar código' }));
    expect(await within(table).findByText('corrigido por você')).toBeTruthy();
    expect(within(table).getByText(/cadastra VALE5/)).toBeTruthy();

    // Sem agrupar: uma linha por execução.
    await user.click(within(dialog).getByRole('switch', { name: /Agrupar execuções/ }));
    await waitFor(() => expect(within(table).getAllByRole('checkbox')).toHaveLength(2));
  });
});
