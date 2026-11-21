/**
 * 07 · Dívidas contra o mock: lista de contratos, detalhe com figuras e tabela, parcela
 * atrasada, simulador de amortização, amortização extraordinária e cadastro de contrato.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Installment } from '@/api/generated/Installment';
import type { LoanView } from '@/api/generated/LoanView';
import type { TxnPage } from '@/api/generated/TxnPage';
import { mockCall, mockCallLog } from '@/api/mock';
import { fixToday } from '@/features/portfolio/testing';
import { chartOption, chartProps, clearCharts } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

beforeEach(() => {
  fixToday(); // 14/10/2026
  clearCharts();
});
afterEach(() => {
  vi.useRealTimers();
});

function open(search = '') {
  return renderApp('unlocked', {}, `#/dividas${search}`);
}

function hashParams(): URLSearchParams {
  return new URLSearchParams(window.location.hash.split('?')[1] ?? '');
}

describe('07 · Dívidas com o mock', () => {
  it('lista os contratos e abre o primeiro com medidas, figuras e a tabela de amortização', async () => {
    const user = userEvent.setup();
    open();
    expect(await screen.findByRole('heading', { level: 1, name: /Dívidas/ })).toBeTruthy();
    const list = await screen.findByRole('grid', { name: 'Contratos' });
    for (const name of ['Financiamento do apartamento', 'CDC do Onix', 'Consignado'])
      expect(within(list).getAllByText(name).length).toBeGreaterThan(0);
    expect(within(list).getByText('SAC · TR')).toBeTruthy();
    expect(within(list).getAllByText('Price (TP) · pré')).toHaveLength(2);
    expect(within(list).getByText(/! atrasada/)).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Saldo devedor total' })).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Compromisso mensal' })).toBeTruthy();

    // Detalhe do financiamento (o primeiro contrato).
    const detail = screen.getByRole('region', { name: 'Contrato Financiamento do apartamento' });
    expect(await within(detail).findByText(/Caixa · de Ana/)).toBeTruthy();
    expect(within(detail).getByText('CET realizado')).toBeTruthy();
    expect(within(detail).getByText(/correção TR \(no aniversário\)/)).toBeTruthy();
    const table = await within(detail).findByRole('grid', {
      name: 'Tabela de amortização · Financiamento do apartamento',
    });
    // Recorte padrão: 3 últimas pagas + 12 próximas.
    expect(within(table).getAllByText('Paga')).toHaveLength(3);
    expect(within(table).getByText('Próxima')).toBeTruthy();
    // Meses sem TR divulgada: parcela e saldo estimados (≈).
    expect(within(table).getAllByText('índice estimado').length).toBeGreaterThan(0);
    // A amortização com FGTS aparece na parcela em que entrou.
    await user.click(within(detail).getByRole('radio', { name: /^Pagas · 42/ }));
    expect(within(table).getAllByText('Paga')).toHaveLength(42);
    expect(within(table).getByText(/\+ extra/)).toBeTruthy();

    // Figuras: saldo devedor e composição com correção e seguros, num eixo só.
    expect(chartProps(/^Saldo devedor de Financiamento do apartamento/)).toBeTruthy();
    const opt = chartOption(/^Composição das parcelas de Financiamento/);
    const series = opt.series as { name: string }[];
    expect(series.map((s) => s.name)).toEqual([
      'Amortização',
      'Juros',
      'Correção do saldo',
      'Seguros e taxas',
    ]);
    expect(Array.isArray(opt.yAxis)).toBe(false);
  });

  it('parcela atrasada: avisa e some depois de marcar como paga', async () => {
    const user = userEvent.setup();
    open('?contrato=loan-2');
    const detail = await screen.findByRole('region', { name: 'Contrato CDC do Onix' });
    expect(within(detail).getByText('1 parcela atrasada')).toBeTruthy();
    const table = await within(detail).findByRole('grid', { name: /Tabela de amortização/ });
    expect(within(table).getByText('Atrasada')).toBeTruthy();
    await user.click(within(table).getByRole('button', { name: /Marcar parcela 26 como paga/ }));
    const dialog = await screen.findByRole('dialog', { name: 'Parcela 26 paga' });
    await user.click(within(dialog).getByRole('button', { name: 'Marcar como paga' }));
    expect(await screen.findByText('Parcela 26 marcada como paga')).toBeTruthy();
    await waitFor(() => expect(screen.queryByText('1 parcela atrasada')).toBeNull());
    const v = (await mockCall('debt.get', { id: 'loan-2' })) as LoanView;
    expect(v.overdue_count).toBe(0);
    const sched = (await mockCall('debt.schedule', { id: 'loan-2' })) as Installment[];
    const p26 = sched.find((i) => i.number === 26)!;
    // A data do pagamento já vem com o vencimento da parcela.
    expect(p26).toMatchObject({ paid: true, paid_date: p26.due_date });
  });

  it('categoria ligada ao contrato: o lançamento nela paga a parcela atrasada', async () => {
    open('?contrato=loan-2');
    await screen.findByRole('region', { name: 'Contrato CDC do Onix' });
    await mockCall('debt.update', { id: 'loan-2', category_id: 'cat-impostos-juros' });
    await expect(
      mockCall('debt.update', { id: 'loan-1', category_id: 'cat-impostos-juros' }),
    ).rejects.toThrow(/já está ligada ao contrato CDC do Onix/);
    const before = (await mockCall('debt.schedule', { id: 'loan-2' })) as Installment[];
    const p26 = before.find((i) => i.number === 26)!;
    expect(p26.paid).toBe(false);
    const t = (await mockCall('transactions.create', {
      account_id: 'acc-nuconta',
      date: p26.due_date,
      amount: -(p26.total + 1500),
      description: 'PARCELA ONIX',
      category_id: 'cat-impostos-juros',
    })) as { id: string };
    const after = (await mockCall('debt.schedule', { id: 'loan-2' })) as Installment[];
    expect(after.find((i) => i.number === 26)).toMatchObject({
      paid: true,
      txn_id: t.id,
      paid_amount: p26.total + 1500,
    });
    expect(((await mockCall('debt.get', { id: 'loan-2' })) as LoanView).overdue_count).toBe(0);
  });

  it('financiamento imobiliário: MIP sobre o saldo e DFI sobre o imóvel na parcela', async () => {
    open();
    await screen.findByRole('heading', { level: 1, name: /Dívidas/ });
    const l = (await mockCall('debt.create', {
      name: 'Apartamento',
      kind: 'mortgage',
      principal: 40_000_000,
      annual_rate: '12',
      system: 'sac',
      contract_date: '2026-01-05',
      first_due_date: '2026-02-05',
      term_months: 360,
      fees_monthly: 2_500,
      property_value: 50_000_000,
      mip_rate: '0,0247',
      dfi_rate: '0,0075',
    })) as LoanView;
    const sched = (await mockCall('debt.schedule', { id: l.id })) as Installment[];
    expect(sched[0]).toMatchObject({ insurance_mip: 9_880, insurance_dfi: 3_750, fees: 2_500 });
    expect(sched.at(-1)!.insurance_dfi).toBe(3_750);
    await expect(mockCall('debt.update', { id: l.id, property_value: null })).rejects.toThrow(
      /valor do imóvel/,
    );
  });

  it('financiamento imobiliário com MIP e DFI em reais por mês, como no contrato', async () => {
    const user = userEvent.setup();
    open('?novo=contrato');
    const dialog = await screen.findByRole('dialog', { name: 'Novo contrato' });
    await user.type(within(dialog).getByLabelText('Nome do contrato'), 'Casa');
    await user.click(within(dialog).getByRole('combobox', { name: /Tipo/ }));
    await user.click(await screen.findByRole('option', { name: /imobiliário/i }));
    await user.type(within(dialog).getByLabelText('Valor financiado'), '300.000,00');
    await user.click(within(dialog).getByRole('radio', { name: 'Informar a taxa' }));
    await user.type(within(dialog).getByLabelText('Taxa efetiva (% a.a.)'), '9');
    await user.type(within(dialog).getByLabelText('Prazo (meses)'), '360');
    const mip = within(dialog).getByRole('radiogroup', { name: 'Como o contrato traz o MIP' });
    await user.click(within(mip).getByRole('radio', { name: 'R$' }));
    await user.type(within(dialog).getByLabelText('Seguro MIP (R$ por mês)'), '120,00');
    // O app mostra o percentual equivalente sobre o saldo.
    expect(within(dialog).getByText(/Equivale a 0,04% a\.m\./)).toBeTruthy();
    const dfi = within(dialog).getByRole('radiogroup', { name: 'Como o contrato traz o DFI' });
    await user.click(within(dfi).getByRole('radio', { name: 'R$' }));
    await user.type(within(dialog).getByLabelText('Seguro DFI (R$ por mês)'), '30,00');
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar contrato' }));
    expect(await screen.findByText('Contrato cadastrado')).toBeTruthy();
    await waitFor(() => expect(hashParams().get('contrato')).toBeTruthy());
    const id = hashParams().get('contrato')!;
    const v = (await mockCall('debt.get', { id })) as LoanView;
    expect(v).toMatchObject({ mip_amount: 12_000, dfi_amount: 3_000, mip_rate: null });
    const sched = (await mockCall('debt.schedule', { id })) as Installment[];
    expect(sched[0]).toMatchObject({ insurance_mip: 12_000, insurance_dfi: 3_000 });
    expect(sched.at(-1)!.insurance_mip).toBe(12_000);
  });

  it('importa o extrato do contrato: parcelas do banco, amortização achada e projeção do saldo real', async () => {
    const user = userEvent.setup();
    const { queryClient } = open();
    await screen.findByRole('heading', { level: 1, name: /Dívidas/ });
    const l = (await mockCall('debt.create', {
      name: 'Casa Caixa',
      kind: 'mortgage',
      principal: 10_000_000,
      annual_rate: '12',
      system: 'price',
      contract_date: '2025-12-10',
      first_due_date: '2026-01-10',
      term_months: 120,
      fees_monthly: 2_500,
    })) as LoanView;
    await queryClient.invalidateQueries();
    window.location.hash = `#/dividas?contrato=${l.id}`;
    const detail = await screen.findByRole('region', { name: 'Contrato Casa Caixa' });
    await user.click(within(detail).getByRole('button', { name: 'Importar extrato' }));
    const dialog = await screen.findByRole('dialog', { name: 'Importar extrato do contrato' });
    const text = [
      'Data\tPrestação\tAmortização\tJuros\tSeguros\tTaxas\tSaldo devedor\tCorreção monetária\tValor pago',
      '10/01/2026\t1.461,40\t484,80\t951,60\t0,00\t25,00\t99.515,20\t0,00\t1.461,40',
      '10/02/2026\t1.461,40\t489,41\t946,99\t0,00\t25,00\t99.025,79\t0,00\t1.461,40',
      '10/03/2026\t1.461,40\t494,07\t942,33\t0,00\t25,00\t78.531,72\t0,00\t1.500,00',
    ].join('\n');
    await user.click(within(dialog).getByLabelText('Extrato colado'));
    await user.paste(text);
    await user.type(within(dialog).getByLabelText('Parcelas que faltam'), '60');
    await user.click(within(dialog).getByRole('button', { name: 'Conferir' }));
    const prev = await within(dialog).findByRole('region', { name: 'Prévia do extrato' });
    const found = within(prev).getByRole('list', { name: 'Amortizações achadas' });
    expect(within(found).getByText(/20\.000,00/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: /Importar 3 parcela/ }));
    expect(await screen.findByText('3 parcela(s) do extrato registradas')).toBeTruthy();
    const v = (await mockCall('debt.get', { id: l.id })) as LoanView;
    expect(v.statement_until).toBe(3);
    expect(v.remaining_count).toBe(60);
    const sched = (await mockCall('debt.schedule', { id: l.id })) as Installment[];
    expect(sched.find((i) => i.number === 3)).toMatchObject({
      paid: true,
      paid_amount: 150_000,
      extra_payment: 2_000_000,
      balance_after: 7_853_172,
    });
  });

  it('pagamento com o saldo do comprovante: sincroniza e a amortização abre a versão 2', async () => {
    const user = userEvent.setup();
    const { queryClient } = open();
    await screen.findByRole('heading', { level: 1, name: /Dívidas/ });
    const l = (await mockCall('debt.create', {
      name: 'Casa Caixa',
      kind: 'mortgage',
      principal: 10_000_000,
      annual_rate: '12',
      system: 'price',
      contract_date: '2025-12-10',
      first_due_date: '2026-01-10',
      term_months: 120,
    })) as LoanView;
    await queryClient.invalidateQueries();
    window.location.hash = `#/dividas?contrato=${l.id}&filtro=todas`;
    const detail = await screen.findByRole('region', { name: 'Contrato Casa Caixa' });
    const table = await within(detail).findByRole('grid', { name: /Tabela de amortização/ });
    await user.click(within(table).getByRole('button', { name: /Marcar parcela 2 como paga/ }));
    const dialog = await screen.findByRole('dialog', { name: 'Parcela 2 paga' });
    await user.type(
      within(dialog).getByLabelText('Saldo devedor depois do pagamento'),
      '79.025,79',
    );
    await user.type(within(dialog).getByLabelText('Amortização da parcela'), '489,41');
    await user.type(within(dialog).getByLabelText('Parcelas que faltam'), '100');
    await user.click(within(dialog).getByRole('button', { name: 'Marcar como paga' }));
    expect(await screen.findByText('Parcela 2 paga e saldo sincronizado')).toBeTruthy();
    const sched = (await mockCall('debt.schedule', { id: l.id })) as Installment[];
    // 100 parcelas depois da 2 (a 1 continua em aberto).
    expect(Math.max(...sched.map((i) => i.number))).toBe(102);
    expect(sched.find((i) => i.number === 2)).toMatchObject({
      paid: true,
      balance_after: 7_902_579,
      // O que o saldo caiu além da amortização da parcela.
      extra_payment: sched.find((i) => i.number === 1)!.balance_after - 48_941 - 7_902_579,
      version: 2,
      version_number: 1,
    });
    expect(sched.find((i) => i.number === 3)).toMatchObject({ version: 2, version_number: 2 });
    expect(sched.find((i) => i.number === 1)).toMatchObject({ version: 1, version_number: 1 });
    await waitFor(() => expect(within(table).getAllByText('v2').length).toBeGreaterThan(0));
  });

  it('cadastro de contrato antigo: marca as parcelas vencidas como pagas', async () => {
    const user = userEvent.setup();
    open('?novo=contrato');
    const dialog = await screen.findByRole('dialog', { name: 'Novo contrato' });
    await user.type(within(dialog).getByLabelText('Nome do contrato'), 'Consignado antigo');
    await user.type(within(dialog).getByLabelText('Valor financiado'), '12.000,00');
    await user.type(within(dialog).getByLabelText('Prazo (meses)'), '24');
    await user.click(within(dialog).getByRole('radio', { name: 'Informar a taxa' }));
    await user.type(within(dialog).getByLabelText('Taxa efetiva (% a.a.)'), '20');
    const due = within(dialog).getByLabelText('Primeiro vencimento');
    await user.clear(due);
    await user.type(due, '10/01/2026');
    const sw = await within(dialog).findByRole('switch', {
      name: /Já paguei as \d+ parcelas vencidas/,
    });
    expect(sw.getAttribute('aria-checked')).toBe('true');
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar contrato' }));
    expect(await screen.findByText('Contrato cadastrado')).toBeTruthy();
    await waitFor(() => expect(hashParams().get('novo')).toBeNull());
    const v = (await mockCall('debt.get', { id: hashParams().get('contrato')! })) as LoanView;
    expect(v.overdue_count).toBe(0);
    expect(v.paid_count).toBeGreaterThanOrEqual(9);
  });

  it('antecipa as duas últimas parcelas com desconto dos juros', async () => {
    const user = userEvent.setup();
    open('?contrato=loan-3');
    const detail = await screen.findByRole('region', { name: /Contrato Consignado/ });
    const before = (await mockCall('debt.get', { id: 'loan-3' })) as LoanView;
    const card = await within(detail).findByRole('region', { name: /Antecipar parcelas/ });
    const count = within(card).getByLabelText('Parcelas');
    await user.clear(count);
    await user.type(count, '2');
    expect(await within(card).findByText(/o desconto dos juros é de/)).toBeTruthy();
    await user.click(within(card).getByRole('button', { name: 'Registrar antecipação' }));
    expect(await screen.findByText('Parcelas antecipadas')).toBeTruthy();
    const sched = (await mockCall('debt.schedule', { id: 'loan-3' })) as Installment[];
    const last2 = sched.slice(-2);
    expect(last2.every((i) => i.paid && (i.paid_amount ?? 0) < i.total)).toBe(true);
    const after = (await mockCall('debt.get', { id: 'loan-3' })) as LoanView;
    expect(after.interest_remaining).toBeLessThan(before.interest_remaining);
  });

  it('simula as duas formas de amortizar e registra a escolhida com o lançamento na conta', async () => {
    const user = userEvent.setup();
    open('?contrato=loan-1');
    const detail = await screen.findByRole('region', {
      name: 'Contrato Financiamento do apartamento',
    });
    await user.type(await within(detail).findByLabelText('Valor da amortização'), '50.000,00');
    const choices = await within(detail).findByRole('radiogroup', { name: 'Forma de amortizar' });
    const term = within(choices).getByRole('radio', { name: 'Reduzir prazo' });
    const inst = within(choices).getByRole('radio', { name: 'Reduzir parcela' });
    await waitFor(() => expect(within(term).getByText('Juros economizados')).toBeTruthy());
    await waitFor(() => expect(within(inst).getByText('Juros economizados')).toBeTruthy());
    expect(within(term).getByText(/a menos$/)).toBeTruthy();
    expect(term.getAttribute('aria-checked')).toBe('true');
    // O saldo simulado entra na figura do saldo.
    await waitFor(() => expect(chartProps(/com a amortização simulada\.$/)).toBeTruthy());
    const [before, after] = await Promise.all(
      (['reduce_term', 'reduce_installment'] as const).map((mode) =>
        mockCall('debt.simulate_prepayment', {
          loan_id: 'loan-1',
          date: '2026-10-14',
          amount: 5_000_000,
          mode,
        }),
      ),
    );
    const t = before as { interest_saved: number; months_after: number; months_before: number };
    const i = after as { interest_saved: number; months_after: number };
    expect(t.interest_saved).toBeGreaterThan(i.interest_saved);
    expect(t.months_after).toBeLessThan(t.months_before);
    expect(i.months_after).toBe(t.months_before);

    await user.click(inst);
    await user.click(within(detail).getByRole('button', { name: 'Registrar esta amortização' }));
    const dialog = await screen.findByRole('dialog', { name: 'Amortização extraordinária' });
    expect(
      within(dialog).getByRole('radio', { name: 'Reduzir parcela' }).getAttribute('aria-checked'),
    ).toBe('true');
    await user.click(within(dialog).getByRole('button', { name: 'Registrar amortização' }));
    expect(await screen.findByText('Amortização registrada')).toBeTruthy();
    const v = (await mockCall('debt.get', { id: 'loan-1' })) as LoanView;
    expect(v.next_installment!.total).toBe(
      (after as { next_payment_after: number }).next_payment_after,
    );
    const page = (await mockCall('transactions.query', {
      text: 'Amortização extraordinária',
    })) as TxnPage;
    expect(page.items[0]).toMatchObject({ account_id: 'acc-itau', amount: -5_000_000 });
  });

  it('simula a portabilidade com nova taxa e custos e diz se compensa', async () => {
    const user = userEvent.setup();
    open('?contrato=loan-2');
    const detail = await screen.findByRole('region', { name: /Simulador de portabilidade/ });
    expect(within(detail).getByText(/A simulação não grava nada/)).toBeTruthy();
    await user.type(within(detail).getByLabelText('Nova taxa (% a.a.)'), '1');
    const status = await within(detail).findByRole('status');
    expect(status.textContent).toMatch(/Vale a pena: economia de/);
    expect(within(detail).getByText('Novo prazo')).toBeTruthy();
    // Custos altos demais viram prejuízo.
    const costs = within(detail).getByLabelText('Custos da troca');
    await user.clear(costs);
    await user.type(costs, '50.000,00');
    await waitFor(() =>
      expect(within(detail).getByRole('status').textContent).toMatch(/Não compensa/),
    );
    expect(mockCallLog().some((c) => c.startsWith('debt.simulate_portability'))).toBe(true);
  });

  it('cadastra um contrato Price e abre o cronograma dele', async () => {
    const user = userEvent.setup();
    open('?novo=contrato');
    const dialog = await screen.findByRole('dialog', { name: 'Novo contrato' });
    await user.type(within(dialog).getByLabelText('Nome do contrato'), 'Reforma da cozinha');
    await user.type(within(dialog).getByLabelText('Valor financiado'), '24.000,00');
    await user.click(within(dialog).getByRole('radio', { name: 'Informar a taxa' }));
    await user.type(within(dialog).getByLabelText('Taxa efetiva (% a.a.)'), '18');
    await user.type(within(dialog).getByLabelText('Prazo (meses)'), '24');
    // Prévia da parcela antes de gravar.
    expect(within(dialog).getByText('Parcela')).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar contrato' }));
    expect(await screen.findByText('Contrato cadastrado')).toBeTruthy();
    await waitFor(() => expect(hashParams().get('novo')).toBeNull());
    const id = hashParams().get('contrato')!;
    const v = (await mockCall('debt.get', { id })) as LoanView;
    expect(v).toMatchObject({
      name: 'Reforma da cozinha',
      principal: 2_400_000,
      annual_rate: '18',
      system: 'price',
      remaining_count: 24,
    });
    expect(await screen.findByRole('region', { name: 'Contrato Reforma da cozinha' })).toBeTruthy();
    expect(mockCallLog()).toContain('debt.create');
  });

  it('edita todos os campos do contrato e recalcula sem perder as parcelas pagas', async () => {
    const user = userEvent.setup();
    open('?contrato=loan-2');
    const detail = await screen.findByRole('region', { name: 'Contrato CDC do Onix' });
    const before = (await mockCall('debt.get', { id: 'loan-2' })) as LoanView;
    const paidBefore = (
      (await mockCall('debt.schedule', { id: 'loan-2' })) as Installment[]
    ).filter((i) => i.paid);
    expect(paidBefore.length).toBeGreaterThan(0);
    await user.click(within(detail).getByRole('button', { name: 'Editar' }));
    const dialog = await screen.findByRole('dialog', { name: 'Editar contrato' });
    // Campos do contrato assinado também ficam editáveis.
    const term = within(dialog).getByLabelText('Prazo (meses)');
    expect(term.hasAttribute('disabled')).toBe(false);
    await user.clear(term);
    await user.type(term, String(before.term_months + 12));
    const rate = within(dialog).getByLabelText('Taxa efetiva (% a.a.)');
    await user.clear(rate);
    await user.type(rate, '15');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar alterações' }));
    expect(await screen.findByText('Contrato alterado')).toBeTruthy();
    const after = (await mockCall('debt.get', { id: 'loan-2' })) as LoanView;
    expect(after.term_months).toBe(before.term_months + 12);
    expect(Number(after.annual_rate)).toBe(15);
    const sched = (await mockCall('debt.schedule', { id: 'loan-2' })) as Installment[];
    // As pagas continuam pagas, com o valor e a data registrados.
    for (const p of paidBefore) {
      expect(sched.find((i) => i.id === p.id)).toMatchObject({
        paid: true,
        paid_date: p.paid_date,
        paid_amount: p.paid_amount,
      });
    }
    expect(mockCallLog()).toContain('debt.update');
  });

  it('calcula a taxa pela parcela e grava o número do contrato', async () => {
    const user = userEvent.setup();
    open('?novo=contrato');
    const dialog = await screen.findByRole('dialog', { name: 'Novo contrato' });
    await user.type(within(dialog).getByLabelText('Nome do contrato'), 'CDC do notebook');
    await user.type(within(dialog).getByLabelText('Número do contrato'), '0012345-6');
    await user.type(within(dialog).getByLabelText('Valor financiado'), '60.000,00');
    await user.type(within(dialog).getByLabelText('Prazo (meses)'), '48');
    await user.type(within(dialog).getByLabelText('Valor da parcela'), '1.580,02');
    // 60 mil em 48× de 1.580,02 na Price ≈ 1% a.m.
    expect(within(dialog).getByTestId('taxa-calculada').textContent).toMatch(
      /12,68% a\.a\. · 1% a\.m\./,
    );
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar contrato' }));
    expect(await screen.findByText('Contrato cadastrado')).toBeTruthy();
    await waitFor(() => expect(hashParams().get('novo')).toBeNull());
    const v = (await mockCall('debt.get', { id: hashParams().get('contrato')! })) as LoanView;
    expect(v.contract_number).toBe('0012345-6');
    expect(Number(v.annual_rate)).toBeCloseTo(12.6825, 3);
    expect(v.next_installment?.total).toBeGreaterThanOrEqual(158_000);
    expect(v.next_installment?.total).toBeLessThanOrEqual(158_004);
    // A data do contrato acompanhou o 1º vencimento (um mês antes).
    expect(v.contract_date < v.first_due_date).toBe(true);
  });

  it('cadastra um pós-fixado CDI + spread com juros por período e dias úteis', async () => {
    const user = userEvent.setup();
    open('?novo=contrato');
    const dialog = await screen.findByRole('dialog', { name: 'Novo contrato' });
    await user.type(within(dialog).getByLabelText('Nome do contrato'), 'Capital de giro');
    await user.type(within(dialog).getByLabelText('Valor financiado'), '100.000,00');
    await user.type(within(dialog).getByLabelText('Prazo (meses)'), '12');
    await user.click(within(dialog).getByRole('radio', { name: 'Bullet' }));
    await user.click(within(dialog).getByRole('radio', { name: 'Pós-fixada (CDI, Selic)' }));
    expect(within(dialog).queryByLabelText('Valor da parcela')).toBeNull();
    await user.type(within(dialog).getByLabelText('Spread (% a.a.)'), '2');
    expect(within(dialog).getByText(/A parcela varia com o CDI/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar contrato' }));
    expect(await screen.findByText('Contrato cadastrado')).toBeTruthy();
    await waitFor(() => expect(hashParams().get('novo')).toBeNull());
    const id = hashParams().get('contrato')!;
    const v = (await mockCall('debt.get', { id })) as LoanView;
    expect(v).toMatchObject({
      indexer: 'cdi',
      system: 'bullet',
      annual_rate: '2',
      index_pct: '100',
    });
    // A tabela mostra os juros do período (índice × spread) e os dias úteis.
    const region = await screen.findByRole('region', { name: 'Contrato Capital de giro' });
    expect(within(region).getByText('CDI + 2% a.a.')).toBeTruthy();
    expect((await within(region).findAllByText(/dias úteis$/)).length).toBeGreaterThan(0);
  });

  it('IPCA com índice de M − 2 e piso zero: grava a convenção escolhida', async () => {
    const user = userEvent.setup();
    open('?novo=contrato');
    const dialog = await screen.findByRole('dialog', { name: 'Novo contrato' });
    await user.type(within(dialog).getByLabelText('Nome do contrato'), 'Imóvel IPCA');
    await user.type(within(dialog).getByLabelText('Valor financiado'), '300.000,00');
    await user.type(within(dialog).getByLabelText('Prazo (meses)'), '240');
    await user.click(within(dialog).getByRole('radio', { name: 'Informar a taxa' }));
    await user.type(within(dialog).getByLabelText('Taxa efetiva (% a.a.)'), '5');
    await user.click(within(dialog).getByRole('combobox', { name: 'Correção do saldo' }));
    await user.click(await screen.findByRole('option', { name: 'IPCA' }));
    await user.click(within(dialog).getByRole('combobox', { name: 'Mês do índice' }));
    await user.click(await screen.findByRole('option', { name: 'M − 2' }));
    await user.click(within(dialog).getByRole('switch', { name: /Ignorar deflação/ }));
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar contrato' }));
    expect(await screen.findByText('Contrato cadastrado')).toBeTruthy();
    await waitFor(() => expect(hashParams().get('novo')).toBeNull());
    const v = (await mockCall('debt.get', { id: hashParams().get('contrato')! })) as LoanView;
    expect(v).toMatchObject({
      indexer: 'ipca',
      correction_lag_months: 2,
      deflation_floor: true,
      correction_reference: 'month',
    });
  });

  it('parcela que não cobre o valor financiado é recusada antes de gravar', async () => {
    const user = userEvent.setup();
    open('?novo=contrato');
    const dialog = await screen.findByRole('dialog', { name: 'Novo contrato' });
    await user.type(within(dialog).getByLabelText('Nome do contrato'), 'Teste');
    await user.type(within(dialog).getByLabelText('Valor financiado'), '60.000,00');
    await user.type(within(dialog).getByLabelText('Prazo (meses)'), '48');
    await user.type(within(dialog).getByLabelText('Valor da parcela'), '1.000,00');
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar contrato' }));
    expect(within(dialog).getByText(/a taxa não fecha/)).toBeTruthy();
    expect(mockCallLog()).not.toContain('debt.create');
  });

  it('valida o formulário antes de chamar o núcleo', async () => {
    const user = userEvent.setup();
    open('?novo=contrato');
    const dialog = await screen.findByRole('dialog', { name: 'Novo contrato' });
    await user.type(within(dialog).getByLabelText('Prazo (meses)'), '12');
    await user.clear(within(dialog).getByLabelText('Carência (meses)'));
    await user.type(within(dialog).getByLabelText('Carência (meses)'), '12');
    await user.click(within(dialog).getByRole('button', { name: 'Cadastrar contrato' }));
    expect(within(dialog).getByText('Dê um nome ao contrato.')).toBeTruthy();
    expect(within(dialog).getByText('A carência deve ser menor que o prazo.')).toBeTruthy();
    expect(mockCallLog()).not.toContain('debt.create');
  });
});
