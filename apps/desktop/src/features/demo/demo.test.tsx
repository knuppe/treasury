import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { mockCall } from '@/api/mock';
import type { VaultList } from '@/api/generated/VaultList';
import { useUi } from '@/state/ui';
import { renderApp } from '@/test/renderApp';

const next = () => screen.getByRole('button', { name: 'Continuar' });

describe('demo vault', () => {
  it('the wizard creates a separate vault, marked as a demo', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=cofres');
    await screen.findByRole('list', { name: 'Cofres' });
    await user.click(screen.getByRole('button', { name: 'Cofre de demonstração' }));
    const quick = await screen.findByRole('dialog', { name: 'Cofre de demonstração' });
    // Quick creation: the five profiles; "Personalizar" opens the full wizard.
    expect(within(quick).getAllByRole('radio')).toHaveLength(5);
    await user.click(within(quick).getByRole('button', { name: 'Personalizar antes de criar' }));
    // The wizard opens on top (the window below is hidden from screen readers).
    await screen.findByText(/Passo 1 de 6/);
    const dialog = screen.getByRole('dialog', { name: 'Cofre de demonstração' });

    // Profile: the five ready-made ones and the build-from-scratch one.
    const profiles = within(dialog).getByRole('radiogroup', { name: 'Perfil da família' });
    expect(within(profiles).getAllByRole('radio')).toHaveLength(6);
    await user.click(within(profiles).getByRole('radio', { name: /Montar do zero/ }));
    await user.click(next());

    // Family: one adult and two children.
    await user.click(within(dialog).getByRole('radio', { name: 'Um' }));
    await user.click(within(dialog).getByRole('radio', { name: '2' }));
    expect(within(dialog).getByLabelText<HTMLInputElement>('Nome do filho 2').value).toBe('Davi');
    // Age and sex of each person: a blank age is left to the profile.
    expect(within(dialog).getByLabelText<HTMLInputElement>('Idade de Ana').value).toBe('41');
    expect(within(dialog).getByLabelText<HTMLInputElement>('Idade de Davi').value).toBe('');
    expect(within(dialog).getByRole('combobox', { name: 'Sexo de Davi' }).textContent).toBe(
      'Homem',
    );
    await user.click(next());

    // Income: the total appears and the following values are suggested from it.
    expect(within(dialog).getByText(/Renda total por mês/)).toBeTruthy();
    await user.click(next());
    await user.click(within(dialog).getByRole('radio', { name: 'Financiamento' }));
    expect(within(dialog).getByLabelText('Parcela do financiamento')).toBeTruthy();
    expect(within(dialog).getByLabelText('Escola, por filho')).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Sugerir pela renda' }));
    await user.click(next());
    expect(within(dialog).getByLabelText('Limite do cartão')).toBeTruthy();
    await user.click(next());

    // Last step: no password, with the notice that it is only for fictitious data.
    expect(within(dialog).queryByLabelText('Senha do cofre de demonstração')).toBeNull();
    expect(within(dialog).getByText(/abre sem senha/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Criar cofre de demonstração' }));

    // The new vault opens, with the demo badge; the main one stays in the list.
    await waitFor(() =>
      expect(
        within(document.querySelector<HTMLElement>('header.topbar')!).getByText('Demonstração'),
      ).toBeTruthy(),
    );
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Cofre de demonstração' })).toBeNull(),
    );
    const list = (await mockCall('vaults.list', {})) as VaultList;
    // It came from the profile selected in quick creation (middle class) and was built by hand.
    expect(list.vaults.map((v) => v.name)).toEqual([
      'Principal',
      'Demonstração · Classe média com folga',
    ]);
    expect(list.vaults.find((v) => v.active)?.purpose).toMatch(/Demonstração/);
    expect(list.vaults.find((v) => v.active)?.demo).toBe(true);
  });

  it('quick creation: struggling family, and the demo opens without a password', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=cofres');
    // Privacy on in the previous vault: the demo opens with it off.
    useUi.getState().setPrivacyOnUnlock(true);
    useUi.getState().setPrivacy(true);
    await screen.findByRole('list', { name: 'Cofres' });
    await user.click(screen.getByRole('button', { name: 'Cofre de demonstração' }));
    const dialog = await screen.findByRole('dialog', { name: 'Cofre de demonstração' });
    await user.click(within(dialog).getByRole('radio', { name: /Família em dificuldade/ }));
    await user.click(within(dialog).getByRole('button', { name: 'Criar e abrir' }));
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Cofre de demonstração' })).toBeNull(),
    );
    const list = (await mockCall('vaults.list', {})) as VaultList;
    expect(list.vaults.find((v) => v.active)?.name).toBe('Demonstração · Família em dificuldade');
    await waitFor(() => expect(useUi.getState().privacy).toBe(false));
    // Locked, it opens with a button, no password.
    await user.click(screen.getAllByRole('button', { name: /Bloquear cofre/ })[0]!);
    const open = await screen.findByRole('button', { name: 'Abrir a demonstração' });
    expect(screen.getByText(/nunca com dados reais/)).toBeTruthy();
    expect(screen.queryByLabelText('Senha mestra')).toBeNull();
    useUi.getState().setPrivacy(true);
    await user.click(open);
    await waitFor(() =>
      expect(
        within(document.querySelector<HTMLElement>('header.topbar')!).getByText('Demonstração'),
      ).toBeTruthy(),
    );
    // When opened again, privacy stays off in the demo.
    expect(useUi.getState().privacy).toBe(false);
  });

  it('first time: picks the demo and the vault is born ready, without a password', async () => {
    const user = userEvent.setup();
    renderApp('empty');
    await user.click(await screen.findByRole('radio', { name: /Demonstração/ }));
    await user.click(await screen.findByRole('radio', { name: /Jovem morando só/ }));
    expect(screen.queryByLabelText('Senha mestra')).toBeNull();
    await user.click(screen.getByRole('button', { name: 'Criar e abrir' }));
    await waitFor(() =>
      expect(
        within(document.querySelector<HTMLElement>('header.topbar')!).getByText('Demonstração'),
      ).toBeTruthy(),
    );
    // No empty vault left over in the list: the demo took the first-time vault.
    const list = (await mockCall('vaults.list', {})) as VaultList;
    expect(list.vaults).toHaveLength(1);
    expect(list.vaults[0]!.demo).toBe(true);
  });

  it('the welcome offers the demo vault', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/boas-vindas?refazer=1');
    await user.click(await screen.findByRole('button', { name: 'Criar um cofre de demonstração' }));
    expect(await screen.findByRole('dialog', { name: 'Cofre de demonstração' })).toBeTruthy();
  });
});

describe("profile values in the country's currency", () => {
  it('a currency without cents gets the value in its own unit (yen, Chilean peso)', async () => {
    const { fromPreset, initialDraft } = await import('./DemoWizard');
    const base = initialDraft('avancado');
    // R$ 12,000.00 of income: 1,200,000 cents.
    expect(fromPreset('comfortable', 'BR', 'BRL', base).adults[0]!.income).toBe(1_200_000);
    // × 0.45 per real, in US cents: US$ 5,400.00.
    expect(fromPreset('comfortable', 'US', 'USD', base).adults[0]!.income).toBe(540_000);
    // × 300 per real, in pesos (no cents): CLP 3,600,000, not 360 million.
    expect(fromPreset('comfortable', 'CL', 'CLP', base).adults[0]!.income).toBe(3_600_000);
    // × 55 per real, in yen: ¥ 660,000.
    expect(fromPreset('comfortable', 'JP', 'JPY', base).adults[0]!.income).toBe(660_000);
  });
});

describe('people in the sample family', () => {
  it('each profile sends age and sex consistent with the scenario', async () => {
    const { fromPreset, initialDraft, toProfile } = await import('./DemoWizard');
    const base = initialDraft('avancado');
    const retiree = toProfile(fromPreset('retiree', 'BR', 'BRL', base));
    expect(retiree.adults).toEqual([
      expect.objectContaining({ name: 'Helena', retired: true, age: 68, sex: 'female' }),
    ]);
    const family = toProfile(fromPreset('low_income', 'BR', 'BRL', base));
    expect(family.adults.map((a) => [a.name, a.age, a.sex])).toEqual([
      ['Maria', 34, 'female'],
      ['João', 36, 'male'],
    ]);
    expect(family.children).toEqual(['Davi', 'Elisa']);
    expect(family.child_details).toEqual([
      { age: 11, sex: 'male' },
      { age: 7, sex: 'female' },
    ]);
    // Blank age: the core chooses from the profile.
    const blank = toProfile({
      ...base,
      adults: base.adults.map((a) => ({ ...a, age: '' })),
    });
    expect(blank.adults.every((a) => a.age === undefined)).toBe(true);
  });
});
