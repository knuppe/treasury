/**
 * Roteiro de contrato das telas 02, 03 e 11: chama os comandos que elas usam, na
 * ordem de um uso real, confere as formas (`shapes.ts`) e algumas regras de
 * negócio (saldo, divisão, transferência entre moedas, desfazer).
 *
 * Roda contra o núcleo real (`scripts/contract-ledger.mjs`) e contra o mock
 * (`src/api/mock/ledger/contract.test.ts`). Sem aliases nem APIs de navegador.
 */
import * as S from './shapes.ts';

export type Caller = (cmd: string, args?: unknown) => Promise<unknown>;

export interface StepResult {
  step: string;
  cmd: string;
  ok: boolean;
  findings: S.Finding[];
  error?: string;
}

export interface FlowOptions {
  /** Senha mestra atual do cofre (já aberto). */
  password: string;
  /** Pasta gravável fora das pastas do Treasury (exportação). */
  exportDir: string;
  /** Hoje (AAAA-MM-DD). */
  today: string;
  /** Restaurar um backup no fim (troca o banco). */
  restore?: boolean;
  /** Chamar `market.sync` (usa a rede). */
  sync?: boolean;
}

interface ApiErr {
  code?: string;
  message?: string;
}

function b64(text: string): string {
  const bytes = new TextEncoder().encode(text);
  let bin = '';
  for (const b of bytes) bin += String.fromCharCode(b);
  return btoa(bin);
}

function addDays(iso: string, n: number): string {
  const d = new Date(`${iso}T12:00:00Z`);
  d.setUTCDate(d.getUTCDate() + n);
  return d.toISOString().slice(0, 10);
}

export async function runLedgerContract(call: Caller, opts: FlowOptions): Promise<StepResult[]> {
  const results: StepResult[] = [];
  const tag = `ct${Date.now().toString(36)}`;

  async function step<T>(
    name: string,
    cmd: string,
    args: unknown,
    spec: S.Spec | null,
    verify?: (v: T) => string | null,
  ): Promise<T | undefined> {
    try {
      const value = (await call(cmd, args)) as T;
      const findings = spec ? S.check(value, spec) : [];
      const problem = verify?.(value) ?? null;
      if (problem) findings.push({ level: 'erro', path: '$', message: problem });
      results.push({ step: name, cmd, ok: !findings.some((f) => f.level === 'erro'), findings });
      return value;
    } catch (e) {
      const err = e as ApiErr;
      results.push({
        step: name,
        cmd,
        ok: false,
        findings: [],
        error: `${err.code ?? 'erro'}: ${err.message ?? String(e)}`,
      });
      return undefined;
    }
  }

  /** Espera um erro com este código (validação do núcleo). */
  async function expectError(
    name: string,
    cmd: string,
    args: unknown,
    code: string,
  ): Promise<void> {
    try {
      await call(cmd, args);
      results.push({
        step: name,
        cmd,
        ok: false,
        findings: [{ level: 'erro', path: '$', message: `esperava erro ${code}, mas deu certo` }],
      });
    } catch (e) {
      const err = e as ApiErr;
      const ok = err.code === code && typeof err.message === 'string' && err.message.length > 0;
      results.push({
        step: name,
        cmd,
        ok,
        findings: ok
          ? []
          : [
              {
                level: 'erro',
                path: '$',
                message: `esperava ${code}, veio ${err.code}: ${err.message}`,
              },
            ],
      });
    }
  }

  const T = opts.today;

  // ── Família ───────────────────────────────────────────────────
  await step('família', 'household.get', {}, S.HouseholdShape);
  const members =
    (await step<unknown[]>('titulares', 'household.list_members', { include_archived: true }, [
      S.MemberShape,
    ])) ?? [];
  const memberId = (members[0] as { id?: string } | undefined)?.id;
  const inst = await step<{ id: string }>(
    'criar instituição',
    'household.create_institution',
    { name: `Banco ${tag}`, kind: 'bank', code: '0001' },
    S.InstitutionShape,
  );
  if (inst)
    await step(
      'alterar instituição',
      'household.update_institution',
      { id: inst.id, website: 'https://exemplo.com.br' },
      S.InstitutionShape,
    );
  await step('instituições', 'household.list_institutions', {}, [S.InstitutionShape]);

  // ── Contas ────────────────────────────────────────────────────
  const checking = await step<{ id: string; balance: number }>(
    'criar conta corrente',
    'accounts.create',
    {
      name: `Corrente ${tag}`,
      kind: 'checking',
      opening_balance: 100_000,
      opening_date: addDays(T, -60),
      ...(inst ? { institution_id: inst.id } : {}),
      ...(memberId ? { member_ids: [memberId] } : {}),
    },
    S.AccountViewShape,
    (a) => (a.balance === 100_000 ? null : `saldo inicial esperado 100000, veio ${a.balance}`),
  );
  await expectError(
    'cartão sem dados do cartão',
    'accounts.create',
    { name: `Cartão ruim ${tag}`, kind: 'credit_card' },
    'invalid',
  );
  const card = await step<{ id: string; card: unknown; available_credit: number | null }>(
    'criar cartão',
    'accounts.create',
    {
      name: `Cartão ${tag}`,
      kind: 'credit_card',
      opening_date: addDays(T, -60),
      card: {
        credit_limit: 500_000,
        closing_day: 3,
        due_day: 10,
        payment_account_id: checking?.id ?? null,
        network: 'Visa',
        last4: '1234',
      },
    },
    S.AccountViewShape,
    (a) =>
      S.check(a.card, S.CardDetailsShape).length
        ? 'card fora da forma CardDetails'
        : a.available_credit === 500_000
          ? null
          : `limite disponível esperado 500000, veio ${a.available_credit}`,
  );
  const usd = await step<{ id: string }>(
    'criar conta em USD',
    'accounts.create',
    {
      name: `Dólar ${tag}`,
      kind: 'foreign',
      currency: 'USD',
      opening_balance: 10_000,
      opening_date: addDays(T, -60),
    },
    S.AccountViewShape,
  );
  await step('listar contas', 'accounts.list', { include_archived: false }, [S.AccountViewShape]);
  if (checking) {
    await step(
      'alterar conta',
      'accounts.update',
      { id: checking.id, notes: 'conta de teste', color: null },
      S.AccountViewShape,
    );
    await step('obter conta', 'accounts.get', { id: checking.id }, S.AccountViewShape);
  }

  // ── Câmbio ────────────────────────────────────────────────────
  const rate = await step<{ id: string }>(
    'câmbio manual',
    'fx.set',
    { base: 'USD', quote: 'BRL', date: T, rate: '5,4321' },
    S.FxRateShape,
  );
  await step('listar câmbio', 'fx.list', { base: 'USD', quote: 'BRL', limit: 5 }, [S.FxRateShape]);
  await step(
    'converter',
    'fx.convert',
    { amount: 10_000, from: 'USD', to: 'BRL', date: T },
    S.ConvertedShape,
    (c) =>
      (c as { amount: number }).amount === 54_321
        ? null
        : `USD 100,00 × 5,4321 deveria dar 54321, veio ${(c as { amount: number }).amount}`,
  );
  if (usd)
    await step('conta USD em BRL', 'accounts.get', { id: usd.id }, S.AccountViewShape, (a) =>
      (a as { balance_base: number | null }).balance_base === 54_321
        ? null
        : 'balance_base não usou o câmbio',
    );
  await expectError(
    'câmbio inválido',
    'fx.set',
    { base: 'USD', quote: 'USD', date: T, rate: '1' },
    'invalid',
  );

  // ── Categorias ────────────────────────────────────────────────
  const tree =
    (await step<{ id: string; name: string; kind: string; children: { id: string }[] }[]>(
      'árvore de categorias',
      'categories.tree',
      { kind: 'expense' },
      [S.CategoryNodeShape],
    )) ?? [];
  await step('lista de categorias', 'categories.list', { include_archived: true }, [
    S.CategoryShape,
  ]);
  const food = tree.find((c) => c.children.length > 0);
  const sub = food
    ? await step<{ id: string }>(
        'criar subcategoria',
        'categories.create',
        { name: `Sub ${tag}`, kind: 'expense', parent_id: food.id },
        S.CategoryShape,
      )
    : undefined;
  const other = food?.children[0]?.id;
  if (sub) {
    await step(
      'renomear categoria',
      'categories.update',
      { id: sub.id, name: `Sub ${tag} 2`, is_essential: true },
      S.CategoryShape,
    );
    await step('uso da categoria', 'categories.usage', { id: sub.id }, S.CategoryUsageShape);
  }
  await expectError(
    'categoria duplicada',
    'categories.create',
    { name: food?.name ?? 'Moradia', kind: 'expense' },
    'conflict',
  );

  // ── Lançamentos ───────────────────────────────────────────────
  let expenseId: string | undefined;
  if (checking && other && sub) {
    const txn = await step<{ id: string; splits: unknown[] }>(
      'criar despesa dividida',
      'transactions.create',
      {
        account_id: checking.id,
        date: addDays(T, -5),
        amount: -12_345,
        description: `MERCADO ${tag.toUpperCase()} CENTRO`,
        payee: 'Mercado',
        status: 'cleared',
        splits: [
          { category_id: other, amount: -10_000 },
          { category_id: sub.id, amount: -2_345, notes: 'parte 2' },
        ],
        tags: ['contrato', tag],
      },
      S.TxnViewShape,
      (t) => (t.splits.length === 2 ? null : 'divisão não voltou com 2 partes'),
    );
    expenseId = txn?.id;
    await expectError(
      'divisão com soma errada',
      'transactions.create',
      {
        account_id: checking.id,
        date: T,
        amount: -1000,
        description: 'x',
        splits: [{ amount: -400 }, { amount: -500 }],
      },
      'invalid',
    );
    await expectError(
      'valor zero',
      'transactions.create',
      { account_id: checking.id, date: T, amount: 0, description: 'x' },
      'invalid',
    );
    if (expenseId) {
      await step(
        'editar lançamento',
        'transactions.update',
        {
          id: expenseId,
          description: `MERCADO ${tag.toUpperCase()} BAIRRO`,
          splits: [],
          category_id: other,
        },
        S.TxnViewShape,
        (t) =>
          (t as { splits: unknown[]; category_id: string | null }).splits.length === 0 &&
          (t as { category_id: string | null }).category_id === other
            ? null
            : 'edição não tirou a divisão',
      );
      await step('obter lançamento', 'transactions.get', { id: expenseId }, S.TxnViewShape);
      await step(
        'saldo depois da despesa',
        'accounts.get',
        { id: checking.id },
        S.AccountViewShape,
        (a) =>
          (a as { balance: number }).balance === 100_000 - 12_345
            ? null
            : 'saldo não bate com o lançamento',
      );
    }
    const income = await step<{ id: string }>(
      'criar receita',
      'transactions.create',
      {
        account_id: checking.id,
        date: addDays(T, -3),
        amount: 50_000,
        description: `SALARIO ${tag}`,
      },
      S.TxnViewShape,
    );
    if (income) {
      await step(
        'status em lote',
        'transactions.set_status',
        { ids: [income.id], status: 'planned' },
        S.BulkResultShape,
        (r) => ((r as { updated: number }).updated === 1 ? null : 'set_status não alterou'),
      );
      await step(
        'categoria em lote (tirar)',
        'transactions.set_category',
        { ids: [income.id], category_id: null },
        S.BulkResultShape,
      );
    }
    await step(
      'consultar com filtros',
      'transactions.query',
      { account_ids: [checking.id], text: tag, date_from: addDays(T, -30), date_to: T, limit: 50 },
      S.TxnPageShape,
      (p) =>
        (p as { total: number }).total >= 1 ? null : 'busca por texto não achou o lançamento',
    );
    await step(
      'consultar por categoria (com subcategorias)',
      'transactions.query',
      { category_ids: food ? [food.id] : [], limit: 5 },
      S.TxnPageShape,
    );
    await step(
      'consultar sem categoria, caixa',
      'transactions.query',
      { uncategorized: true, basis: 'cash', date_from: addDays(T, -30), date_to: T, limit: 5 },
      S.TxnPageShape,
    );
    await step('tags', 'transactions.list_tags', {}, [S.TagSummaryShape]);
  }
  if (checking && usd) {
    await step(
      'transferência entre moedas',
      'transactions.create_transfer',
      {
        from_account_id: checking.id,
        to_account_id: usd.id,
        date: addDays(T, -2),
        amount: 5_432,
        to_amount: 1_000,
        description: `Remessa ${tag}`,
      },
      S.TransferResultShape,
      (r) => {
        const t = r as { from: { amount: number }; to: { amount: number } };
        return t.from.amount === -5_432 && t.to.amount === 1_000
          ? null
          : 'pernas da transferência com valores errados';
      },
    );
    await expectError(
      'transferência entre moedas sem to_amount',
      'transactions.create_transfer',
      { from_account_id: checking.id, to_account_id: usd.id, date: T, amount: 100 },
      'invalid',
    );
  }
  if (expenseId) {
    const att = await step<{ id: string }>(
      'anexar comprovante',
      'transactions.add_attachment',
      {
        txn_id: expenseId,
        filename: 'nota.txt',
        mime: 'text/plain',
        content_base64: b64('nota fiscal de teste'),
      },
      S.AttachmentMetaShape,
    );
    await step('listar anexos', 'transactions.list_attachments', { txn_id: expenseId }, [
      S.AttachmentMetaShape,
    ]);
    if (att) {
      await step(
        'baixar anexo',
        'transactions.get_attachment',
        { id: att.id },
        S.AttachmentContentShape,
      );
      await step('apagar anexo', 'transactions.delete_attachment', { id: att.id }, null);
    }
  }

  // ── Cartão, faturas e parcelas ────────────────────────────────
  if (card && other) {
    await step(
      'compra no cartão',
      'transactions.create',
      {
        account_id: card.id,
        date: addDays(T, -20),
        amount: -30_000,
        description: `LOJA ${tag}`,
        category_id: other,
      },
      S.TxnViewShape,
      (t) =>
        (t as { statement_id: string | null }).statement_id ? null : 'compra no cartão sem fatura',
    );
    const plan = await step<{ id: string; installments: number }>(
      'compra parcelada',
      'cards.create_installments',
      {
        account_id: card.id,
        description: `TV ${tag}`,
        total_amount: 100_001,
        installments: 3,
        purchase_date: addDays(T, -1),
        category_id: other,
      },
      S.InstallmentPlanShape,
    );
    await step('parcelamentos', 'cards.list_installment_plans', { account_id: card.id }, [
      S.InstallmentPlanShape,
    ]);
    await step(
      'compromissos futuros',
      'cards.future_commitments',
      { account_id: card.id, months: 6 },
      [S.MonthAmountShape],
    );
    const statements =
      (await step<{ id: string; remaining: number; total: number }[]>(
        'faturas',
        'cards.list_statements',
        { account_id: card.id },
        [S.StatementViewShape],
      )) ?? [];
    const st = statements.find((s) => s.remaining > 0);
    if (st) {
      await step('detalhe da fatura', 'cards.get_statement', { id: st.id }, S.StatementDetailShape);
      await step(
        'total informado',
        'cards.set_informed_total',
        { id: st.id, informed_total: st.total + 100 },
        S.StatementViewShape,
        (v) =>
          (v as { informed_total: number }).informed_total === st.total + 100
            ? null
            : 'total informado não gravou',
      );
      if (checking)
        await step(
          'pagar fatura',
          'cards.pay_statement',
          { statement_id: st.id, from_account_id: checking.id, amount: 1_000, date: T },
          S.PayStatementResultShape,
        );
    }
    if (plan)
      await step(
        'apagar parcelamento',
        'cards.delete_installment_plan',
        { id: plan.id },
        S.BulkResultShape,
        (r) => ((r as { updated: number }).updated === 3 ? null : 'deveria apagar 3 parcelas'),
      );
  }

  // ── Recorrências ──────────────────────────────────────────────
  await step(
    'prévia de recorrência',
    'recurrences.preview',
    { spec: { freq: 'MONTHLY', by_month_day: 31 }, start_date: '2026-01-31', count: 3 },
    S.RecurrencePreviewShape,
    (p) => {
      const v = p as { human: string; dates: string[] };
      return v.human === 'todo mês no último dia' &&
        v.dates.join() === '2026-01-31,2026-02-28,2026-03-31'
        ? null
        : `prévia inesperada: ${v.human} ${v.dates.join()}`;
    },
  );
  let recId: string | undefined;
  if (checking && other) {
    const rec = await step<{ id: string }>(
      'criar recorrência',
      'recurrences.create',
      {
        description: `Assinatura ${tag}`,
        account_id: checking.id,
        amount: -3_990,
        kind: 'expense',
        category_id: other,
        spec: { freq: 'MONTHLY', by_month_day: 15 },
        start_date: T,
      },
      S.RecurrenceShape,
    );
    recId = rec?.id;
    if (recId) {
      await step(
        'pausar recorrência',
        'recurrences.update',
        { id: recId, active: false },
        S.RecurrenceShape,
        (r) =>
          (r as { active: boolean; next_dates: string[] }).active === false ? null : 'não pausou',
      );
      await step('listar recorrências', 'recurrences.list', {}, [S.RecurrenceShape]);
      await step(
        'apagar recorrência',
        'recurrences.delete',
        { id: recId, delete_planned: true },
        S.RecurrenceDeleteResultShape,
      );
    }
  }

  // ── Regras ────────────────────────────────────────────────────
  if (expenseId && other) {
    await step('sugerir regra', 'rules.suggest_from_txn', { txn_id: expenseId }, S.RuleInputShape);
    const input = {
      name: `Regra ${tag}`,
      priority: 5,
      conditions: [{ field: 'description', op: 'contains', value: tag }],
      actions: { category_id: other, tags: ['regra'] },
    };
    await step('testar regra', 'rules.test', { rule: input, limit: 10 }, [S.TxnViewShape], (l) =>
      (l as unknown[]).length >= 1 ? null : 'teste da regra não casou',
    );
    const rule = await step<{ id: string }>('criar regra', 'rules.create', input, S.RuleShape);
    await step('listar regras', 'rules.list', {}, [S.RuleShape]);
    await step(
      'simular regras',
      'rules.apply',
      { only_uncategorized: false, dry_run: true, ...(rule ? { rule_ids: [rule.id] } : {}) },
      S.RuleApplyResultShape,
    );
    if (rule) {
      const applied = await step<{ change_set_id: string | null }>(
        'aplicar regras',
        'rules.apply',
        { only_uncategorized: false, rule_ids: [rule.id] },
        S.RuleApplyResultShape,
      );
      if (applied?.change_set_id)
        await step(
          'desfazer aplicação',
          'changes.undo',
          { id: applied.change_set_id },
          S.ChangeSetSummaryShape,
        );
      await step(
        'alterar regra',
        'rules.update',
        { id: rule.id, ...input, active: false },
        S.RuleShape,
      );
      await step('apagar regra', 'rules.delete', { id: rule.id }, null);
    }
    await expectError(
      'regra sem ação',
      'rules.create',
      {
        name: 'x',
        conditions: [{ field: 'description', op: 'contains', value: 'x' }],
        actions: {},
      },
      'invalid',
    );
  }

  // ── Importação ────────────────────────────────────────────────
  await step('modelos de CSV', 'import.list_presets', {}, [S.CsvPresetShape]);
  if (checking) {
    const d = (n: number) => addDays(T, -n).replace(/-/g, '');
    const ofx = [
      'OFXHEADER:100', 'DATA:OFXSGML', 'VERSION:102', 'CHARSET:1252', '', '<OFX><BANKMSGSRSV1><STMTTRNRS><STMTRS><CURDEF>BRL',
      '<BANKACCTFROM><BANKID>0077<BRANCHID>0001<ACCTID>12345-6<ACCTTYPE>CHECKING</BANKACCTFROM>',
      `<BANKTRANLIST><DTSTART>${d(10)}<DTEND>${d(1)}`,
      `<STMTTRN><TRNTYPE>DEBIT<DTPOSTED>${d(9)}120000<TRNAMT>-45.90<FITID>${tag}-1<MEMO>PADARIA ${tag.toUpperCase()}</STMTTRN>`,
      `<STMTTRN><TRNTYPE>DEBIT<DTPOSTED>${d(8)}120000<TRNAMT>-120.00<FITID>${tag}-2<MEMO>FARMACIA ${tag.toUpperCase()}</STMTTRN>`,
      `<STMTTRN><TRNTYPE>CREDIT<DTPOSTED>${d(7)}120000<TRNAMT>3000.00<FITID>${tag}-3<MEMO>PIX RECEBIDO ${tag.toUpperCase()}</STMTTRN>`,
      `<STMTTRN><TRNTYPE>DEBIT<DTPOSTED>${d(6)}120000<TRNAMT>-500.00<FITID>${tag}-4<MEMO>PAGAMENTO FATURA CARTAO</STMTTRN>`,
      '</BANKTRANLIST><LEDGERBAL><BALAMT>1234.56<DTASOF>' + d(1) + '</LEDGERBAL></STMTRS></STMTTRNRS></BANKMSGSRSV1></OFX>',
    ].join('\n'); // prettier-ignore
    const content = b64(ofx);
    await step(
      'detectar OFX',
      'import.detect',
      { filename: `extrato-${tag}.ofx`, content_base64: content },
      S.DetectResultShape,
      (r) => {
        const v = r as { format: string; row_count: number; ofx: unknown };
        return v.format === 'ofx' &&
          v.row_count === 4 &&
          !S.check(v.ofx, S.OfxAccountInfoShape).some((f) => f.level === 'erro')
          ? null
          : 'detecção do OFX inesperada';
      },
    );
    const preview = await step<{
      token: string;
      counts: { new: number };
      rows: { index: number; kind: string | null }[];
    }>(
      'prévia OFX',
      'import.preview',
      { account_id: checking.id, filename: `extrato-${tag}.ofx`, content_base64: content },
      S.ImportPreviewShape,
      (p) =>
        p.counts.new === 4 && p.rows[3]?.kind === 'card_payment'
          ? null
          : `prévia: ${p.counts.new} novas, 4ª linha ${p.rows[3]?.kind}`,
    );
    if (preview) {
      const res = await step<{ inserted: number; change_set_id: string | null; batch_id: string }>(
        'gravar importação',
        'import.commit',
        {
          token: preview.token,
          skip_indexes: [3],
          overrides: [{ index: 0, description: `Padaria ${tag}` }],
        },
        S.ImportResultShape,
        (r) => (r.inserted === 3 ? null : `esperava 3 inseridos, veio ${r.inserted}`),
      );
      await step('lotes', 'import.list_batches', { account_id: checking.id, limit: 5 }, [
        S.ImportBatchShape,
      ]);
      const again = await step<{ counts: { duplicate: number }; already_imported: boolean }>(
        'reimportar (duplicados)',
        'import.preview',
        { account_id: checking.id, filename: `extrato-${tag}.ofx`, content_base64: content },
        S.ImportPreviewShape,
        (p) =>
          p.counts.duplicate === 3 && p.already_imported
            ? null
            : `reimportação: ${p.counts.duplicate} duplicados`,
      );
      void again;
      if (res?.change_set_id) {
        await step(
          'detalhe da importação no histórico',
          'changes.get',
          { id: res.change_set_id },
          S.ChangeSetDetailShape,
        );
        await step(
          'desfazer importação',
          'changes.undo',
          { id: res.change_set_id },
          S.ChangeSetSummaryShape,
          (c) => ((c as { status: string }).status === 'undone' ? null : 'não ficou desfeita'),
        );
        await step(
          'lançamentos do lote somem',
          'transactions.query',
          { import_id: res.batch_id },
          S.TxnPageShape,
          (p) =>
            (p as { total: number }).total === 0 ? null : 'desfazer não apagou os importados',
        );
        await step(
          'refazer importação',
          'changes.redo',
          { id: res.change_set_id },
          S.ChangeSetSummaryShape,
        );
      }
    }
    const csv = `Data;Descrição;Valor\n${addDays(T, -4).split('-').reverse().join('/')};Café ${tag};-12,50\n${addDays(T, -3).split('-').reverse().join('/')};Livraria ${tag};-89,90\n`;
    const csvB64 = b64(csv);
    const det = await step<{ suggested_mapping: unknown }>(
      'detectar CSV',
      'import.detect',
      { filename: `extrato-${tag}.csv`, content_base64: csvB64 },
      S.DetectResultShape,
    );
    void det;
    await step(
      'prévia CSV com mapeamento',
      'import.preview',
      {
        account_id: checking.id,
        filename: `extrato-${tag}.csv`,
        content_base64: csvB64,
        format: 'csv',
        csv_mapping: {
          delimiter: ';',
          has_header: true,
          date_column: 'Data',
          date_format: '%d/%m/%Y',
          amount_column: 'Valor',
          description_column: 'Descrição',
          decimal_comma: true,
          invert_sign: false,
        },
      },
      S.ImportPreviewShape,
      (p) => ((p as { counts: { new: number } }).counts.new === 2 ? null : 'CSV não leu 2 linhas'),
    );
  }

  // ── Histórico ─────────────────────────────────────────────────
  const changes =
    (await step<{ id: string }[]>('histórico', 'changes.list', { limit: 10 }, [
      S.ChangeSetSummaryShape,
    ])) ?? [];
  await step('histórico por autor', 'changes.list', { limit: 5, author_kind: 'import' }, [
    S.ChangeSetSummaryShape,
  ]);
  if (changes[0])
    await step(
      'detalhe do histórico',
      'changes.get',
      { id: changes[0].id },
      S.ChangeSetDetailShape,
    );
  if (expenseId) {
    // Conflito: desfazer a criação de um lançamento depois de editá-lo.
    const own = (await call('changes.list', {
      table_name: 'txn',
      row_id: expenseId,
      limit: 10,
    })) as { id: string; reason: string }[];
    const creation = own.find((c) => c.reason.startsWith('Lançamento criado'));
    if (creation)
      await expectError('desfazer com conflito', 'changes.undo', { id: creation.id }, 'conflict');
  }

  // ── Conciliação ───────────────────────────────────────────────
  if (checking) {
    const now = (await call('accounts.get', { id: checking.id })) as { balance: number };
    await step(
      'conciliar (diferença)',
      'accounts.reconcile',
      { account_id: checking.id, statement_date: T, statement_balance: now.balance + 1 },
      S.ReconcileResultShape,
      (r) => ((r as { difference: number }).difference === 1 ? null : 'diferença deveria ser 1'),
    );
    const rec = await step<{
      marked_reconciled: number;
      change_set_id: string | null;
      difference: number;
      computed_balance: number;
    }>(
      'conciliar e marcar',
      'accounts.reconcile',
      { account_id: checking.id, statement_date: T, statement_balance: now.balance, apply: true },
      S.ReconcileResultShape,
    );
    if (rec && rec.difference !== 0)
      results.push({
        step: 'conciliar e marcar',
        cmd: 'accounts.reconcile',
        ok: false,
        findings: [
          {
            level: 'erro',
            path: '$',
            message: `diferença ${rec.difference} (saldo calculado ${rec.computed_balance})`,
          },
        ],
      });
    if (expenseId)
      await expectError(
        'alterar conciliado',
        'transactions.update',
        { id: expenseId, amount: -1 },
        'conflict',
      );
    await step(
      'histórico de saldo',
      'accounts.balance_history',
      { id: checking.id, from: addDays(T, -30), to: T, interval: 'day' },
      [S.BalancePointShape],
      (l) =>
        (l as unknown[]).length === 31
          ? null
          : `esperava 31 pontos, veio ${(l as unknown[]).length}`,
    );
    await step(
      'histórico mensal com previstos',
      'accounts.balance_history',
      {
        id: checking.id,
        from: addDays(T, -90),
        to: addDays(T, 30),
        interval: 'month',
        include_planned: true,
      },
      [S.BalancePointShape],
    );
  }

  // ── Ajustes ───────────────────────────────────────────────────
  const settings = await step<{ budget_basis: string }>(
    'ajustes',
    'settings.get',
    {},
    S.AppSettingsShape,
  );
  if (settings) {
    await step(
      'mudar ajuste',
      'settings.set',
      { budget_basis: settings.budget_basis === 'cash' ? 'accrual' : 'cash' },
      S.AppSettingsShape,
    );
    await step(
      'voltar ajuste',
      'settings.set',
      { budget_basis: settings.budget_basis },
      S.AppSettingsShape,
    );
  }
  await expectError('ajuste desconhecido', 'settings.set', { tema: 'x' }, 'invalid');
  await step('informações do app', 'app.info', {}, S.AppInfoShape);
  await step('saúde dos backups', 'backup.health', {}, S.BackupHealthShape);
  const bset = await step<Record<string, unknown>>(
    'ajustes de backup',
    'backup.get_settings',
    {},
    S.BackupSettingsShape,
  );
  if (bset)
    await step('salvar ajustes de backup', 'backup.set_settings', bset, S.BackupSettingsShape);
  await expectError(
    'senha de backup curta',
    'backup.set_password',
    { password: 'curta' },
    'invalid',
  );
  const backup = await step<{ id: string }>(
    'fazer backup',
    'backup.create',
    {},
    S.BackupRecordShape,
  );
  if (backup)
    await step('verificar backup', 'backup.verify', { id: backup.id }, S.BackupRecordShape, (b) =>
      (b as { verified: boolean }).verified ? null : 'backup recém-feito não verificou',
    );
  await step('lista de backups', 'backup.list', {}, [S.BackupRecordShape]);
  await step('configuração MCP', 'mcp.client_config', {}, S.McpConfigShape);
  await step('auditoria MCP', 'mcp.audit_list', { limit: 5 }, [S.AuditEntryShape]);
  await step('fontes de dados', 'market.data_sources', {}, S.DataSourcesShape);
  await expectError('chave da FRED inválida', 'market.set_fred_key', { key: 'curta' }, 'invalid');
  if (opts.sync) await step('atualizar dados', 'market.sync', {}, null);
  await step(
    'exportar JSON',
    'export.json',
    { path: `${opts.exportDir}/treasury-${tag}.json` },
    S.ExportResultShape,
  );
  await step(
    'exportar CSV',
    'export.transactions_csv',
    { path: `${opts.exportDir}/lancamentos-${tag}.csv`, filter: { text: tag } },
    S.ExportResultShape,
  );
  await expectError(
    'exportar caminho relativo',
    'export.json',
    { path: 'relativo.json' },
    'invalid',
  );

  // ── Cofre ─────────────────────────────────────────────────────
  await expectError(
    'kit com senha errada',
    'vault.recovery_kit',
    { password: 'senha errada demais' },
    'wrong_password',
  );
  await step(
    'kit de recuperação',
    'vault.recovery_kit',
    { password: opts.password },
    S.RecoveryKitShape,
  );
  const temp = `${opts.password} nova`;
  await expectError(
    'nova senha fraca',
    'vault.change_password',
    { current: opts.password, new: 'curta' },
    'weak_password',
  );
  const changed = await step(
    'trocar senha mestra',
    'vault.change_password',
    { current: opts.password, new: temp },
    S.VaultStatusShape,
  );
  if (changed)
    await step(
      'voltar a senha mestra',
      'vault.change_password',
      { current: temp, new: opts.password },
      S.VaultStatusShape,
    );
  const status = (await call('vault.status', {})) as { keychain_available: boolean };
  if (status.keychain_available) {
    await step(
      'ligar keychain',
      'vault.keychain_enable',
      { password: opts.password },
      S.VaultStatusShape,
      (s) => ((s as { keychain_enabled: boolean }).keychain_enabled ? null : 'keychain não ligou'),
    );
    await step('desligar keychain', 'vault.keychain_disable', {}, S.VaultStatusShape);
  }

  // ── Limpeza e restauração ─────────────────────────────────────
  if (rate) await step('remover câmbio', 'fx.delete', { id: rate.id }, null);
  if (sub && other)
    await step(
      'apagar categoria com reatribuição',
      'categories.delete',
      { id: sub.id, reassign_to: other },
      S.CategoryUsageShape,
    );
  if (opts.restore && backup) {
    await expectError(
      'restaurar com senha errada',
      'backup.restore',
      { id: backup.id, master_password: 'senha errada demais' },
      'wrong_password',
    );
    await step(
      'restaurar backup',
      'backup.restore',
      { id: backup.id, master_password: opts.password },
      S.VaultStatusShape,
      (s) => ((s as { state: string }).state === 'unlocked' ? null : 'cofre não voltou aberto'),
    );
  }
  return results;
}
