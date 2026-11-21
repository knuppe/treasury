/**
 * Impostos contra o mock: total e partes, troca de cenário, fontes e metodologia,
 * grupo e alíquota própria de uma categoria, informe de rendimentos e troca de país.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { TaxDataset } from '@/api/generated/TaxDataset';
import type { TaxEstimate } from '@/api/generated/TaxEstimate';
import { mockCall, mockCallLog } from '@/api/mock';
import { formatMoney } from '@/lib/format';
import { chartOption } from '@/test/echartsMock';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

const estimate = () => mockCall('tax_burden.estimate', {}) as Promise<TaxEstimate>;

async function open(hash = '#/impostos') {
  renderApp('unlocked', {}, hash);
  await screen.findByRole('heading', { name: /Estimativa de \d{4}/ });
  return screen.getByRole('region', { name: 'Cenário da estimativa' });
}

const row = (grid: HTMLElement, text: string) => within(grid).getByText(text).closest('tr')!;

describe('Impostos', () => {
  it('mostra o total do cenário provável, as partes e os três cenários', async () => {
    const bar = await open();
    const e = await estimate();
    expect(within(bar).getByText(formatMoney(e.totals.probable))).toBeTruthy();
    expect(within(bar).getByRole('radio', { name: 'Provável' }).getAttribute('aria-checked')).toBe(
      'true',
    );
    // Gráfico por tipo, no cenário provável, na ordem do núcleo.
    const option = chartOption(/Imposto estimado por tipo, cenário Provável/) as {
      series: { data: number[] }[];
    };
    expect([...option.series[0]!.data].reverse()).toEqual(e.parts.map((p) => p.amount.probable));
    const scenarios = screen.getByRole('table', { name: 'Três cenários' });
    for (const s of ['Conservador', 'Provável', 'Elevado'])
      expect(within(scenarios).getByRole('columnheader', { name: s })).toBeTruthy();
    expect(within(scenarios).getByText(formatMoney(e.totals.high))).toBeTruthy();
    // Aviso obrigatório sempre à vista.
    const ds = (await mockCall('tax_burden.dataset', { country: 'BR' })) as TaxDataset;
    expect(screen.getByText(ds.disclaimer)).toBeTruthy();
    expect(mockCallLog()).toContain('tax_burden.estimate');
  });

  it('o seletor de cenário troca o total na tela e fica na URL', async () => {
    const user = userEvent.setup();
    const bar = await open();
    const e = await estimate();
    await user.click(within(bar).getByRole('radio', { name: 'Elevado' }));
    expect(await within(bar).findByText(formatMoney(e.totals.high))).toBeTruthy();
    expect(window.location.hash).toContain('cenario=elevado');
    const option = chartOption(/cenário Elevado/) as { series: { data: number[] }[] };
    expect([...option.series[0]!.data].reverse()).toEqual(e.parts.map((p) => p.amount.high));
    await user.click(within(bar).getByRole('radio', { name: 'Conservador' }));
    expect(await within(bar).findByText(formatMoney(e.totals.conservative))).toBeTruthy();
  });

  it('abre a URL no cenário pedido', async () => {
    const bar = await open('#/impostos?cenario=conservador');
    const e = await estimate();
    expect(within(bar).getByText(formatMoney(e.totals.conservative))).toBeTruthy();
  });

  it('"Ver fontes e metodologia" lista a versão, a metodologia e cada fonte', async () => {
    const user = userEvent.setup();
    const bar = await open();
    const ds = (await mockCall('tax_burden.dataset', { country: 'BR' })) as TaxDataset;
    await user.click(within(bar).getByRole('button', { name: 'Ver fontes e metodologia' }));
    const dialog = await screen.findByRole('dialog', { name: 'Fontes e metodologia' });
    expect(within(dialog).getAllByText(ds.version).length).toBeGreaterThan(0);
    expect(within(dialog).getByText(ds.methodology[0]!)).toBeTruthy();
    const sources = within(dialog).getByRole('list', { name: 'Fontes' });
    expect(within(sources).getAllByRole('link')).toHaveLength(ds.sources.length);
    const first = within(sources).getAllByRole('link', { name: ds.sources[0]!.title })[0]!;
    expect(first.getAttribute('href')).toBe(ds.sources[0]!.url);
    expect(within(dialog).getByText(ds.disclaimer)).toBeTruthy();
  });

  it('trocar o grupo tira a categoria do consumo; alíquota própria a põe de volta', async () => {
    const user = userEvent.setup();
    await open();
    const grid = screen.getByRole('grid', { name: 'Consumo por categoria' });
    expect(within(grid).getByText('Alimentação › Mercado')).toBeTruthy();

    await user.click(screen.getByRole('button', { name: 'Trocar grupo de Alimentação › Mercado' }));
    const dialog = await screen.findByRole('dialog', { name: 'Trocar grupo de imposto' });
    await user.click(within(dialog).getByRole('combobox', { name: 'Grupo' }));
    await user.click(await screen.findByRole('option', { name: /^Financeiro/ }));
    await user.click(within(dialog).getByRole('button', { name: 'Salvar' }));
    await waitFor(() => expect(within(grid).queryByText('Alimentação › Mercado')).toBeNull());
    const outside = screen.getByRole('region', { name: 'Fora do consumo' });
    expect(within(outside).getByText('Alimentação › Mercado')).toBeTruthy();

    // Exames de saúde não têm dado oficial: com alíquota própria, entram na estimativa.
    const noRate = screen.getByRole('region', { name: 'Sem dado confiável' });
    await user.click(
      within(noRate).getByRole('button', { name: 'Alíquota própria de Saúde › Exames' }),
    );
    const rateDialog = await screen.findByRole('dialog', { name: 'Alíquota própria' });
    await user.type(within(rateDialog).getByLabelText('Alíquota (% do preço)'), '10');
    await user.click(within(rateDialog).getByRole('button', { name: 'Salvar' }));
    const line = await waitFor(() => row(grid, 'Saúde › Exames'));
    const e = await estimate();
    const exames = e.consumption.find((c) => c.path === 'Saúde › Exames')!;
    expect(exames.own_rate).toBe(10);
    expect(within(line).getByText(formatMoney(Math.round(exames.spent / 10)))).toBeTruthy();
    expect(
      within(line).getByRole('button', { name: /Alíquota de Saúde › Exames/ }).textContent,
    ).toContain('10%');

    // Voltar ao padrão pelo menu da linha.
    await user.pointer({ keys: '[MouseRight]', target: line });
    await user.click(await screen.findByRole('menuitem', { name: 'Voltar ao padrão' }));
    await waitFor(() => expect(within(grid).queryByText('Saúde › Exames')).toBeNull());
  });

  it('"Grupos das categorias" (?grupos=1) troca o grupo e a alíquota de qualquer categoria', async () => {
    const user = userEvent.setup();
    // Com o diálogo aberto, o resto da tela fica fora da árvore acessível.
    renderApp('unlocked', {}, '#/impostos?grupos=1');
    const dialog = await screen.findByRole('dialog', { name: 'Grupos das categorias' });
    await user.click(
      await within(dialog).findByRole('combobox', { name: 'Grupo de Alimentação › Mercado' }),
    );
    await user.click(await screen.findByRole('option', { name: /^Restaurantes/ }));
    await waitFor(() =>
      expect(within(dialog).getByText(/^Restaurantes.*sua escolha$/)).toBeTruthy(),
    );
    const rate = within(dialog).getByRole('textbox', {
      name: 'Alíquota própria de Saúde › Farmácia',
    });
    await user.type(rate, '12,5{Enter}');
    await waitFor(async () => {
      const e = await estimate();
      expect(e.consumption.find((c) => c.path === 'Saúde › Farmácia')?.own_rate).toBe(12.5);
    });
    await user.keyboard('{Escape}');
    expect(window.location.hash).not.toContain('grupos=1');
    const grid = screen.getByRole('grid', { name: 'Consumo por categoria' });
    expect(within(row(grid, 'Alimentação › Mercado')).getByText('sua escolha')).toBeTruthy();
  }, 60_000); // diálogo com todas as categorias: sob a carga da build, passa dos 20 s padrão

  it('o informe de rendimentos troca a renda do titular para "informado"', async () => {
    const user = userEvent.setup();
    await open();
    const payroll = screen.getByRole('grid', { name: 'Renda do trabalho' });
    expect(within(row(payroll, 'Ana')).getByText('≈ estimado')).toBeTruthy();

    await user.click(screen.getByRole('button', { name: 'Lançar informe' }));
    const dialog = await screen.findByRole('dialog', { name: /Lançar informe de rendimentos/ });
    await user.type(within(dialog).getByLabelText('Fonte pagadora'), 'Alfa Engenharia');
    await user.click(within(dialog).getByRole('combobox', { name: 'Membro' }));
    await user.click(await screen.findByRole('option', { name: 'Ana' }));
    await user.type(within(dialog).getByLabelText('Rendimentos tributáveis'), '200.000,00');
    await user.type(within(dialog).getByLabelText('Previdência oficial (INSS)'), '9.000,00');
    await user.type(within(dialog).getByLabelText('IR retido na fonte'), '40.000,00');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar' }));

    await waitFor(() => expect(within(row(payroll, 'Ana')).getByText('informado')).toBeTruthy());
    expect(within(row(payroll, 'Ana')).getByText('R$ 40.000,00')).toBeTruthy();
    expect(within(row(payroll, 'Bruno')).getByText('≈ estimado')).toBeTruthy();
    const reports = screen.getByRole('grid', { name: /Informes de rendimentos de/ });
    expect(within(reports).getByText('Alfa Engenharia')).toBeTruthy();
  });

  it('trocar o país para os EUA mostra o esboço e o estado americano', async () => {
    const user = userEvent.setup();
    await open();
    await user.click(screen.getByRole('combobox', { name: 'País dos dados' }));
    await user.click(await screen.findByRole('option', { name: /Estados Unidos/ }));
    expect(await screen.findByText('Estados Unidos: esboço')).toBeTruthy();
    expect(mockCallLog()).toContain('tax_burden.set_settings');
    expect(screen.getByRole('combobox', { name: 'Estado' })).toBeTruthy();
    const e = await estimate();
    expect(e.dataset.country).toBe('US');
    expect(e.dataset.coverage).not.toBe('completo');
  });
  it('"Para onde vai o imposto" reparte o total pelas funções do governo', async () => {
    await open();
    const e = await estimate();
    const card = (await screen.findByRole('heading', { name: 'Para onde vai o imposto' })).closest(
      'section',
    )!;
    const d = e.destination!;
    expect(d.lines).toHaveLength(10);
    expect(d.lines.reduce((a, l) => a + l.amounts.probable, 0)).toBe(e.totals.probable);
    const social = d.lines.find((l) => l.key === 'protecao_social')!;
    const line = within(card).getByText(social.label).closest('li')!;
    expect(within(line).getByText(formatMoney(social.amounts.probable))).toBeTruthy();
    expect(within(card).getByRole('link', { name: /Tesouro Nacional/ })).toBeTruthy();
  });

  it('a arrecadação da cidade pede a cidade, busca e mostra o valor por habitante', async () => {
    const user = userEvent.setup();
    await open();
    const card = (
      await screen.findByRole('heading', { name: 'Arrecadação da sua cidade' })
    ).closest('section')!;
    await user.click(within(card).getByRole('combobox', { name: 'Estado' }));
    await user.click(await screen.findByRole('option', { name: /São Paulo/ }));
    await user.click(within(card).getByRole('combobox', { name: 'Cidade' }));
    await user.click(await screen.findByRole('option', { name: 'Vila Exemplo' }));
    await user.click(within(card).getByRole('button', { name: 'Salvar' }));
    const titled = (
      await screen.findByRole('heading', { name: 'Arrecadação de Vila Exemplo (SP)' })
    ).closest('section')!;
    expect(within(titled).getByText('Nenhuma consulta feita ainda para esta cidade.')).toBeTruthy();
    await user.click(within(titled).getByRole('button', { name: 'Buscar no Tesouro Nacional' }));
    // R$ 30 milhões ÷ 20 mil habitantes.
    expect(await within(titled).findByText(formatMoney(150_000))).toBeTruthy();
    expect(within(titled).getByText('20.000 habitantes')).toBeTruthy();
    expect(within(titled).getByText('ISS')).toBeTruthy();
    expect(mockCallLog()).toEqual(
      expect.arrayContaining([
        'tax_burden.cities',
        'tax_burden.set_city',
        'tax_burden.refresh_city',
      ]),
    );
  });
});
