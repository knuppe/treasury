/**
 * Impostos in English: the screen, the scenarios, consumption by category and the sources dialog
 * come from the `impostos` catalog in the app's language (docs/i18n-design.md).
 */
import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { applyLocale } from '@/i18n';
import { renderApp } from '@/test/renderApp';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

afterEach(async () => {
  await applyLocale('pt-BR');
});

describe('Impostos in other languages', () => {
  it('shows the estimate, the scenarios and the sources in English', async () => {
    const user = userEvent.setup();
    await applyLocale('en');
    renderApp('unlocked', {}, '#/impostos');
    expect(await screen.findByRole('heading', { name: /\d{4} estimate/ })).toBeTruthy();
    const bar = screen.getByRole('region', { name: 'Estimate scenario' });
    expect(within(bar).getByRole('radio', { name: 'Likely' }).getAttribute('aria-checked')).toBe(
      'true',
    );
    expect(within(bar).getByText('Estimated total')).toBeTruthy();
    const scenarios = screen.getByRole('table', { name: 'Three scenarios' });
    expect(within(scenarios).getByRole('columnheader', { name: 'Conservative' })).toBeTruthy();
    expect(screen.getByRole('grid', { name: 'Spending by category' })).toBeTruthy();

    await user.click(within(bar).getByRole('button', { name: 'See sources and methodology' }));
    const dialog = await screen.findByRole('dialog', { name: 'Sources and methodology' });
    expect(within(dialog).getByRole('heading', { name: 'Methodology' })).toBeTruthy();
  });
});
