/**
 * 16 · Calculadoras com o mock: as quatro abas, as referências oficiais com data e o botão que
 * preenche a taxa, e as contas na tela (os números exatos estão em features/calculators/math.test.ts).
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { renderApp } from '@/test/renderApp';

const open = (q = '') => renderApp('unlocked', {}, `#/calculadoras${q}`);
/** A frase do resultado (com o valor num elemento próprio). */
const statusMatching = (re: RegExp) =>
  screen.getAllByRole('status').find((el) => re.test(el.textContent ?? ''));

describe('16 · Calculadoras', () => {
  it('juros compostos: referências com data e o botão que preenche a taxa', async () => {
    const user = userEvent.setup();
    open();
    expect(await screen.findByRole('heading', { level: 1, name: /Calculadoras/ })).toBeTruthy();
    const refs = await screen.findByRole('region', { name: 'Referências oficiais' });
    expect(within(refs).getByText('CDI anualizado')).toBeTruthy();
    expect(within(refs).getByText(/14,90% em 30\/10\/2026/)).toBeTruthy();
    expect(within(refs).getByText('Inflação 12 meses (IPCA)')).toBeTruthy();
    const tabs = screen.getByRole('tablist', { name: 'Calculadoras' });
    expect(within(tabs).getAllByRole('tab')).toHaveLength(4);
    // Padrão: 10.000 + 500/mês a 10% a.a. por 10 anos.
    expect(screen.getByText('Saldo no fim')).toBeTruthy();
    const rate = screen.getByLabelText<HTMLInputElement>('Rendimento ao ano (%)');
    await user.click(screen.getByRole('button', { name: 'Usar CDI anualizado' }));
    await waitFor(() => expect(rate.value).toBe('14,9'));
    expect(screen.getByRole('grid', { name: 'Evolução ano a ano' })).toBeTruthy();
  });

  it('à vista × parcelado mostra os juros escondidos e quem custa menos hoje', async () => {
    open('?aba=parcelado');
    expect(await screen.findByText('Juros escondidos na parcela')).toBeTruthy();
    // 2.700 à vista contra 10 × 300: juros ao mês e ao ano.
    expect(screen.getByText(/ao mês \(.*ao ano\)/)).toBeTruthy();
    expect(statusMatching(/custa .* a menos em dinheiro de hoje/)).toBeTruthy();
  });

  it('viver de renda e alugar × comprar calculam na hora', async () => {
    const user = userEvent.setup();
    open('?aba=renda');
    expect(await screen.findByText('Patrimônio que paga o gasto só com o rendimento')).toBeTruthy();
    expect(
      screen.getByRole('link', { name: 'Ver o plano de aposentadoria completo' }),
    ).toBeTruthy();
    await user.click(screen.getByRole('tab', { name: 'Alugar × comprar' }));
    expect(await screen.findByRole('region', { name: 'Comprando' })).toBeTruthy();
    expect(screen.getByRole('region', { name: 'Alugando' })).toBeTruthy();
    expect(statusMatching(/deixa .* a mais de patrimônio no fim/)).toBeTruthy();
  });
});
