import { expect, test, type APIRequestContext } from '@playwright/test';

import { openVault, passGates } from './helpers';

async function core<T>(request: APIRequestContext, cmd: string, args: unknown): Promise<T> {
  const res = await request.post('http://127.0.0.1:7799/api', {
    headers: { Authorization: 'Bearer e2e-token' },
    data: { cmd, args },
  });
  const body = (await res.json()) as { ok: boolean; data?: T; error?: unknown };
  if (!body.ok) throw new Error(`${cmd}: ${JSON.stringify(body.error)}`);
  return body.data as T;
}

/** Titles with no words in common (the core merges similar titles into one reinforcement). */
const TITLES = [
  'Gráfico de patrimônio pisca ao trocar o tema',
  'Importação OFX ignora tarifa bancária',
  'Busca global não acha categoria arquivada',
  'Exportar orçamento anual em planilha',
  'Cotação intradiária atrasa depois das 17h',
  'Tooltip cortado na borda direita',
  'Recorrência mensal pula fevereiro',
  'Meta de viagem sem projeção de câmbio',
  'Atalho de teclado duplica lançamento',
  'Relatório PDF trunca nomes longos',
  'Cartão virtual aparece como conta corrente',
  'Saldo previsto ignora transferência agendada',
  'Proventos de FII sem data com',
  'Tela de bens demora com muitas fotos',
  'Lembrete de IPTU dispara duas vezes',
  'Ícone do glossário sem contraste no tema claro',
];

test.use({ viewport: { width: 1440, height: 1000 } });

test('Developer: board with 16 items, next in the queue, panel and drag with undo', async ({
  page,
  request,
}) => {
  await openVault(page);
  await core(request, 'settings.set', { developer_tools: true });
  // The vault is the same as in the other scenarios: the core may have already logged feedback by itself
  // (e.g. a score without critical data for an asset registered earlier). Counts from what is there.
  const existing = await core<{ status: string }[]>(request, 'devfeedback.list', {});
  const open = existing.filter((f) => f.status === 'aberto').length + TITLES.length;
  const openColumn = new RegExp(`^Aberto: ${open} itens`);
  const ids: string[] = [];
  for (const [i, title] of TITLES.entries()) {
    const r = await core<{ feedback: { id: string }; deduplicated: boolean }>(
      request,
      'devfeedback.add',
      {
        title,
        body: `Problema de exemplo número ${i + 1}, com dados inventados.`,
        area: i % 2 ? 'ui' : 'dados',
        kind: i % 3 ? 'melhoria' : 'bug',
      },
    );
    expect(r.deduplicated).toBe(false);
    ids.push(r.feedback.id);
  }
  // The highest-priority one, with proposal, acceptance criteria and where to change.
  const urgent = ids[9]!;
  await core(request, 'devfeedback.set_priority', { id: urgent, importance: 9 });
  await core(request, 'devfeedback.update', {
    id: urgent,
    proposal: 'Quebrar o nome em duas linhas no relatório.',
    acceptance_criteria: 'Nome com 80 caracteres aparece inteiro no PDF.',
    code_pointers: ['crates/treasury-core/src/reports/pdf.rs:120'],
  });

  await page.goto('/#/desenvolvedor');
  await page.reload();
  await passGates(page);
  const board = page.getByRole('group', { name: 'Quadro de feedback' });
  const column = (name: RegExp) => board.getByRole('region', { name });
  // In 5 seconds: the next in the queue and its priority visible, at the top of Open.
  const next = page.getByRole('region', { name: 'Próximo da fila' });
  await expect(next).toContainText('Relatório PDF trunca nomes longos', { timeout: 5_000 });
  await expect(next).toContainText('Prioridade 9');
  await expect(column(openColumn)).toBeVisible();
  const aberto = column(/^Aberto/).getByRole('listitem');
  await expect(aberto.first()).toHaveAccessibleName(/Relatório PDF trunca nomes longos/);

  // Side panel: fixed sections, clickable where-to-change, discreet origin.
  await next.getByRole('button', { name: /Relatório PDF/ }).click();
  const panel = page.getByRole('dialog', { name: 'Relatório PDF trunca nomes longos' });
  await expect(panel.getByRole('region', { name: 'Critério de aceite' })).toContainText(
    'Nome com 80 caracteres',
  );
  await expect(panel.getByRole('region', { name: 'Proposta' })).toContainText('duas linhas');
  await expect(
    panel.getByRole('button', { name: 'crates/treasury-core/src/reports/pdf.rs:120' }),
  ).toBeVisible();
  await expect(panel.getByRole('button', { name: 'Copiar Markdown' })).toBeVisible();
  await panel.getByRole('button', { name: 'Fechar' }).click();
  await expect(panel).toBeHidden();

  // Dragging to In progress changes the status; "Desfazer" in the notice reverts it.
  // The second in the column (right below the urgent one): the newest, the last registered.
  const card = aberto.nth(1);
  await expect(card).toHaveAccessibleName(/Ícone do glossário/);
  // Dropped on the column header (without scrolling the page mid-drag).
  await card.dragTo(column(/^Em andamento/), { targetPosition: { x: 60, y: 16 } });
  await expect(column(/^Em andamento: 1 item/)).toBeVisible();
  await expect(column(/^Em andamento/)).toContainText('Ícone do glossário sem contraste');
  const moved = await core<{ feedback: { status: string } }>(request, 'devfeedback.get', {
    id: ids[15],
  });
  expect(moved.feedback.status).toBe('em_andamento');
  await page.getByRole('button', { name: 'Desfazer' }).click();
  await expect(column(openColumn)).toBeVisible();
  const back = await core<{ feedback: { status: string } }>(request, 'devfeedback.get', {
    id: ids[15],
  });
  expect(back.feedback.status).toBe('aberto');

  // Dragging over another card in the same column reorders it (manual position, without changing priority).
  // The third in the column (priority 1) goes to the top, above the urgent one.
  const lastName = (await aberto.nth(2).getAttribute('aria-label'))!.replace('Feedback: ', '');
  await aberto.nth(2).dragTo(aberto.first(), { targetPosition: { x: 40, y: 12 } });
  await expect(aberto.first()).toHaveAccessibleName(`Feedback: ${lastName}`);
  const top = await core<{ feedback: { position: number; importance: number } }>(
    request,
    'devfeedback.get',
    { id: ids[TITLES.indexOf(lastName)] },
  );
  expect(top.feedback).toMatchObject({ position: 1, importance: 1 });

  // The current list stays one click away.
  await page.getByRole('group', { name: 'Visão' }).getByRole('button', { name: 'Lista' }).click();
  await expect(
    page.getByRole('list', { name: 'Feedback' }).getByRole('listitem', { name: /^Feedback: / }),
  ).toHaveCount(existing.length + TITLES.length);
});
