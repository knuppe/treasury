import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { mockCall, mockCallLog } from '@/api/mock';
import type { AssetLabel } from '@/api/generated/AssetLabel';
import type { ConcentrationReport } from '@/api/generated/ConcentrationReport';
import { formatMoney } from '@/lib/format';
import { renderApp } from '@/test/renderApp';

import { PORTFOLIO_UI_DEFAULTS, usePortfolioUi } from './state';
import { fixToday, quietCanvas } from './testing';

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
  usePortfolioUi.setState({ ...PORTFOLIO_UI_DEFAULTS });
});
afterEach(() => {
  vi.useRealTimers();
});

/** Abre as posições e aplica `prepare` no mock (depois do reset do `renderApp`). */
async function openPositions(prepare?: () => Promise<unknown>) {
  const app = renderApp('unlocked', {}, '#/carteira');
  if (prepare) {
    await prepare();
    await app.queryClient.invalidateQueries();
  }
  return app;
}

const buy = (asset_id: string, date: string) =>
  mockCall('portfolio.create_trade', {
    asset_id,
    account_id: 'acc-xp',
    date,
    side: 'buy',
    quantity: '10',
    price: '50',
  });

describe('08 · Carteira: posição recente', () => {
  it('menos de 30 dias: "—" com a etiqueta de posição recente e o retorno do período', async () => {
    await openPositions(() => buy('ast-vale3', '2026-10-09'));
    await waitFor(() => {
      const r = screen.getAllByRole('row').find((x) => within(x).queryByText('VALE3'));
      expect(r && within(r).queryByText('posição recente')).toBeTruthy();
    });
    const r = screen.getAllByRole('row').find((x) => within(x).queryByText('VALE3'))!;
    expect(within(r).getByText('—', { selector: 'span.num' })).toBeTruthy();
    expect(within(r).getByText('no período')).toBeTruthy();
    // A dica diz há quantos dias e quando vem o anualizado.
    expect(
      within(r).getByText(/Comprada há 5 dias\. Retorno no período: .+%\. O anualizado aparece/),
    ).toBeTruthy();
  });

  it('entre 30 dias e um ano: o retorno do período e o anualizado em texto discreto', async () => {
    await openPositions(() => buy('ast-taee11', '2026-06-16'));
    await waitFor(() => {
      const r = screen.getAllByRole('row').find((x) => within(x).queryByText('TAEE11'));
      expect(r && within(r).queryByText(/anualizado sobre 120 dias/)).toBeTruthy();
    });
    const r = screen.getAllByRole('row').find((x) => within(x).queryByText('TAEE11'))!;
    expect(within(r).queryByText('posição recente')).toBeNull();
  });
});

describe('08 · Carteira: concentração', () => {
  it('o card mostra o alerta crítico com o valor acima do limite', async () => {
    let report: ConcentrationReport | undefined;
    await openPositions(async () => {
      report = (await mockCall('portfolio.set_concentration_limit', {
        scope: 'asset',
        asset_id: 'ast-aapl',
        pct: 5,
      })) as ConcentrationReport;
    });
    const alert = report!.alerts.find((a) => a.ref === 'ast-aapl')!;
    expect(alert.severity).toBe('critical');
    const list = await screen.findByRole('list', { name: 'Alertas de concentração' });
    const item = within(list)
      .getAllByRole('listitem')
      .find((li) => within(li).queryByText('AAPL'))!;
    expect(within(item).getByText('Bem acima do limite')).toBeTruthy();
    expect(within(item).getByText(/da carteira · limite 5,0%/)).toBeTruthy();
    expect(item.textContent).toContain(`${formatMoney(alert.excess_base)} acima do limite`);
    // O grupo de exemplo (Exterior) também passa do padrão de 30%.
    expect(within(list).getByText('Exterior')).toBeTruthy();
  });

  it('o limite de um ativo no diálogo grava com portfolio.set_concentration_limit', async () => {
    const user = userEvent.setup();
    await openPositions();
    await user.click(await screen.findByRole('button', { name: 'Limites de concentração' }));
    const dialog = await screen.findByRole('dialog', { name: 'Limites de concentração' });
    const input = await within(dialog).findByLabelText('Limite de MSFT (%)');
    await user.type(input, '8');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar limites' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(mockCallLog()).toContain('portfolio.set_concentration_limit');
    const report = (await mockCall('portfolio.concentration', {})) as ConcentrationReport;
    const msft = report.assets.find((a) => a.ref === 'ast-msft')!;
    expect(msft.custom_limit).toBe(true);
    expect(msft.limit).toBeCloseTo(0.08);
  });

  it('marcar uma etiqueta como grupo grava com assets.label_update', async () => {
    const user = userEvent.setup();
    await openPositions();
    await user.click(await screen.findByRole('button', { name: 'Limites de concentração' }));
    const dialog = await screen.findByRole('dialog', { name: 'Limites de concentração' });
    await user.click(
      await within(dialog).findByRole('checkbox', { name: 'Bancos é grupo de concentração' }),
    );
    await user.type(within(dialog).getByLabelText('Limite do grupo Bancos (%)'), '5');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar limites' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(mockCallLog()).toContain('assets.label_update');
    expect(mockCallLog()).not.toContain('assets.labels_set');
    const labels = (await mockCall('assets.labels_list', {})) as AssetLabel[];
    const bancos = labels.find((l) => l.name === 'Bancos')!;
    expect(bancos.is_concentration_group).toBe(true);
    expect(bancos.concentration_limit_pct).toBe(5);
    // ITUB4 + BBAS3 passam de 5%: o grupo entra no card.
    const list = await screen.findByRole('list', { name: 'Alertas de concentração' });
    expect(await within(list).findByText('Bancos')).toBeTruthy();
  });

  it('as etiquetas de um ativo gravam com assets.labels_set', async () => {
    const user = userEvent.setup();
    await openPositions();
    await user.click(await screen.findByRole('button', { name: 'Limites de concentração' }));
    const dialog = await screen.findByRole('dialog', { name: 'Limites de concentração' });
    await user.type(await within(dialog).findByLabelText('Etiquetas de PETR4'), 'Energia{Enter}');
    await user.click(within(dialog).getByRole('button', { name: 'Salvar limites' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    const labels = (await mockCall('assets.labels_list', {})) as AssetLabel[];
    expect(labels.find((l) => l.name === 'Energia')?.asset_ids).toEqual(['ast-petr4']);
  });
});
