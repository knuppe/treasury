/**
 * Relatórios › IR contra o mock: apuração mensal, gerar e pagar DARF, mês abaixo do
 * mínimo e o relatório anual com valores copiáveis. O aviso de que não é aconselhamento
 * tributário aparece sempre.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Darf } from '@/api/generated/Darf';
import type { TaxMonth } from '@/api/generated/TaxMonth';
import type { TxnPage } from '@/api/generated/TxnPage';
import { mockCall } from '@/api/mock';
import { fixToday } from '@/features/portfolio/testing';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

beforeEach(() => fixToday()); // 14/10/2026
afterEach(() => {
  vi.useRealTimers();
});

function open(search = '?v=ir') {
  return renderApp('unlocked', {}, `#/relatorios${search}`);
}

function hashParams(): URLSearchParams {
  return new URLSearchParams(window.location.hash.split('?')[1] ?? '');
}

describe('Relatórios › IR com o mock', () => {
  it('abre pela aba, mostra o aviso e a apuração do mês a pagar', async () => {
    const user = userEvent.setup();
    open('');
    await user.click(await screen.findByRole('tab', { name: /Imposto de renda/ }));
    await waitFor(() => expect(hashParams().get('v')).toBe('ir'));
    expect(await screen.findByText('Não é aconselhamento tributário')).toBeTruthy();
    expect(screen.getByText(/Não é aconselhamento tributário\./)).toBeTruthy();
    // Os filtros de despesas não valem para o IR.
    expect(screen.queryByRole('button', { name: /Período:/ })).toBeNull();
    const table = await screen.findByRole('grid', { name: 'Apuração mensal' });
    expect(within(table).getByText('jan/26')).toBeTruthy();
    expect(within(table).getByText('isento')).toBeTruthy();
    expect(within(table).getByText('Abaixo de R$ 10')).toBeTruthy();
    expect(within(table).getByText('Pago')).toBeTruthy();
    // Setembro: lucro em ações acima de R$ 20 mil, parte compensada pelo prejuízo de abril.
    const detail = screen.getByRole('region', { name: 'Apuração de setembro de 2026' });
    const pools = within(detail).getByRole('grid', { name: 'Grupos de prejuízo' });
    expect(within(pools).getByText('R$ 937,52')).toBeTruthy();
    expect(within(pools).getByText('15%')).toBeTruthy();
    expect(within(detail).getByText('Acumulado (< R$ 10)')).toBeTruthy();
    expect(within(detail).getByText('30/10/2026')).toBeTruthy();
    expect(within(detail).getByRole('button', { name: 'Gerar DARF de 386,74' })).toBeTruthy();
  });

  it('gera o DARF do mês e lista os DARFs com o vencido em destaque', async () => {
    const user = userEvent.setup();
    open();
    const detail = await screen.findByRole('region', { name: 'Apuração de setembro de 2026' });
    await user.click(within(detail).getByRole('button', { name: 'Gerar DARF de 386,74' }));
    expect(await screen.findByText('DARF gerado')).toBeTruthy();
    const m = (await mockCall('tax.month', { month: '2026-09' })) as TaxMonth;
    expect(m.darf_id).toBeTruthy();
    await user.click(await within(detail).findByRole('button', { name: 'Ver o DARF' }));
    const darfs = await screen.findByRole('grid', { name: 'DARFs' });
    expect(within(darfs).getAllByRole('row')).toHaveLength(4); // cabeçalho + 3
    expect(within(darfs).getByText('Vencido')).toBeTruthy();
    expect(screen.getByText(/Um DARF venceu sem pagamento/)).toBeTruthy();
    // O vencido mostra a multa e os juros de pagar hoje (Lei 9.430/1996 art. 61).
    expect(await within(darfs).findByText(/Pagando hoje/)).toBeTruthy();
    // Gerar de novo o mesmo mês: o núcleo recusa.
    await expect(mockCall('tax.generate_darf', { month: '2026-09' })).rejects.toMatchObject({
      code: 'conflict',
    });
  });

  it('marca o DARF vencido como pago lançando a despesa de IR', async () => {
    const user = userEvent.setup();
    open('?v=ir&ir=darf');
    const darfs = await screen.findByRole('grid', { name: 'DARFs' });
    await user.click(
      within(darfs).getByRole('button', { name: 'Marcar DARF de 2026-05 como pago' }),
    );
    const dialog = await screen.findByRole('dialog', { name: 'DARF de 2026-05 pago' });
    expect(within(dialog).getByText(/Pago depois do vencimento/)).toBeTruthy();
    await user.click(within(dialog).getByRole('combobox'));
    await user.click(await screen.findByRole('option', { name: /Itaú Conta Corrente/ }));
    await user.click(within(dialog).getByRole('button', { name: 'Marcar como pago' }));
    expect(await screen.findByText('DARF pago')).toBeTruthy();
    const list = (await mockCall('tax.list_darfs', { year: 2026 })) as Darf[];
    const may = list.find((d) => d.period === '2026-05')!;
    expect(may).toMatchObject({ status: 'paid', paid_at: '2026-10-14' });
    const page = (await mockCall('transactions.query', { text: 'DARF 6015' })) as TxnPage;
    expect(page.items[0]).toMatchObject({
      account_id: 'acc-itau',
      amount: -may.total,
      kind: 'expense',
      category_path: 'Impostos e taxas › Imposto de renda',
    });
    await waitFor(() => expect(screen.queryByText('Vencido')).toBeNull());
  });

  it('mês abaixo de R$ 10 não gera DARF: o valor vai para o mês seguinte', async () => {
    const user = userEvent.setup();
    open();
    const table = await screen.findByRole('grid', { name: 'Apuração mensal' });
    await user.click(within(table).getByText('ago/26'));
    const detail = await screen.findByRole('region', { name: 'Apuração de agosto de 2026' });
    expect(within(detail).getByText('Vai para o mês seguinte')).toBeTruthy();
    expect(within(detail).queryByRole('button', { name: /Gerar DARF/ })).toBeNull();
    expect(hashParams().get('mes')).toBe('2026-08');
    await expect(mockCall('tax.generate_darf', { month: '2026-08' })).rejects.toMatchObject({
      message: 'imposto abaixo de R$ 10,00: acumula para o mês seguinte',
    });
  });

  it('declaração anual: rendimentos, bens e direitos e valores para copiar', async () => {
    const user = userEvent.setup();
    const writeText = vi.fn(() => Promise.resolve());
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
    open('?v=ir&ir=declaracao');
    const income = await screen.findByRole('grid', { name: 'Rendimentos para a declaração' });
    expect(within(income).getByText('Lucros e dividendos recebidos')).toBeTruthy();
    expect(within(income).getByText('R$ 214,57')).toBeTruthy();
    await user.click(
      within(income).getByRole('button', {
        name: 'Copiar Ganhos líquidos em ações com vendas até R$ 20 mil no mês',
      }),
    );
    expect(writeText).toHaveBeenCalledWith('214,57');
    expect(await within(income).findByText('✓ Copiado')).toBeTruthy();
    const bens = screen.getByRole('grid', { name: 'Bens e direitos' });
    expect(within(bens).getByText('BBAS3')).toBeTruthy();
    await user.click(within(bens).getByRole('button', { name: 'Copiar discriminação de BBAS3' }));
    expect(writeText).toHaveBeenLastCalledWith(expect.stringMatching(/^Ações: 250 BBAS3/));
    expect(screen.getByText('Prejuízos a compensar em 31/12')).toBeTruthy();
    expect(screen.getByText('Não é aconselhamento tributário')).toBeTruthy();
  });
});

describe('09 · IR › Declaração de ajuste (prévia)', () => {
  it('mostra os dois modelos e refaz o cálculo quando um valor muda', async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    open('?v=ir&ir=declaracao&ano=2025');
    const card = await screen.findByRole('region', {
      name: /Declaração de ajuste de 2025/,
    });
    // Completo: 96 mil − (INSS 10 mil + dependente + instrução no teto + saúde 6 mil);
    // imposto 9.541,16, retido 10.800,00 → restituir 1.258,84.
    expect(await within(card).findByText('R$ 1.258,84')).toBeTruthy();
    const medical = within(card).getByLabelText('Despesas médicas');
    await user.clear(medical);
    await user.type(medical, '0');
    // Sem a saúde, o simplificado ganha: 10.938,77 devidos → 138,77 a pagar.
    expect(await within(card).findByText('R$ 138,77')).toBeTruthy();
    await user.click(within(card).getByRole('button', { name: 'Voltar aos valores do cofre' }));
    expect(await within(card).findByText('R$ 1.258,84')).toBeTruthy();
  });
});

describe('09 · IR › Carnê-leão do mês', () => {
  it('calcula pela tabela mensal com a redução de 2026', async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    open();
    const detail = await screen.findByRole('region', { name: 'Apuração de setembro de 2026' });
    await user.click(within(detail).getByText(/Carnê-leão do mês \(rendimentos/));
    await user.type(within(detail).getByLabelText('Recebido no mês'), '10.000,00');
    // 10.000 − 607,20 (simplificado) → 27,5% − 908,73 = 1.674,29; sem redução acima de 7.350.
    expect(await within(detail).findByText('R$ 1.674,29')).toBeTruthy();
    expect(within(detail).getByText(/código 0190/)).toBeTruthy();
  });
});
