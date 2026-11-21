import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

import { PORTFOLIO_UI_DEFAULTS, usePortfolioUi } from '../../portfolio/state';
import { fixToday, quietCanvas } from '../../portfolio/testing';
import type { JobInfo } from '@/api/generated/JobInfo';
import { jobLinkHref, newlyDoneStudies } from '@/app/BackgroundTasks';

import { studyText } from './StudyPanel';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

beforeAll(quietCanvas);
beforeEach(() => {
  fixToday();
  usePortfolioUi.setState({ ...PORTFOLIO_UI_DEFAULTS });
});
afterEach(() => {
  vi.useRealTimers();
});

describe('AI Study', () => {
  it('always-visible notice, consent, context, package, generate and open with the check', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/ativos?ativo=ast-taee11&secao=estudo');
    const notice = await screen.findByRole(
      'note',
      { name: 'Gerado por IA: pode conter erros' },
      { timeout: 15_000 },
    );
    expect(within(notice).getByText(/provavelmente contém, informações incorretas/)).toBeTruthy();
    // First-time box, with no prior checkmark.
    const box = screen.getByRole('checkbox', { name: /Entendi que o estudo pode estar errado/ });
    expect((box as HTMLInputElement).checked).toBe(false);
    const go = screen.getByRole('button', { name: 'Continuar' });
    expect((go as HTMLButtonElement).disabled).toBe(true);
    await user.click(box);
    await user.click(go);
    // Context and position options off by default.
    const ctx = await screen.findByLabelText('Contexto para a IA');
    await user.type(ctx, 'Tenho para renda');
    await user.click(screen.getByRole('button', { name: 'Salvar contexto' }));
    expect(await screen.findByText('Contexto salvo')).toBeTruthy();
    const pos = screen.getByRole('checkbox', { name: /Incluir meu peso e ganho/ });
    expect((pos as HTMLInputElement).checked).toBe(false);
    // The exact package that would go to the engine.
    await user.click(screen.getByRole('button', { name: 'O que vai para Anthropic' }));
    const pre = await screen.findByLabelText('Pacote do estudo');
    expect(pre.textContent).toContain('Tenho para renda');
    expect(pre.textContent).not.toContain('my_position');
    await user.click(screen.getByRole('button', { name: 'Gerar estudo' }));
    const history = await screen.findByRole('list', { name: 'Estudos anteriores' });
    expect(await within(history).findByText(/1 número\(s\) não conferido\(s\)/)).toBeTruthy();
    // The new study opens by itself, without having to look in the history.
    const study = await screen.findByRole('article', { name: 'Estudo aberto' });
    expect(
      within(study).getByText(/Gerado por IA em .* com Claude · pode conter erros/),
    ).toBeTruthy();
    expect(within(study).getByText('confere com os dados')).toBeTruthy();
    expect(within(study).getByText('não encontrado nos dados')).toBeTruthy();
    expect(
      within(study).getByText(/Gerado por IA · pode conter erros · confira nas fontes/),
    ).toBeTruthy();
  });

  it('the copied text carries the full notice at the end', () => {
    const text = studyText({
      id: 's',
      asset_id: 'a',
      created_at: '2026-10-20T10:00:00Z',
      engine_name: 'Claude',
      provider: 'Anthropic',
      model: null,
      question: null,
      confidence: 'low',
      unchecked: 0,
      summary: 'Resumo.',
      include_position: false,
      include_values: false,
      context_snapshot: null,
      disclaimer_version: '1',
      checks: [],
      output: {
        summary: 'Resumo.',
        business: null,
        strengths: [],
        risks: [],
        key_numbers: [],
        changes: null,
        context_check: null,
        manual_checks: [],
        sources: [],
        confidence: 'low',
        missing: null,
      },
    });
    expect(text.startsWith('Gerado por IA em 20/10/2026 com Claude')).toBe(true);
    expect(text.endsWith('a decisão de comprar, vender ou manter é sua.')).toBe(true);
  });

  it('the task list notifies once per finished study and links to it', () => {
    const job = (id: string, status: JobInfo['status']): JobInfo => ({
      job_id: id,
      kind: 'routine',
      task: 'study',
      status,
      created_at: '2026-10-21T10:00:00Z',
      conversation_id: null,
      engine_name: null,
      title: 'Estudo com IA',
      activity: null,
      finished_at: null,
      cancelling: false,
      link: { screen: 'asset_study', asset_id: 'ast-taee11', id: 'study-1' },
    });
    const seen = new Set<string>();
    expect(newlyDoneStudies(seen, [job('a', 'running')])).toEqual([]);
    expect(newlyDoneStudies(seen, [job('a', 'done')]).map((j) => j.job_id)).toEqual(['a']);
    expect(newlyDoneStudies(seen, [job('a', 'done')])).toEqual([]);
    expect(jobLinkHref(job('a', 'done').link!)).toBe(
      '/ativos?ativo=ast-taee11&secao=estudo&estudo=study-1',
    );
  });

  it('?estudo= opens the study directly on the tab', async () => {
    renderApp('unlocked', {}, '#/carteira');
    await screen.findByRole('heading', { name: /Carteira/ }, { timeout: 15_000 });
    await mockCall('assets.study_accept', {});
    await mockCall('assets.study_start', { asset_id: 'ast-taee11' });
    window.location.hash = '#/ativos?ativo=ast-taee11&secao=estudo&estudo=study-1';
    expect(
      await screen.findByRole('article', { name: 'Estudo aberto' }, { timeout: 15_000 }),
    ).toBeTruthy();
  });
});
