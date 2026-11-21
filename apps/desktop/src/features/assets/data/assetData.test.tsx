import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

import { PORTFOLIO_UI_DEFAULTS, usePortfolioUi } from '../../portfolio/state';
import { fixToday, quietCanvas } from '../../portfolio/testing';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
  usePortfolioUi.setState({ ...PORTFOLIO_UI_DEFAULTS });
});
afterEach(() => {
  vi.useRealTimers();
});

describe('Dados dos ativos com fonte', () => {
  it('Saúde dos ativos: marca só os vazios de confiança alta, mostra conflito e grava', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/carteira?aba=saude');
    const panel = await screen.findByRole(
      'region',
      { name: 'Dados dos ativos' },
      { timeout: 15_000 },
    );
    // CNPJ do emissor da Petrobras: vazio, oficial → marcado de saída.
    const isin = await within(panel).findByRole('checkbox', { name: 'PETR4 · CNPJ do emissor' });
    expect((isin as HTMLInputElement).checked).toBe(true);
    // Setor da Taesa: o seu valor difere da CVM → só aviso, desmarcado.
    const taesa = within(panel).getByRole('checkbox', { name: 'TAEE11 · Setor' });
    expect((taesa as HTMLInputElement).checked).toBe(false);
    expect(within(panel).getAllByText('Trocar o seu valor por este').length).toBeGreaterThan(0);
    // CNPJ do BOVA11 achado pelo nome: confiança média, desmarcado.
    const bova = within(panel).getByRole('checkbox', { name: 'BOVA11 · CNPJ' });
    expect((bova as HTMLInputElement).checked).toBe(false);
    // Conflito entre duas fontes oficiais: nenhuma escolhida.
    const radios = within(within(panel).getByRole('group', { name: 'escolha uma' })).getAllByRole(
      'radio',
    );
    expect(radios.length).toBe(2);
    expect(radios.every((r) => !(r as HTMLInputElement).checked)).toBe(true);
    expect(within(panel).getByText(/Dados de fontes externas e de IA/)).toBeTruthy();

    const save = within(panel).getByRole('button', { name: /^Gravar marcados/ });
    await user.click(save);
    expect(await screen.findByText(/campo\(s\) gravado\(s\)/)).toBeTruthy();
    const petr = (await mockCall('assets.field_sources', { asset_id: 'ast-petr4' })) as {
      fields: { status: { field: string; current: string | null; origin: string | null } }[];
    };
    const cnpj = petr.fields.find((f) => f.status.field === 'issuer_cnpj')!.status;
    expect(cnpj.current).toBe('33.000.167/0001-01');
    expect(cnpj.origin).toBe('enrichment');
    const taee = (await mockCall('assets.get', { id: 'ast-taee11' })) as { sector: string };
    expect(taee.sector).toBe('Utilidade pública');
  });

  it('Cadastro do ativo: origem, sugestão, de onde veio e trava', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ativos?ativo=ast-petr4&secao=cadastro');
    const panel = await screen.findByRole(
      'region',
      { name: 'Dados do cadastro' },
      { timeout: 15_000 },
    );
    expect(within(panel).getAllByText('já preenchido').length).toBeGreaterThan(0);
    await user.click(within(panel).getByRole('button', { name: 'Completar dados' }));
    expect(await screen.findByText(/campo\(s\) gravado\(s\)/)).toBeTruthy();
    expect(
      (await within(panel).findAllByText(/CVM · FCA · 14\/10\/2026 · 90/)).length,
    ).toBeGreaterThan(0);
    const unlocked = await within(panel).findAllByRole('button', { name: 'Travar campo' });
    await user.click(unlocked[0]!);
    expect(await within(panel).findByText('travado')).toBeTruthy();
    await user.click(within(panel).getAllByRole('button', { name: 'De onde veio' })[0]!);
    expect(await within(panel).findByText('Leituras das fontes no cofre')).toBeTruthy();
  });

  it('Consultar fontes traz as leituras da rede (LEI do GLEIF)', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ativos?ativo=ast-petr4&secao=cadastro');
    const panel = await screen.findByRole(
      'region',
      { name: 'Dados do cadastro' },
      { timeout: 15_000 },
    );
    await user.click(within(panel).getByRole('button', { name: 'Consultar fontes' }));
    expect(await screen.findByText(/leitura\(s\) nova\(s\) das fontes/)).toBeTruthy();
    expect(await within(panel).findByText(/5493000J801JZRCMFE49/)).toBeTruthy();
    const r = (await mockCall('assets.data_gaps', { asset_ids: ['ast-petr4'] })) as {
      assets: { fields: { field: string; candidate: { source: string } | null }[] }[];
    };
    const lei = r.assets[0]!.fields.find((f) => f.field === 'lei');
    expect(lei?.candidate?.source).toBe('gleif');
  });

  it('Alocação por setor agrupa pelo setor do cadastro e as outras classes pela classe', async () => {
    renderApp('unlocked', {}, '#/carteira?aba=alocacao');
    const list = await screen.findByRole('list', { name: 'Por setor' }, { timeout: 15_000 });
    expect(within(list).getByText('Financeiro')).toBeTruthy();
    // Classes sem setor por natureza ficam pela classe, não como "sem setor".
    expect(within(list).getByText('Renda fixa')).toBeTruthy();
    expect(within(list).queryByText('Sem setor')).toBeNull();
  });

  it('Carteira avisa dos dados para completar e o ativo mostra o atalho', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/carteira');
    const note = await screen.findByText(
      /dado\(s\) dos seus ativos podem ser completados/,
      undefined,
      {
        timeout: 15_000,
      },
    );
    expect(note).toBeTruthy();
    await user.click(screen.getByRole('link', { name: 'Ver dados' }));
    expect(await screen.findByRole('region', { name: 'Dados dos ativos' })).toBeTruthy();
  });

  it('cabeçalho do ativo leva à aba Cadastro quando há dados para completar', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ativos?ativo=ast-petr4');
    const chip = await screen.findByRole(
      'button',
      { name: /dado\(s\) para completar/ },
      { timeout: 15_000 },
    );
    await user.click(chip);
    expect(await screen.findByRole('region', { name: 'Dados do cadastro' })).toBeTruthy();
  });
});
