/**
 * 10 · Market against the mock: overview by sector, map, the sector cut in the explorer, the
 * old Assets links and the Watch action for a security not in the registry.
 */
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeAll, describe, expect, it } from 'vitest';

import { mockCall } from '@/api/mock';
import type { AssetTag } from '@/api/generated/AssetTag';
import type { WatchlistItem } from '@/api/generated/WatchlistItem';
import { quietCanvas } from '@/features/portfolio/testing';
import { renderApp } from '@/test/renderApp';

beforeAll(quietCanvas);

describe('10 · Market', () => {
  it('overview by sector, with the notice; the sector opens the explorer with the cut', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/mercado');
    expect(await screen.findByRole('heading', { level: 1, name: /Mercado/ })).toBeTruthy();
    expect(await screen.findByText('O mercado hoje')).toBeTruthy();
    expect(
      screen.getAllByRole('note').some((n) => /não é recomendação/i.test(n.textContent ?? '')),
    ).toBe(true);
    const grid = await screen.findByRole('grid', { name: 'Ações por setor' });
    const first = grid.querySelector('tbody tr[data-row-id]')!;
    const sector = first.getAttribute('data-row-id')!;
    // One click on the sector name is enough (double click on the row also opens it, below).
    const open = within(first as HTMLElement).getByRole('button', { name: sector });
    // A long name that wraps onto two lines stays left-aligned, like the short ones (the button centers).
    expect(open.classList.contains('text-left')).toBe(true);
    await user.click(open);
    await waitFor(() => expect(window.location.hash).toContain('aba=explorador'));
    expect(decodeURIComponent(window.location.hash)).toContain(`setor=${sector}`);
    // The cut already runs: sector stocks, only robust and healthy ones, highest score first.
    const result = await screen.findByRole('grid', { name: 'Resultado do screener' });
    const scores = Array.from(result.querySelectorAll('tbody tr[data-row-id]'));
    expect(scores.length).toBeGreaterThan(0);
    const filters = (await mockCall('research.screen', {
      filters: {
        conditions: [{ field: 'score_band', op: 'in', value: ['robusto', 'saudavel'] }],
        universe: { classes: ['stock_br'], sectors: [sector] },
      },
    })) as { total: number };
    expect(scores.length).toBe(filters.total);
  });

  it('traits: selecting a tag redoes the cut asking for the tag; the rule stays in the title', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/mercado');
    const grid = await screen.findByRole('grid', { name: 'Ações por setor' });
    const first = grid.querySelector('tbody tr[data-row-id]')!;
    const sector = first.getAttribute('data-row-id')!;
    await user.dblClick(first.querySelector('td')!);
    await screen.findByRole('grid', { name: 'Resultado do screener' });
    const tags = (await mockCall('research.tags', {})) as AssetTag[];
    const tag = tags.find((x) => x.key === 'proventos_5_anos')!;
    const chip = await screen.findByRole('button', { name: new RegExp(`^${tag.label} ·`) });
    expect(chip.getAttribute('title')).toBe(tag.rule);
    expect(chip.getAttribute('aria-pressed')).toBe('false');
    await user.click(chip);
    expect(chip.getAttribute('aria-pressed')).toBe('true');
    const expected = (await mockCall('research.screen', {
      filters: {
        conditions: [{ field: 'score_band', op: 'in', value: ['robusto', 'saudavel'] }],
        universe: { classes: ['stock_br'], sectors: [sector] },
        tags: [tag.key],
      },
      limit: 500,
    })) as { total: number; items: { tags: string[] }[]; tag_counts: Record<string, number> };
    for (const i of expected.items) expect(i.tags).toContain(tag.key);
    await waitFor(() => {
      const result = screen.getByRole('grid', { name: 'Resultado do screener' });
      expect(result.querySelectorAll('tbody tr[data-row-id]').length).toBe(expected.total);
    });
    // Faceted counts: the selected one gives the cut total; the others, how many of it have the tag.
    expect(chip.textContent).toBe(`${tag.label} · ${expected.total}`);
    const counts = expected.tag_counts;
    const other = tags.find((x) => x.key === 'proventos_10_anos')!;
    expect(screen.getByRole('button', { name: new RegExp(`^${other.label} ·`) }).textContent).toBe(
      `${other.label} · ${counts[other.key] ?? 0}`,
    );
    // Only the tags that apply to stocks (nothing about FII unit holders).
    expect(screen.queryByRole('button', { name: /^Muitos cotistas ·/ })).toBeNull();
  });

  it('FII segment opens only the healthy FIIs of the segment, without P/L', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/mercado');
    await user.click(await screen.findByRole('radio', { name: 'FIIs por segmento' }));
    const grid = await screen.findByRole('grid', { name: 'FIIs por segmento' });
    const first = grid.querySelector('tbody tr[data-row-id]')!;
    const segment = first.getAttribute('data-row-id')!;
    await user.dblClick(first.querySelector('td')!);
    await waitFor(() =>
      expect(decodeURIComponent(window.location.hash)).toContain(`segmento=${segment}`),
    );
    const result = await screen.findByRole('grid', { name: 'Resultado do screener' });
    expect(within(result).queryByRole('columnheader', { name: /P\/L/ })).toBeNull();
    const r = (await mockCall('research.screen', {
      filters: {
        conditions: [{ field: 'score_band', op: 'in', value: ['robusto', 'saudavel'] }],
        universe: { classes: ['fii'], segments: [segment] },
      },
    })) as { total: number };
    expect(result.querySelectorAll('tbody tr[data-row-id]').length).toBe(r.total);
  });

  it('Health × Attractiveness map and the redirect of old links', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ativos?aba=screener');
    await waitFor(() => expect(window.location.hash).toContain('/mercado?aba=explorador'));
    await user.click(await screen.findByRole('tab', { name: 'Mapa' }));
    expect(await screen.findByText('Saúde × Atratividade')).toBeTruthy();
    expect(screen.getByRole('img', { name: /Mapa de \d+ papéis/ })).toBeTruthy();
  });

  it('methods: cards, the recipe with what was adapted, the list and the why of a security', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/mercado?aba=metodos');
    expect(await screen.findByText('Graham defensivo')).toBeTruthy();
    expect(await screen.findAllByText(/cumprem os critérios hoje/)).not.toHaveLength(0);
    const buttons = screen.getAllByRole('button', { name: 'Ver a receita e a lista' });
    await user.click(buttons[0]!);
    expect(await screen.findByRole('list', { name: 'Critérios' })).toBeTruthy();
    expect(
      await screen.findByText(/papéis avaliados cumprem todos os critérios hoje/),
    ).toBeTruthy();
    expect(screen.getByRole('grid', { name: 'Quem cumpre Graham defensivo' })).toBeTruthy();
    await user.type(screen.getByRole('textbox', { name: /Por que um papel/ }), 'petr4');
    await user.click(screen.getByRole('button', { name: 'Explicar' }));
    expect(await screen.findByText(/PETR4 (cumpre|não cumpre|fica de fora)/)).toBeTruthy();
    await user.click(screen.getByRole('button', { name: 'Todos os métodos' }));
    expect(await screen.findAllByRole('button', { name: 'Ver a receita e a lista' })).toHaveLength(
      buttons.length,
    );
  });

  it('security outside the registry: Watch adds it to the watchlist; unwatching removes it from the registry', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ativos?aba=indicadores&ticker=ABEV3');
    await user.click(await screen.findByRole('button', { name: 'Observar' }));
    await waitFor(async () => {
      const w = (await mockCall('portfolio.list_watchlist', {})) as WatchlistItem[];
      expect(w.some((x) => x.asset.ticker === 'ABEV3')).toBe(true);
    });
    await user.click(await screen.findByRole('button', { name: 'Deixar de observar' }));
    await waitFor(async () => {
      const w = (await mockCall('portfolio.list_watchlist', {})) as WatchlistItem[];
      expect(w.some((x) => x.asset.ticker === 'ABEV3')).toBe(false);
    });
    expect(await screen.findByRole('button', { name: 'Observar' })).toBeTruthy();
    expect(within(document.body).queryByText(/Observado pelo Mercado/)).toBeNull();
  });
});
