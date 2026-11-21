import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { mockCall } from '@/api/mock';
import type { VaultList } from '@/api/generated/VaultList';
import { useUi } from '@/state/ui';
import { renderApp } from '@/test/renderApp';

const next = () => screen.getByRole('button', { name: 'Continuar' });

describe('cofre de demonstração', () => {
  it('o assistente cria um cofre separado, marcado como demonstração', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=cofres');
    await screen.findByRole('list', { name: 'Cofres' });
    await user.click(screen.getByRole('button', { name: 'Cofre de demonstração' }));
    const quick = await screen.findByRole('dialog', { name: 'Cofre de demonstração' });
    // Criação rápida: os cinco perfis; "Personalizar" abre o assistente completo.
    expect(within(quick).getAllByRole('radio')).toHaveLength(5);
    await user.click(within(quick).getByRole('button', { name: 'Personalizar antes de criar' }));
    // O assistente abre por cima (a janela de baixo fica escondida dos leitores de tela).
    await screen.findByText(/Passo 1 de 6/);
    const dialog = screen.getByRole('dialog', { name: 'Cofre de demonstração' });

    // Perfil: os cinco prontos e o de montar do zero.
    const profiles = within(dialog).getByRole('radiogroup', { name: 'Perfil da família' });
    expect(within(profiles).getAllByRole('radio')).toHaveLength(6);
    await user.click(within(profiles).getByRole('radio', { name: /Montar do zero/ }));
    await user.click(next());

    // Família: um adulto e dois filhos.
    await user.click(within(dialog).getByRole('radio', { name: 'Um' }));
    await user.click(within(dialog).getByRole('radio', { name: '2' }));
    expect(within(dialog).getByLabelText<HTMLInputElement>('Nome do filho 2').value).toBe('Davi');
    // Idade e sexo de cada pessoa: a idade em branco fica pelo perfil.
    expect(within(dialog).getByLabelText<HTMLInputElement>('Idade de Ana').value).toBe('41');
    expect(within(dialog).getByLabelText<HTMLInputElement>('Idade de Davi').value).toBe('');
    expect(within(dialog).getByRole('combobox', { name: 'Sexo de Davi' }).textContent).toBe(
      'Homem',
    );
    await user.click(next());

    // Renda: o total aparece e os valores seguintes vêm sugeridos por ela.
    expect(within(dialog).getByText(/Renda total por mês/)).toBeTruthy();
    await user.click(next());
    await user.click(within(dialog).getByRole('radio', { name: 'Financiamento' }));
    expect(within(dialog).getByLabelText('Parcela do financiamento')).toBeTruthy();
    expect(within(dialog).getByLabelText('Escola, por filho')).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Sugerir pela renda' }));
    await user.click(next());
    expect(within(dialog).getByLabelText('Limite do cartão')).toBeTruthy();
    await user.click(next());

    // Último passo: sem senha, com o aviso de que é só para dados fictícios.
    expect(within(dialog).queryByLabelText('Senha do cofre de demonstração')).toBeNull();
    expect(within(dialog).getByText(/abre sem senha/)).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: 'Criar cofre de demonstração' }));

    // O cofre novo abre, com o selo de demonstração; o principal continua na lista.
    await waitFor(() =>
      expect(
        within(document.querySelector<HTMLElement>('header.topbar')!).getByText('Demonstração'),
      ).toBeTruthy(),
    );
    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Cofre de demonstração' })).toBeNull(),
    );
    const list = (await mockCall('vaults.list', {})) as VaultList;
    // Veio do perfil marcado na criação rápida (classe média) e foi montado à mão.
    expect(list.vaults.map((v) => v.name)).toEqual([
      'Principal',
      'Demonstração · Classe média com folga',
    ]);
    expect(list.vaults.find((v) => v.active)?.purpose).toMatch(/Demonstração/);
    expect(list.vaults.find((v) => v.active)?.demo).toBe(true);
  });

  it('criação rápida: família em dificuldade, e a demonstração abre sem senha', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ajustes?secao=cofres');
    // Privacidade ligada no cofre de antes: a demonstração abre com ela desligada.
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
    // Bloqueado, abre por um botão, sem senha.
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
    // Ao abrir de novo, a privacidade continua desligada na demonstração.
    expect(useUi.getState().privacy).toBe(false);
  });

  it('primeira vez: escolhe a demonstração e o cofre nasce pronto, sem senha', async () => {
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
    // Sem cofre vazio sobrando na lista: a demonstração ocupou o cofre da primeira vez.
    const list = (await mockCall('vaults.list', {})) as VaultList;
    expect(list.vaults).toHaveLength(1);
    expect(list.vaults[0]!.demo).toBe(true);
  });

  it('as boas-vindas oferecem o cofre de demonstração', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/boas-vindas?refazer=1');
    await user.click(await screen.findByRole('button', { name: 'Criar um cofre de demonstração' }));
    expect(await screen.findByRole('dialog', { name: 'Cofre de demonstração' })).toBeTruthy();
  });
});

describe('valores do perfil na moeda do país', () => {
  it('moeda sem centavos recebe o valor na própria unidade (iene, peso chileno)', async () => {
    const { fromPreset, initialDraft } = await import('./DemoWizard');
    const base = initialDraft('avancado');
    // R$ 12.000,00 de renda: 1.200.000 centavos.
    expect(fromPreset('comfortable', 'BR', 'BRL', base).adults[0]!.income).toBe(1_200_000);
    // × 0,45 por real, em centavos de dólar: US$ 5.400,00.
    expect(fromPreset('comfortable', 'US', 'USD', base).adults[0]!.income).toBe(540_000);
    // × 300 por real, em pesos (sem centavos): CLP 3.600.000, não 360 milhões.
    expect(fromPreset('comfortable', 'CL', 'CLP', base).adults[0]!.income).toBe(3_600_000);
    // × 55 por real, em ienes: ¥ 660.000.
    expect(fromPreset('comfortable', 'JP', 'JPY', base).adults[0]!.income).toBe(660_000);
  });
});

describe('pessoas da família de exemplo', () => {
  it('cada perfil manda idade e sexo coerentes com o cenário', async () => {
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
    // Idade em branco: o núcleo escolhe pelo perfil.
    const blank = toProfile({
      ...base,
      adults: base.adults.map((a) => ({ ...a, age: '' })),
    });
    expect(blank.adults.every((a) => a.age === undefined)).toBe(true);
  });
});
