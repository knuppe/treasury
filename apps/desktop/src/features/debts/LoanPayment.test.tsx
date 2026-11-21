/**
 * 07 · Dívidas · parcela × lançamento contra o mock: ligar a parcela a uma saída pela tabela
 * de amortização (candidatos, menu da linha, desligar) e pelo lançamento
 * (`LoanPaymentDialog`: contrato, parcela sugerida, trocar e desligar).
 */
import { QueryClientProvider } from '@tanstack/react-query';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Installment } from '@/api/generated/Installment';
import type { TxnView } from '@/api/generated/TxnView';
import { mockCall, resetMock } from '@/api/mock';
import { createQueryClient } from '@/api/queryClient';
import { Toaster, useToastStore } from '@/design/components';
import { fixToday } from '@/features/portfolio/testing';
import { formatDate } from '@/lib/format';
import { addDays } from '@/lib/months';
import { clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

import { LoanPaymentDialog } from './LoanPaymentDialog';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

beforeEach(() => {
  fixToday(); // 14/10/2026
  clearCharts();
});
afterEach(() => {
  vi.useRealTimers();
});

const schedule = async (id: string) => (await mockCall('debt.schedule', { id })) as Installment[];
const installment = async (id: string, n: number) =>
  (await schedule(id)).find((i) => i.number === n)!;
const createTxn = async (args: Record<string, unknown>) =>
  (await mockCall('transactions.create', args)) as TxnView;
const txnView = async (id: string) => (await mockCall('transactions.get', { id })) as TxnView;

describe('parcela × lançamento pela tabela de amortização', () => {
  it('lista as saídas perto do vencimento, liga, troca pelo menu da linha e desliga', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/dividas?contrato=loan-2');
    const detail = await screen.findByRole('region', { name: 'Contrato CDC do Onix' });
    const table = await within(detail).findByRole('grid', { name: /Tabela de amortização/ });
    const p26 = await installment('loan-2', 26);
    expect(p26.paid).toBe(false);
    const exact = await createTxn({
      account_id: 'acc-itau',
      date: addDays(p26.due_date, 2),
      amount: -p26.total,
      description: 'DEB AUTOMATICO ONIX',
    });
    const pix = await createTxn({
      account_id: 'acc-nuconta',
      date: p26.due_date,
      amount: -(p26.total + 5_000),
      description: 'PIX ITAU VEICULOS',
    });

    await user.click(
      within(table).getByRole('button', { name: 'Ligar a parcela 26 a um lançamento' }),
    );
    let dialog = await screen.findByRole('dialog', { name: 'Ligar a parcela 26 a um lançamento' });
    let grid = await within(dialog).findByRole('grid', {
      name: 'Lançamentos para a parcela 26',
    });
    // A de valor igual vem primeiro (e já escolhida), marcada como da conta de débito.
    const first = within(grid).getAllByRole('row')[1]!;
    expect(within(first).getByText('DEB AUTOMATICO ONIX')).toBeTruthy();
    expect(within(first).getByText('igual')).toBeTruthy();
    expect(within(first).getByText('débito')).toBeTruthy();
    expect(within(grid).getByText('PIX ITAU VEICULOS')).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Ligar ao lançamento' }));
    expect(await screen.findByText('Parcela 26 ligada ao lançamento')).toBeTruthy();
    await waitFor(async () =>
      expect(await installment('loan-2', 26)).toMatchObject({
        paid: true,
        txn_id: exact.id,
        paid_amount: p26.total,
        paid_date: exact.date,
      }),
    );
    // A linha mostra o lançamento; em Lançamentos, o lançamento mostra a parcela.
    expect(
      await within(table).findByText(`${formatDate(exact.date)} · DEB AUTOMATICO ONIX`),
    ).toBeTruthy();
    expect(await txnView(exact.id)).toMatchObject({
      loan_id: 'loan-2',
      loan_name: 'CDC do Onix',
      loan_installment_number: 26,
    });

    // Menu da linha: trocar pelo PIX.
    await user.pointer({
      keys: '[MouseRight]',
      target: within(table).getByText(formatDate(p26.due_date)),
    });
    const menu = screen.getByRole('menu', { name: 'Ações da parcela 26' });
    expect(within(menu).getByRole('menuitem', { name: /Desligar o lançamento/ })).toBeTruthy();
    await user.click(within(menu).getByRole('menuitem', { name: /Ligar a outro lançamento/ }));
    dialog = await screen.findByRole('dialog', { name: 'Ligar a parcela 26 a um lançamento' });
    expect(within(dialog).getByRole('note', { name: 'Lançamento ligado' })).toBeTruthy();
    grid = await within(dialog).findByRole('grid', { name: 'Lançamentos para a parcela 26' });
    // O já ligado não é candidato.
    expect(within(grid).queryByText('DEB AUTOMATICO ONIX')).toBeNull();
    await user.click(within(grid).getByText('PIX ITAU VEICULOS'));
    await user.click(within(dialog).getByRole('button', { name: 'Trocar pelo lançamento' }));
    await waitFor(async () =>
      expect(await installment('loan-2', 26)).toMatchObject({
        txn_id: pix.id,
        paid_amount: p26.total + 5_000,
      }),
    );
    expect((await txnView(exact.id)).loan_installment_id).toBeNull();

    // Desligar: a parcela volta a ficar em aberto (atrasada).
    await user.click(
      await within(table).findByRole('button', { name: 'Desligar o lançamento da parcela 26' }),
    );
    const confirm = await screen.findByRole('dialog', {
      name: 'Desligar o lançamento da parcela 26',
    });
    await user.click(within(confirm).getByRole('button', { name: 'Desligar o lançamento' }));
    expect(await screen.findByText('Parcela 26 desligada do lançamento e em aberto')).toBeTruthy();
    await waitFor(async () =>
      expect(await installment('loan-2', 26)).toMatchObject({ paid: false, txn_id: null }),
    );
    expect((await txnView(pix.id)).loan_installment_id).toBeNull();
  });

  it('apagar o lançamento ligado reabre a parcela', async () => {
    resetMock('unlocked');
    const p26 = await installment('loan-2', 26);
    const t = await createTxn({
      account_id: 'acc-itau',
      date: p26.due_date,
      amount: -p26.total,
      description: 'Parcela',
    });
    await mockCall('debt.link_payment', { installment_id: p26.id, txn_id: t.id });
    expect((await installment('loan-2', 26)).paid).toBe(true);
    await expect(
      mockCall('debt.link_payment', {
        installment_id: (await installment('loan-2', 27)).id,
        txn_id: t.id,
      }),
    ).rejects.toThrow(/já está ligado à parcela 26 de CDC do Onix/);
    await mockCall('transactions.delete', { ids: [t.id] });
    expect(await installment('loan-2', 26)).toMatchObject({ paid: false, txn_id: null });
  });
});

function openDialog(txn: TxnView) {
  useToastStore.getState().clear();
  const onOpenChange = vi.fn();
  const onDone = vi.fn();
  render(
    <QueryClientProvider client={createQueryClient()}>
      <MemoryRouter>
        <LoanPaymentDialog txn={txn} open onOpenChange={onOpenChange} onDone={onDone} />
        <Toaster />
      </MemoryRouter>
    </QueryClientProvider>,
  );
  return { onOpenChange, onDone };
}

describe('LoanPaymentDialog (Pagamento de parcela… no lançamento)', () => {
  it('escolhe o contrato, sugere a parcela em aberto mais perto e liga', async () => {
    const user = userEvent.setup();
    resetMock('unlocked');
    const p26 = await installment('loan-2', 26);
    const t = await createTxn({
      account_id: 'acc-itau',
      date: addDays(p26.due_date, 3),
      amount: -(p26.total + 1_200),
      description: 'PAGTO ONIX',
    });
    const { onDone, onOpenChange } = openDialog(t);
    const dialog = await screen.findByRole('dialog', { name: 'Pagamento de parcela' });
    // Dois contratos debitam na mesma conta: o usuário escolhe.
    await user.click(await within(dialog).findByRole('combobox', { name: 'Contrato' }));
    await user.click(await screen.findByRole('option', { name: 'CDC do Onix' }));
    const grid = await within(dialog).findByRole('grid', { name: 'Parcelas de CDC do Onix' });
    expect(within(grid).getByText(formatDate(p26.due_date))).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Ligar à parcela 26' }));
    await waitFor(() =>
      expect(onDone).toHaveBeenCalledWith('Lançamento ligado à parcela 26 de CDC do Onix'),
    );
    expect(onOpenChange).toHaveBeenCalledWith(false);
    expect(await installment('loan-2', 26)).toMatchObject({
      paid: true,
      txn_id: t.id,
      paid_amount: p26.total + 1_200,
    });
  });

  it('lançamento já ligado: mostra a parcela, troca por outra e desliga', async () => {
    const user = userEvent.setup();
    resetMock('unlocked');
    const p26 = await installment('loan-2', 26);
    const p27 = await installment('loan-2', 27);
    const t = await createTxn({
      account_id: 'acc-itau',
      date: p26.due_date,
      amount: -p26.total,
      description: 'PAGTO ONIX',
    });
    await mockCall('debt.link_payment', { installment_id: p26.id, txn_id: t.id });
    openDialog(await txnView(t.id));
    const dialog = await screen.findByRole('dialog', { name: 'Pagamento de parcela' });
    expect(
      within(dialog).getByText('Este lançamento paga a parcela 26 de CDC do Onix.'),
    ).toBeTruthy();
    const grid = await within(dialog).findByRole('grid', { name: 'Parcelas de CDC do Onix' });
    expect(within(grid).getByText('✓ Este lançamento')).toBeTruthy();
    expect(
      within(dialog).getByRole<HTMLButtonElement>('button', { name: 'Ligar à parcela 26' })
        .disabled,
    ).toBe(true);
    await user.click(within(grid).getByText(formatDate(p27.due_date)));
    await user.click(within(dialog).getByRole('button', { name: 'Trocar para a parcela 27' }));
    await waitFor(async () =>
      expect(await installment('loan-2', 27)).toMatchObject({ paid: true, txn_id: t.id }),
    );
    expect(await installment('loan-2', 26)).toMatchObject({ paid: false, txn_id: null });
  });

  it('desliga pelo aviso: a parcela volta a ficar em aberto', async () => {
    const user = userEvent.setup();
    resetMock('unlocked');
    const p26 = await installment('loan-2', 26);
    const t = await createTxn({
      account_id: 'acc-itau',
      date: p26.due_date,
      amount: -p26.total,
      description: 'PAGTO ONIX',
    });
    await mockCall('debt.link_payment', { installment_id: p26.id, txn_id: t.id });
    const { onDone } = openDialog(await txnView(t.id));
    const dialog = await screen.findByRole('dialog', { name: 'Pagamento de parcela' });
    await user.click(within(dialog).getByRole('button', { name: 'Desligar' }));
    await waitFor(() =>
      expect(onDone).toHaveBeenCalledWith('Lançamento desligado da parcela 26 de CDC do Onix'),
    );
    expect(await installment('loan-2', 26)).toMatchObject({ paid: false, txn_id: null });
  });

  it('entrada não paga parcela: avisa e não deixa ligar', async () => {
    resetMock('unlocked');
    const t = await createTxn({
      account_id: 'acc-itau',
      date: '2026-10-05',
      amount: 50_000,
      description: 'Estorno',
    });
    openDialog(t);
    const dialog = await screen.findByRole('dialog', { name: 'Pagamento de parcela' });
    expect(
      within(dialog).getByText('Só uma saída (valor negativo) pode pagar uma parcela.'),
    ).toBeTruthy();
    for (const b of within(dialog).getAllByRole<HTMLButtonElement>('button', {
      name: /^Ligar à parcela/,
    }))
      expect(b.disabled).toBe(true);
  });

  it('pelo menu do lançamento em 04 · Lançamentos, com a parcela marcada na linha', async () => {
    const user = userEvent.setup();
    const { queryClient } = renderApp('unlocked', {}, '#/lancamentos?q=PARCELA ONIX MENU');
    const grid = await screen.findByRole('grid', { name: 'Lançamentos' });
    const p26 = await installment('loan-2', 26);
    await createTxn({
      account_id: 'acc-itau',
      date: addDays(p26.due_date, 1),
      amount: -p26.total,
      description: 'PARCELA ONIX MENU',
    });
    await queryClient.invalidateQueries({ queryKey: ['transactions'] });
    await user.pointer({
      keys: '[MouseRight]',
      target: await within(grid).findByText('PARCELA ONIX MENU'),
    });
    await user.click(
      within(screen.getByRole('menu', { name: 'Ações de PARCELA ONIX MENU' })).getByRole(
        'menuitem',
        { name: /Pagamento de parcela de contrato/ },
      ),
    );
    const dialog = await screen.findByRole('dialog', { name: 'Pagamento de parcela' });
    await user.click(await within(dialog).findByRole('combobox', { name: 'Contrato' }));
    await user.click(await screen.findByRole('option', { name: 'CDC do Onix' }));
    await user.click(await within(dialog).findByRole('button', { name: 'Ligar à parcela 26' }));
    expect(await within(grid).findByLabelText('Parcela 26 de CDC do Onix')).toBeTruthy();
  });
});
