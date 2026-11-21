import { expect, test, type APIRequestContext } from '@playwright/test';

import { openVault, passGates } from './helpers';

/** Nota SINACOR em texto (o núcleo aceita texto puro além de PDF). */
const NOTE = [
  'NOTA DE NEGOCIAÇÃO',
  'Nr. nota Folha Data pregão',
  '778899 1 13/10/2026',
  '1-BOVESPA C VISTA PETROBRAS PN N2 100 28,50 2.850,00 D',
  '1-BOVESPA C FRACIONARIO ITAUSA PN N1 7 10,00 70,00 D',
  '1-BOVESPA C VISTA FII HGLG PAX HGLG11 CI 10 160,00 1.600,00 D',
  'Taxa de liquidação 0,95 D',
  'Emolumentos 0,23 D',
  'Líquido para 15/10/2026 4.521,18 D',
].join('\n');

async function core<T>(request: APIRequestContext, cmd: string, args: unknown): Promise<T> {
  const res = await request.post('http://127.0.0.1:7799/api', {
    headers: { Authorization: 'Bearer e2e-token' },
    data: { cmd, args },
  });
  const body = (await res.json()) as { ok: boolean; data?: T; error?: unknown };
  if (!body.ok) throw new Error(`${cmd}: ${JSON.stringify(body.error)}`);
  return body.data as T;
}

test('nota de corretagem: prévia, ativo escolhido, gravação e desfazer no núcleo real', async ({
  page,
  request,
}) => {
  await openVault(page);
  const account = await core<{ id: string }>(request, 'accounts.create', {
    name: 'Corretora E2E',
    kind: 'brokerage',
  });
  await core(request, 'assets.create', {
    ticker: 'PETR4',
    name: 'Petrobras PN',
    class: 'stock_br',
  });
  // Sem ITSA4 cadastrado (nem dados da CVM), a ITAUSA fica sem ativo e o usuário escolhe.
  await core(request, 'assets.create', {
    ticker: 'BBAS3',
    name: 'Banco do Brasil ON',
    class: 'stock_br',
  });

  // Recarrega: a conta e os ativos acabaram de ser criados direto no núcleo.
  await page.goto('/#/carteira?importar=notas');
  await page.reload();
  await passGates(page);
  const dialog = page.getByRole('dialog', { name: 'Importar notas de corretagem' });
  await expect(dialog.getByRole('combobox', { name: 'Conta da corretora' })).toContainText(
    'Corretora E2E',
  );
  await dialog.getByLabel('Notas de corretagem (PDF)').setInputFiles({
    name: 'nota-778899.txt',
    mimeType: 'text/plain',
    buffer: Buffer.from(NOTE, 'utf-8'),
  });

  const note = dialog.getByRole('region', { name: 'Nota 778899' });
  await expect(note).toContainText('Total de taxas');
  await expect(dialog.getByText(/Líquido da nota ≠/)).toHaveCount(0);
  const table = dialog.getByRole('table', { name: 'Operações das notas' });
  await expect(table.getByText('Sem ativo')).toBeVisible();
  await expect(table.getByText('cadastra HGLG11', { exact: false })).toBeVisible();

  await dialog
    .getByRole('group', { name: 'Ativo para ITAUSA PN N1' })
    .getByRole('combobox', { name: 'Ativo cadastrado' })
    .click();
  await page.getByRole('option', { name: /BBAS3/ }).click();
  await expect(
    table.getByRole('row', { name: /BBAS3 ITAUSA PN N1 corrigido por você/ }),
  ).toBeVisible();
  await dialog.getByRole('button', { name: 'Gravar 3 operação(ões)' }).click();
  await expect(dialog.getByText(/3 operação\(ões\) nova\(s\) e 0 completada/)).toBeVisible();

  const trades = () =>
    core<{ ticker: string; fees: number; note_number: string | null }[]>(
      request,
      'portfolio.list_trades',
      { account_id: account.id },
    );
  const written = await trades();
  expect(written.map((t) => t.ticker).sort()).toEqual(['BBAS3', 'HGLG11', 'PETR4']);
  expect(written.reduce((s, t) => s + t.fees, 0)).toBe(118);
  expect(written.every((t) => t.note_number === '778899')).toBe(true);

  await dialog.getByRole('button', { name: 'Desfazer importação' }).click();
  await expect(dialog).toBeHidden();
  await expect.poll(async () => (await trades()).length).toBe(0);
});
