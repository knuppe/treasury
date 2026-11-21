import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => ({
  EChart: ({ ariaLabel }: { ariaLabel: string }) => <div role="img" aria-label={ariaLabel} />,
}));

// Extrato sintético (empregador, CNPJ, datas e valores inventados), no layout da Caixa.
const STATEMENT = `EMPREGADOR INSCRICAO DO EMPREGADOR DATA DE ADMISSAO
EMPRESA EXEMPLO DE 98.765.432/0001-10 01/02/2023
TESTES S.A.
TIPO DE CONTA TAXA DE JUROS VALOR PARA FINS RESCISORIOS
OPTANTE 3.0% R$ 1.234,56
DATA LANCAMENTOS VALOR TOTAL
06/03/2023 115-DEPOSITO FEVEREIRO 2023 R$ 400,00 R$ 400,00
10/04/2023 CREDITO DE JAM 0,002466 R$ 0,98 R$ 400,98
06/04/2023 115-DEPOSITO MARCO 2023 R$ 400,00 R$ 800,98
15/04/2023 SAQUE DEP COD 50 -R$ 100,00 R$ 700,98
`;

describe('03 · Contas › FGTS', () => {
  it('o tipo FGTS só aparece num cofre do Brasil', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', { country: 'US' }, '#/contas');
    await screen.findByRole('grid', { name: 'Contas' });
    await expect(mockCall('accounts.create', { name: 'FGTS', kind: 'fgts' })).rejects.toThrow(
      /só existe no Brasil/,
    );
    await user.click(screen.getAllByRole('button', { name: /Nova conta/ })[0]!);
    const dialog = await screen.findByRole('dialog');
    await user.click(within(dialog).getAllByRole('combobox')[0]!);
    expect(screen.queryByRole('option', { name: /FGTS/ })).toBeNull();
  });

  it('importa o extrato da Caixa, mostra os lançamentos e o saque-aniversário', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/contas');
    await user.click((await screen.findAllByRole('button', { name: /Nova conta/ }))[0]!);
    const form = await screen.findByRole('dialog', { name: 'Nova conta' });
    await user.type(within(form).getByLabelText('Nome'), 'FGTS Exemplo');
    await user.click(within(form).getAllByRole('combobox')[0]!);
    await user.click(await screen.findByRole('option', { name: 'FGTS' }));
    await user.click(within(form).getByRole('button', { name: 'Criar conta' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Nova conta' })).toBeNull());
    const acc = ((await mockCall('accounts.list', {})) as AccountView[]).find(
      (a) => a.name === 'FGTS Exemplo',
    )!;
    expect(acc.kind).toBe('fgts');
    expect(acc.currency).toBe('BRL');
    // Criar a conta abre a página dela.
    expect(await screen.findByRole('tab', { name: 'Extrato do FGTS' })).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Importar extrato da Caixa' }));
    const dialog = await screen.findByRole('dialog', { name: 'Importar extrato do FGTS' });
    const file = new File([STATEMENT], 'extrato.pdf', { type: 'application/pdf' });
    await user.upload(within(dialog).getByLabelText('Extrato em PDF'), file);
    expect(await within(dialog).findByText('EMPRESA EXEMPLO DE TESTES S.A.')).toBeTruthy();
    expect(within(dialog).getByText('4 novos, 0 já importados')).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Gravar 4 lançamentos' }));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Importar extrato do FGTS' })).toBeNull(),
    );
    const table = await screen.findByRole('grid', { name: 'Lançamentos do FGTS' });
    expect(within(table).getAllByText('Depósito').length).toBe(2);
    expect(within(table).getByText(/Saque · Saque emergencial/)).toBeTruthy();
    const list = (await mockCall('fgts.list', {})) as { balance: number; employer_name: string }[];
    expect(list[0]!.balance).toBe(70_098);
    expect(list[0]!.employer_name).toBe('EMPRESA EXEMPLO DE TESTES S.A.');

    await user.click(screen.getByRole('tab', { name: 'Saques e saque-aniversário' }));
    // 700,98 na faixa de 500,01 a 1.000: 40% + 50 = 330,39.
    expect(await screen.findByText('Valor deste ano (pela tabela)')).toBeTruthy();
    const a = (await mockCall('fgts.anniversary_simulate', {})) as { amount: number };
    expect(a.amount).toBe(33_039);
  });
});
