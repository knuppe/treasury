import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { mockCall, mockCallLog } from '@/api/mock';
import { mockScoreEmpty } from '@/api/mock/score';
import type { AssetScoreFeedback } from '@/api/generated/AssetScoreFeedback';
import type { AssetScoreMethodology } from '@/api/generated/AssetScoreMethodology';
import { renderApp } from '@/test/renderApp';

import { PORTFOLIO_UI_DEFAULTS, usePortfolioUi } from '../portfolio/state';
import { fixToday, quietCanvas } from '../portfolio/testing';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
  usePortfolioUi.setState({ ...PORTFOLIO_UI_DEFAULTS });
});
afterEach(() => {
  vi.useRealTimers();
});

const asset = (id: string) => renderApp('unlocked', {}, `#/ativos?ativo=${id}&secao=saude`);

describe('Score de saúde do ativo · aba Saúde', () => {
  it('mostra score, faixa, sub-scores, por que a nota e as métricas da dimensão', async () => {
    const user = userEvent.setup();
    asset('ast-petr4');
    const header = await screen.findByRole('region', { name: 'Score de saúde do ativo' });
    expect(screen.getByRole('tab', { name: 'Saúde', selected: true })).toBeTruthy();
    // Nota inteira, faixa como texto (não só a cor), confiança e sub-scores.
    expect(within(header).getByText('Score')).toBeTruthy();
    expect(within(header).getByText(/^(Robusto|Saudável|Atenção|Risco)$/)).toBeTruthy();
    expect(within(header).getByText('Confiança')).toBeTruthy();
    expect(within(header).getByText('Saúde')).toBeTruthy();
    expect(within(header).getByText('Atratividade')).toBeTruthy();
    expect(within(header).getByText(/\+2,4 pontos/)).toBeTruthy();

    const why = screen.getByRole('region', { name: 'Por que essa nota' });
    const favor = within(why).getByRole('list', { name: 'A favor' });
    expect(within(favor).getAllByRole('listitem')).toHaveLength(3);
    expect(within(why).getByRole('list', { name: 'Contra' })).toBeTruthy();
    const flags = within(why).getByRole('list', { name: 'Red flags' });
    expect(within(flags).getByText('Controle estatal')).toBeTruthy();
    expect(within(flags).getByText('−5 pontos')).toBeTruthy();

    // Dimensão abre a tabela das métricas: valor pela unidade e percentil entre os pares.
    const dims = screen.getByRole('list', { name: 'Dimensões do score' });
    const solv = within(dims).getByRole('button', { name: /Endividamento e liquidez/ });
    expect(solv.getAttribute('aria-expanded')).toBe('false');
    await user.click(solv);
    const table = await screen.findByRole('grid', { name: 'Métricas · Endividamento e liquidez' });
    expect(within(table).getByText('Dívida líquida / EBITDA')).toBeTruthy();
    expect(within(table).getByText('1,42×')).toBeTruthy();
    expect(within(table).getAllByText(/^p\d+ de 9 pares do setor Petróleo e Gás$/).length).toBe(4);
    expect(within(table).getByText('crítica')).toBeTruthy();

    // Modelos clássicos, conferência e rodapé.
    expect(screen.getByRole('region', { name: 'Modelos clássicos' })).toBeTruthy();
    const piotroski = within(screen.getByRole('region', { name: 'Modelos clássicos' })).getByRole(
      'list',
      { name: 'Itens · Piotroski F-Score' },
    );
    expect(within(piotroski).getAllByLabelText('atende')).toHaveLength(8);
    expect(within(piotroski).getAllByLabelText('não atende')).toHaveLength(1);
    expect(screen.getByRole('grid', { name: 'Conferência com outras fontes' })).toBeTruthy();
    expect(await screen.findByText('Histórico do score')).toBeTruthy();
    expect(screen.getByText(/Não é recomendação de investimento/)).toBeTruthy();
  });

  it('dados insuficientes: sem score, com a métrica crítica que falta', async () => {
    asset('ast-mglu3');
    const notice = await screen.findByText('Score não calculado: falta um dado crítico');
    const box = notice.closest('[role="status"]') as HTMLElement;
    const critical = within(box).getByRole('list', { name: 'Dados críticos ausentes' });
    expect(within(critical).getByText('Dívida líquida / EBITDA')).toBeTruthy();
    expect(within(box).getByText(/registrado para o desenvolvedor/)).toBeTruthy();
    const header = screen.getByRole('region', { name: 'Score de saúde do ativo' });
    expect(within(header).getByText('Dados insuficientes')).toBeTruthy();
    // As dimensões continuam lá, sem nota.
    const dims = screen.getByRole('list', { name: 'Dimensões do score' });
    expect(within(dims).getAllByRole('img', { name: /: sem nota$/ }).length).toBe(9);
  });

  it('classe sem metodologia (cripto) explica que o score não existe ainda', async () => {
    asset('ast-btc');
    expect(
      await screen.findByText('Score de saúde ainda não disponível para criptoativos'),
    ).toBeTruthy();
    expect(screen.getByText(/ainda não foi implementado/)).toBeTruthy();
  });

  it('troca o segmento da metodologia e volta ao automático', async () => {
    const user = userEvent.setup();
    asset('ast-wege3');
    await screen.findByRole('region', { name: 'Score de saúde do ativo' });
    await user.click(await screen.findByRole('combobox', { name: 'Segmento da metodologia' }));
    await user.click(await screen.findByRole('option', { name: 'Tecnologia e comunicação' }));
    await waitFor(async () => {
      const m = (await mockCall('score.methodology', {})) as AssetScoreMethodology;
      expect(m.overrides.WEGE3).toBe('tech');
    });
    expect(await screen.findByText(/Escolhido por você/)).toBeTruthy();
    expect(mockCallLog()).toContain('score.set_segment');
  });

  it('"Discordo" registra a opinião e mostra a lista', async () => {
    const user = userEvent.setup();
    asset('ast-itub4');
    await screen.findByRole('region', { name: 'Score de saúde do ativo' });
    await user.click(screen.getByRole('button', { name: 'Discordo' }));
    const dialog = await screen.findByRole('dialog', { name: /Discordo do score · ITUB4/ });
    // Sem motivo, não envia.
    await user.click(within(dialog).getByRole('button', { name: 'Registrar opinião' }));
    expect(within(dialog).getByText('Conte o motivo.')).toBeTruthy();
    await user.type(
      within(dialog).getByLabelText('Motivo'),
      'Banco com ROE alto e inadimplência baixa',
    );
    await user.click(within(dialog).getByRole('button', { name: 'Registrar opinião' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    const list = await screen.findByRole('list', { name: 'Opiniões registradas' });
    expect(within(list).getByText('Banco com ROE alto e inadimplência baixa')).toBeTruthy();
    const saved = (await mockCall('score.feedback.list', {
      asset_id: 'ast-itub4',
    })) as AssetScoreFeedback[];
    expect(saved).toHaveLength(1);
    expect(saved[0]).toMatchObject({ verdict: 'discordo', expected_band: null });
  });

  it('registra um evento qualitativo e permite resolvê-lo', async () => {
    const user = userEvent.setup();
    asset('ast-itub4');
    await screen.findByRole('region', { name: 'Score de saúde do ativo' });
    await user.click(screen.getByRole('button', { name: 'Novo evento' }));
    const dialog = await screen.findByRole('dialog', { name: /Novo evento · ITUB4/ });
    await user.click(within(dialog).getByRole('button', { name: 'Registrar e aplicar' }));
    expect(within(dialog).getByText('Descreva o evento.')).toBeTruthy();
    await user.type(within(dialog).getByLabelText('O que aconteceu'), 'Troca abrupta do CFO');
    await user.type(within(dialog).getByLabelText('Data'), '01102026');
    await user.click(within(dialog).getByRole('button', { name: 'Registrar e aplicar' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    const list = await screen.findByRole('list', { name: 'Eventos registrados' });
    expect(within(list).getByText('Troca abrupta do CFO')).toBeTruthy();
    expect(within(list).getByText('Aceito (vale na nota)')).toBeTruthy();
    await user.click(within(list).getByRole('button', { name: 'Marcar como resolvido' }));
    await waitFor(() => expect(within(list).getByText('Resolvido')).toBeTruthy());
  });
});

describe('Carteira › Saúde dos ativos', () => {
  it('lista os ativos com posição ou na watchlist e leva à aba Saúde', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/carteira?aba=saude');
    const grid = await screen.findByRole('grid', { name: 'Saúde dos ativos' });
    // Watchlist (VALE3, TAEE11) e posições; com score primeiro.
    expect(within(grid).getByText('VALE3')).toBeTruthy();
    expect(within(grid).getByText('TAEE11')).toBeTruthy();
    const rows = within(grid).getAllByRole('row').slice(1);
    expect(rows.length).toBeGreaterThan(8);
    expect(within(rows[rows.length - 1]!).getByText(/classe não suportada/)).toBeTruthy();
    const mglu = within(grid).getByText('MGLU3').closest('tr')!;
    expect(within(mglu).getByText('Dados insuficientes')).toBeTruthy();
    const bbas = within(grid).getByText('BBAS3').closest('tr')!;
    expect(within(bbas).getByText('Parcial')).toBeTruthy();
    await user.click(within(grid).getByRole('link', { name: 'PETR4' }));
    expect(await screen.findByRole('region', { name: 'Score de saúde do ativo' })).toBeTruthy();
    expect(window.location.hash).toContain('secao=saude');
  });

  it('sem ativos: estado vazio', async () => {
    renderApp('unlocked', {}, '#/carteira?aba=saude');
    mockScoreEmpty();
    expect(await screen.findByText('Nenhum ativo na carteira ou na watchlist')).toBeTruthy();
  });
});
