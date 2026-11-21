/**
 * Financial health against the mock: dashboard card, reports section and the model in
 * Settings (including turning it off).
 */
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { HealthReport } from '@/api/generated/HealthReport';
import { mockCall, mockCallLog } from '@/api/mock';
import { useUi } from '@/state/ui';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

/** The dashboard card (waits for the dashboard to open: the settings section has the same name). */
async function card() {
  await screen.findByRole('heading', { level: 2, name: 'Patrimônio líquido' });
  return screen.findByRole('region', { name: 'Saúde financeira' });
}

describe('financial health', () => {
  it('dashboard: score, band, national average and the four I-SFB dimensions', async () => {
    renderApp('unlocked', {}, '#/painel');
    const c = await card();
    const report = (await mockCall('health.score', {})) as HealthReport;
    expect(report.score).not.toBeNull();
    const score = report.score!.toFixed(1).replace('.', ',');
    expect(within(c).getByText(score)).toBeTruthy();
    expect(within(c).getByText(report.band!.label)).toBeTruthy();
    expect(within(c).getByText(/Média nacional/)).toBeTruthy();
    const dims = within(c).getByRole('list', { name: 'Dimensões' });
    for (const d of ['Comportamento', 'Habilidade', 'Liberdade', 'Segurança'])
      expect(within(dims).getByText(d)).toBeTruthy();
    expect(within(c).getByText(report.summary)).toBeTruthy();
    expect(mockCallLog()).toContain('health.score');
    const link = within(c).getByRole('link', { name: /Ver indicadores/ });
    expect(link.getAttribute('href')).toBe('#/relatorios?v=saude');
  });

  it('privacy mode hides the score and the values, but not the band', async () => {
    renderApp('unlocked', {}, '#/painel');
    const c = await card();
    const report = (await mockCall('health.score', {})) as HealthReport;
    act(() => {
      useUi.setState({ privacy: true });
    });
    const score = report.score!.toFixed(1).replace('.', ',');
    // Neither the gauge nor the summary ("Nota 84,7 …") shows the score.
    await waitFor(() => expect(within(c).queryByText(new RegExp(score))).toBeNull());
    expect(within(c).getAllByText('valor oculto').length).toBeGreaterThan(0);
    expect(within(c).getByText(report.band!.label)).toBeTruthy();
  });

  it('reports: each indicator with value × target, calculation and suggestions', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/relatorios?v=saude');
    expect(
      await screen.findByRole('tab', { name: /Saúde financeira/, selected: true }),
    ).toBeTruthy();
    const savings = await screen.findByRole('article', { name: 'Taxa de poupança' });
    expect(within(savings).getByText('≥ 20% da renda')).toBeTruthy();
    expect(within(savings).getByText(/receitas − despesas/)).toBeTruthy();
    // Indicators off target bring concrete suggestions.
    const report = (await mockCall('health.score', {})) as HealthReport;
    const weak = report.metrics.find((m) => m.status !== 'good' && m.status !== 'unknown')!;
    const art = screen.getByRole('article', { name: weak.label });
    expect(within(art).getByText('Sugestões')).toBeTruthy();
    expect(within(art).getByText(weak.suggestions[0]!)).toBeTruthy();
    for (const d of ['Comportamento', 'Habilidade', 'Liberdade', 'Segurança'])
      expect(screen.getByRole('region', { name: d })).toBeTruthy();
    expect(screen.getByRole('link', { name: /Sobre o I-SFB/ }).getAttribute('href')).toContain(
      'meubolsoemdia.com.br',
    );
    // Previous month: recomputes the score for that month.
    await user.click(screen.getByRole('button', { name: 'Mês anterior' }));
    await waitFor(() => expect(window.location.hash).toMatch(/mes=\d{4}-\d{2}/));
  });

  it('settings: switches to the classic indicators and then turns off (the card disappears)', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=saude');
    const group = await screen.findByRole('radiogroup', { name: 'Modelo de saúde financeira' });
    const isf = within(group).getByRole('radio', { name: /I-SFB/ });
    expect(isf.getAttribute('aria-checked')).toBe('true');
    await user.click(within(group).getByRole('radio', { name: /Indicadores clássicos/ }));
    await waitFor(async () =>
      expect(((await mockCall('health.get_settings', {})) as { model: string }).model).toBe(
        'ratios',
      ),
    );
    expect(await screen.findByText('Modelo: Indicadores clássicos')).toBeTruthy();

    window.location.hash = '#/painel';
    const c = await card();
    expect(await within(c).findByRole('list', { name: 'Indicadores' })).toBeTruthy();
    expect(within(c).getByText('Custo da moradia')).toBeTruthy();

    window.location.hash = '#/ajustes?secao=saude';
    const again = await screen.findByRole('radiogroup', { name: 'Modelo de saúde financeira' });
    await user.click(within(again).getByRole('radio', { name: /Desativado/ }));
    expect(await screen.findByText('Saúde financeira desativada')).toBeTruthy();

    window.location.hash = '#/painel';
    await screen.findByRole('heading', { level: 2, name: 'Patrimônio líquido' });
    await waitFor(() =>
      expect(screen.queryByRole('region', { name: 'Saúde financeira' })).toBeNull(),
    );

    window.location.hash = '#/relatorios?v=saude';
    expect(await screen.findByText('Análise de saúde financeira desligada')).toBeTruthy();
  });
});
