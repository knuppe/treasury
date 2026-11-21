import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import type { IncomeEvent } from '@/api/generated/IncomeEvent';
import type { PendingIncome } from '@/api/generated/PendingIncome';
import { mockCall } from '@/api/mock';
import { formatDate } from '@/lib/format';
import { useCompanion } from '@/state/companion';
import { renderApp } from '@/test/renderApp';

import { maskAmount, maskDate, parseLooseDate, parsePasted, typeAmount } from './QuickIncomeDialog';
import { fixToday, quietCanvas } from './testing';

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
});
afterEach(() => {
  vi.useRealTimers();
});

describe('proventos em grade', () => {
  it('lê datas soltas e linhas coladas de planilha', () => {
    expect(parseLooseDate('15/10/2026')).toBe('2026-10-15');
    expect(parseLooseDate('1/2/26')).toBe('2026-02-01');
    expect(parseLooseDate('05/03', '2026-10-17')).toBe('2026-03-05');
    expect(parseLooseDate('2026-10-15')).toBe('2026-10-15');
    expect(parseLooseDate('31/02/2026')).toBeNull();
    expect(parseLooseDate('07/10/0720')).toBeNull();
    // Só o dia: mês e ano da linha de cima (ou de hoje).
    expect(parseLooseDate('08', '2026-10-15')).toBe('2026-10-08');
    // Dia/mês: o ano da última data digitada (linha de cima), não o de hoje.
    expect(parseLooseDate('12/12', '2025-12-19')).toBe('2025-12-12');
    expect(parseLooseDate('8', '2026-02-01')).toBe('2026-02-08');
    expect(parseLooseDate('31', '2026-02-01')).toBeNull();
    expect(maskAmount(12540)).toBe('125,40');
    expect(maskAmount(123450)).toBe('1.234,50');
    expect(typeAmount('1')).toBe('0,01');
    expect(typeAmount('1234')).toBe('12,34');
    expect(typeAmount('12,345')).toBe('123,45');
    expect(typeAmount('1.234,5')).toBe('123,45');
    expect(typeAmount('0,0')).toBe('');
    expect(maskDate('15')).toBe('15');
    expect(maskDate('151')).toBe('15/1');
    expect(maskDate('15102026')).toBe('15/10/2026');
    expect(maskDate('15/10/2026')).toBe('15/10/2026');
    expect(maskDate('151026')).toBe('15/10/26');
    expect(maskDate('2026-10-15')).toBe('2026-10-15');
    expect(parsePasted('15/10/2026\tITSA4\t25\nHGLG11;16/10/2026;1.100,00\n\n')).toEqual([
      { date: '15/10/2026', ticker: 'ITSA4', amount: '25' },
      { date: '16/10/2026', ticker: 'HGLG11', amount: '1.100,00' },
    ]);
  });

  it('digita data, ticker e valor e o Treasury completa o resto', async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime.bind(vi) });
    renderApp('unlocked', {}, '#/carteira?novo=proventos');
    const dialog = await screen.findByRole('dialog', { name: 'Registrar proventos' });
    await user.click(await within(dialog).findByRole('radio', { name: 'Digitar' }));
    const pending = (await mockCall('portfolio.pending_income', {})) as PendingIncome[];
    const petr = pending.find((p) => p.asset.ticker === 'PETR4')!;
    const amount = (petr.net / 100).toFixed(2).replace('.', ',');
    const digits = formatDate(petr.pay_date).replace(/\D/g, '');

    // Só os números: a máscara põe as barras.
    const date = within(dialog).getByLabelText<HTMLInputElement>('Data do pagamento, linha 1');
    await user.type(date, digits);
    expect(date.value).toBe(formatDate(petr.pay_date));
    // Sugestão do ticker: "pet" mostra PETR4; Enter escolhe e vai para o valor.
    await user.type(within(dialog).getByLabelText('Ticker, linha 1'), 'pet');
    const list = await screen.findByRole('listbox', { name: 'Ativos para a linha 1' });
    expect(within(list).getByRole('option', { name: /PETR4/ })).toBeTruthy();
    await user.keyboard('{Enter}');
    expect(within(dialog).getByLabelText<HTMLInputElement>('Ticker, linha 1').value).toBe('PETR4');
    expect(document.activeElement).toBe(within(dialog).getByLabelText('Valor recebido, linha 1'));
    await user.keyboard(amount);
    // Só dia/mês na linha 2: completa com o ano da linha 1 ao sair do campo.
    const d2 = within(dialog).getByLabelText<HTMLInputElement>('Data do pagamento, linha 2');
    await user.type(d2, formatDate(petr.pay_date).slice(0, 5));
    await user.tab();
    expect(d2.value).toBe(formatDate(petr.pay_date));
    // Segunda linha herda a data; ticker que não está na carteira.
    await user.type(within(dialog).getByLabelText('Ticker, linha 2'), 'XXXX9');
    await user.type(within(dialog).getByLabelText('Valor recebido, linha 2'), '10');

    expect(await within(dialog).findByText(/confere:/)).toBeTruthy();
    expect(within(dialog).getByText('Dividendos')).toBeTruthy();
    expect(within(dialog).getByText('XXXX9 não está cadastrado na carteira')).toBeTruthy();
    await waitFor(() =>
      expect(within(dialog).getByText(/1 pronta\(s\) · 1 com problema/)).toBeTruthy(),
    );

    await user.click(within(dialog).getByRole('button', { name: 'Registrar 1 provento' }));
    expect(await screen.findByText('1 provento(s) registrado(s)')).toBeTruthy();
    const income = (await mockCall('portfolio.list_income', {})) as IncomeEvent[];
    const rec = income.find((i) => i.ticker === 'PETR4' && i.pay_date === petr.pay_date)!;
    expect(rec.kind).toBe('dividend');
    expect(rec.ex_date).toBe(petr.ex_date);
    expect(rec.net).toBe(petr.net);
  });

  it('abre com os proventos anunciados prontos: desmarcar, corrigir o valor e confirmar', async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime.bind(vi) });
    const pending = (await (async () => {
      renderApp('unlocked', {}, '#/carteira?novo=proventos');
      return mockCall('portfolio.pending_income', {});
    })()) as PendingIncome[];
    expect(pending.length).toBeGreaterThan(1);
    const dialog = await screen.findByRole('dialog', { name: 'Registrar proventos' });
    const table = await within(dialog).findByRole('table', { name: 'Proventos a confirmar' });
    expect(
      within(dialog).getByRole('radio', { name: `A confirmar (${pending.length})` }),
    ).toBeTruthy();
    const [first, second] = pending;
    const label = (p: PendingIncome) => p.asset.ticker ?? p.asset.name;
    // Não recebeu o segundo; o primeiro veio com outro valor no extrato.
    await user.click(
      within(table).getByRole('checkbox', {
        name: new RegExp(`Recebi o provento de ${label(second!)}`),
      }),
    );
    const amount = within(table).getByLabelText<HTMLInputElement>(
      `Valor recebido de ${label(first!)}`,
    );
    await user.clear(amount);
    await user.type(amount, '1234');
    expect(amount.value).toBe('12,34');
    await user.click(
      within(dialog).getByRole('button', {
        name: `Confirmar ${pending.length - 1} provento${pending.length - 1 > 1 ? 's' : ''}`,
      }),
    );
    expect(await screen.findByText(`${pending.length - 1} provento(s) registrado(s)`)).toBeTruthy();
    const income = (await mockCall('portfolio.list_income', {})) as IncomeEvent[];
    const rec = income.find((i) => i.asset_id === first!.asset.id && i.ex_date === first!.ex_date)!;
    expect(rec.net).toBe(1234);
    expect(
      income.some((i) => i.asset_id === second!.asset.id && i.ex_date === second!.ex_date),
    ).toBe(false);
  });

  it('mostra a conferência entre fontes e pede ao agente para conferir as divergências', async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime.bind(vi) });
    renderApp('unlocked', {}, '#/carteira?novo=proventos');
    const dialog = await screen.findByRole('dialog', { name: 'Registrar proventos' });
    const table = await within(dialog).findByRole('table', { name: 'Proventos a confirmar' });
    expect(within(table).getByText(/fontes divergem/)).toBeTruthy();
    expect(within(table).getAllByText(/uma fonte/).length).toBeGreaterThan(0);
    await user.click(
      await within(dialog).findByRole('button', { name: 'Pedir ao agente para conferir' }),
    );
    expect(useCompanion.getState().open).toBe(true);
    // O pedido vai para a caixa de mensagem do agente (o usuário revisa e envia).
    const box = await screen.findByDisplayValue(/record_income_event/);
    expect((box as HTMLTextAreaElement).value).toContain('PETR4');
  });
});
