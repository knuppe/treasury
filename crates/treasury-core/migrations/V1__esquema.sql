-- Esquema do cofre (docs/development.md, "Migrations"): as 94 migrações de antes da 1.0.0
-- (V1 a V164) juntadas numa só, com o esquema e os dados iniciais exatamente como a cadeia
-- antiga os deixava. Um cofre gravado por ela passa, ao abrir, a ter só esta V1 no histórico
-- (storage::rebaseline_legacy). Migração nova = arquivo novo, a partir da V2.

-- Tabelas
CREATE TABLE setting (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,              -- JSON
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE change_set (
    id             TEXT PRIMARY KEY,
    created_at     TEXT NOT NULL,
    author_kind    TEXT NOT NULL CHECK (author_kind IN ('user', 'import', 'ai', 'system')),
    author_detail  TEXT,                    -- motor/modelo, cliente MCP, lote de importação
    reason         TEXT NOT NULL,
    status         TEXT NOT NULL CHECK (status IN ('pending', 'applied', 'undone', 'rejected')),
    applied_at     TEXT,
    undone_at      TEXT,
    changeset      BLOB,                    -- changeset da extensão session do SQLite
    summary        TEXT NOT NULL DEFAULT '{}'  -- JSON: {"insert": {"txn": 3}, ...}
) STRICT;

CREATE TABLE change_item (
    change_set_id  TEXT NOT NULL REFERENCES change_set (id) ON DELETE CASCADE,
    seq            INTEGER NOT NULL,
    table_name     TEXT NOT NULL,
    row_id         TEXT NOT NULL,
    op             TEXT NOT NULL CHECK (op IN ('insert', 'update', 'delete')),
    before         TEXT,                    -- JSON (linha anterior ou colunas alteradas)
    after          TEXT,                    -- JSON
    PRIMARY KEY (change_set_id, seq)
) STRICT;

CREATE TABLE backup_record (
    id             TEXT PRIMARY KEY,
    created_at     TEXT NOT NULL,
    kind           TEXT NOT NULL CHECK (kind IN ('auto_daily', 'manual', 'pre_migration', 'pre_import',
                                                 'pre_ai', 'pre_restore', 'pre_rekey', 'on_close')),
    path           TEXT NOT NULL,
    destination    TEXT NOT NULL DEFAULT 'primary',
    key_mode       TEXT NOT NULL CHECK (key_mode IN ('same', 'separate')),
    size_bytes     INTEGER NOT NULL DEFAULT 0,
    sha256         TEXT,
    verified       INTEGER NOT NULL DEFAULT 0 CHECK (verified IN (0, 1)),
    verify_error   TEXT,
    deleted_at     TEXT
) STRICT;

CREATE TABLE household (
    id             TEXT PRIMARY KEY,
    name           TEXT NOT NULL,
    base_currency  TEXT NOT NULL DEFAULT 'BRL',
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
, country TEXT NOT NULL DEFAULT 'BR'
    CHECK (length(country) = 2 AND country = upper(country))) STRICT;

CREATE TABLE member (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    color       TEXT,
    is_primary  INTEGER NOT NULL DEFAULT 0 CHECK (is_primary IN (0, 1)),
    archived    INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
, birth_date TEXT, relationship TEXT, irpf_dependent INTEGER NOT NULL DEFAULT 0 CHECK (irpf_dependent IN (0, 1)), occupation TEXT, notes TEXT, sex TEXT CHECK (sex IN ('male', 'female', 'unspecified'))) STRICT;

CREATE TABLE institution (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    kind        TEXT NOT NULL DEFAULT 'bank' CHECK (kind IN ('bank', 'broker', 'issuer', 'other')),
    code        TEXT,                       -- COMPE/ISPB, opcional
    website     TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE account_member (
    account_id  TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    member_id   TEXT NOT NULL REFERENCES member (id),
    PRIMARY KEY (account_id, member_id)
) STRICT;

CREATE TABLE credit_card (
    account_id          TEXT PRIMARY KEY REFERENCES account (id) ON DELETE CASCADE,
    credit_limit        INTEGER,
    closing_day         INTEGER NOT NULL CHECK (closing_day BETWEEN 1 AND 31),
    due_day             INTEGER NOT NULL CHECK (due_day BETWEEN 1 AND 31),
    payment_account_id  TEXT REFERENCES account (id),
    network             TEXT,
    last4               TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
) STRICT;

CREATE TABLE card_statement (
    id                   TEXT PRIMARY KEY,
    account_id           TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    reference_month      TEXT NOT NULL,
    period_start         TEXT NOT NULL,     -- primeiro dia de compras desta fatura
    closing_date         TEXT NOT NULL,
    due_date             TEXT NOT NULL,
    status               TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed', 'paid', 'partial')),
    informed_total       INTEGER,           -- total informado pelo banco (para conferência)
    created_at           TEXT NOT NULL,
    updated_at           TEXT NOT NULL, dates_source TEXT NOT NULL DEFAULT 'calc'
    CHECK (dates_source IN ('calc', 'manual', 'import')), informed_minimum INTEGER, informed_limit INTEGER, informed_previous_balance INTEGER,
    UNIQUE (account_id, reference_month)
) STRICT;

CREATE TABLE category (
    id            TEXT PRIMARY KEY,
    parent_id     TEXT REFERENCES category (id),
    name          TEXT NOT NULL,
    kind          TEXT NOT NULL CHECK (kind IN ('income', 'expense')),
    icon          TEXT,
    color         TEXT,
    is_essential  INTEGER NOT NULL DEFAULT 0 CHECK (is_essential IN (0, 1)),  -- base da reserva de emergência
    archived      INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    sort_order    INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
, seed_key TEXT, spend_group TEXT
    CHECK (spend_group IN ('essential', 'debt', 'future', 'non_essential'))) STRICT;

CREATE TABLE recurrence (
    id             TEXT PRIMARY KEY,
    description    TEXT NOT NULL,
    account_id     TEXT NOT NULL REFERENCES account (id),
    amount         INTEGER NOT NULL,          -- com sinal, como o lançamento
    currency       TEXT NOT NULL,
    kind           TEXT NOT NULL CHECK (kind IN ('income', 'expense', 'transfer', 'investment')),
    category_id    TEXT REFERENCES category (id),
    member_id      TEXT REFERENCES member (id),
    to_account_id  TEXT REFERENCES account (id),  -- para transferências recorrentes
    rrule          TEXT NOT NULL,             -- ex.: FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=5
    start_date     TEXT NOT NULL,
    end_date       TEXT,
    generated_until TEXT,                     -- até onde os previstos já foram gerados
    active         INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
) STRICT;

CREATE TABLE installment_plan (
    id              TEXT PRIMARY KEY,
    account_id      TEXT NOT NULL REFERENCES account (id),
    description     TEXT NOT NULL,
    total_amount    INTEGER NOT NULL,         -- positivo (valor da compra)
    currency        TEXT NOT NULL,
    installments    INTEGER NOT NULL CHECK (installments BETWEEN 1 AND 120),
    purchase_date   TEXT NOT NULL,
    first_due_month TEXT,                     -- fatura da 1ª parcela, se diferente da calculada
    category_id     TEXT REFERENCES category (id),
    member_id       TEXT REFERENCES member (id),
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
) STRICT;

CREATE TABLE txn (
    id                   TEXT PRIMARY KEY,
    account_id           TEXT NOT NULL REFERENCES account (id),
    date                 TEXT NOT NULL,       -- competência (data da compra no cartão)
    amount               INTEGER NOT NULL,
    currency             TEXT NOT NULL,
    kind                 TEXT NOT NULL CHECK (kind IN ('income', 'expense', 'transfer', 'card_payment',
                                                       'investment', 'adjustment')),
    status               TEXT NOT NULL DEFAULT 'cleared' CHECK (status IN ('planned', 'cleared', 'reconciled')),
    description          TEXT NOT NULL DEFAULT '',
    payee                TEXT,
    notes                TEXT,
    category_id          TEXT REFERENCES category (id),
    member_id            TEXT REFERENCES member (id),
    transfer_id          TEXT,                -- mesmo valor nas duas pernas da transferência
    statement_id         TEXT REFERENCES card_statement (id),
    installment_plan_id  TEXT REFERENCES installment_plan (id),
    installment_number   INTEGER,
    recurrence_id        TEXT REFERENCES recurrence (id),
    import_id            TEXT REFERENCES import_batch (id),
    external_id          TEXT,                -- ex.: FITID do OFX
    fingerprint          TEXT,                -- hash(data, valor, descrição normalizada) p/ deduplicação
    created_at           TEXT NOT NULL,
    updated_at           TEXT NOT NULL
, project_id TEXT REFERENCES project (id) ON DELETE SET NULL, member_origin TEXT CHECK (member_origin IN ('auto', 'manual')), card_plastic_id TEXT REFERENCES card_plastic (id), orig_amount INTEGER, orig_currency TEXT
    CHECK (orig_currency IS NULL OR (length(orig_currency) = 3 AND orig_currency = upper(orig_currency)))) STRICT;

CREATE TABLE txn_split (
    id           TEXT PRIMARY KEY,
    txn_id       TEXT NOT NULL REFERENCES txn (id) ON DELETE CASCADE,
    category_id  TEXT REFERENCES category (id),
    member_id    TEXT REFERENCES member (id),
    amount       INTEGER NOT NULL,
    notes        TEXT,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
) STRICT;

CREATE TABLE tag (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    color       TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE txn_tag (
    txn_id  TEXT NOT NULL REFERENCES txn (id) ON DELETE CASCADE,
    tag_id  TEXT NOT NULL REFERENCES tag (id) ON DELETE CASCADE,
    PRIMARY KEY (txn_id, tag_id)
) STRICT;

CREATE TABLE attachment (
    id          TEXT PRIMARY KEY,
    txn_id      TEXT REFERENCES txn (id) ON DELETE CASCADE,
    filename    TEXT NOT NULL,
    mime        TEXT NOT NULL,
    size_bytes  INTEGER NOT NULL,
    sha256      TEXT NOT NULL,
    data        BLOB NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE rule (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    priority     INTEGER NOT NULL DEFAULT 100,
    active       INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    conditions   TEXT NOT NULL,
    actions      TEXT NOT NULL,
    hits         INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
) STRICT;

CREATE TABLE budget (
    id          TEXT PRIMARY KEY,
    month       TEXT NOT NULL UNIQUE,         -- YYYY-MM
    notes       TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE budget_line (
    id           TEXT PRIMARY KEY,
    budget_id    TEXT NOT NULL REFERENCES budget (id) ON DELETE CASCADE,
    category_id  TEXT NOT NULL REFERENCES category (id),
    amount       INTEGER NOT NULL,            -- positivo: limite de gasto (ou meta de receita)
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    UNIQUE (budget_id, category_id)
) STRICT;

CREATE TABLE goal (
    id                    TEXT PRIMARY KEY,
    name                  TEXT NOT NULL,
    kind                  TEXT NOT NULL DEFAULT 'other' CHECK (kind IN ('emergency', 'travel', 'debt_payoff',
                                                                         'purchase', 'retirement', 'education', 'other')),
    target_amount         INTEGER NOT NULL,
    currency              TEXT NOT NULL DEFAULT 'BRL',
    target_date           TEXT,
    initial_amount        INTEGER NOT NULL DEFAULT 0,  -- valor já guardado fora das contas vinculadas
    monthly_contribution  INTEGER,                     -- aporte mensal planejado
    archived              INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    notes                 TEXT,
    created_at            TEXT NOT NULL,
    updated_at            TEXT NOT NULL
, real_return REAL CHECK (real_return IS NULL OR (real_return >= -0.05 AND real_return <= 0.15))) STRICT;

CREATE TABLE goal_account (
    goal_id     TEXT NOT NULL REFERENCES goal (id) ON DELETE CASCADE,
    account_id  TEXT NOT NULL REFERENCES account (id),
    PRIMARY KEY (goal_id, account_id)
) STRICT;

CREATE TABLE fx_rate (
    id          TEXT PRIMARY KEY,
    base        TEXT NOT NULL,                -- ex.: USD
    quote       TEXT NOT NULL,                -- ex.: BRL
    date        TEXT NOT NULL,
    rate        TEXT NOT NULL,                -- decimal exato: 1 base = rate quote
    source      TEXT NOT NULL,                -- 'manual', 'bcb_ptax_venda', ...
    fetched_at  TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    UNIQUE (base, quote, date, source)
) STRICT;

CREATE TABLE rule_hit (
    rule_id      TEXT PRIMARY KEY,
    hits         INTEGER NOT NULL DEFAULT 0,
    last_hit_at  TEXT
) STRICT;

CREATE TABLE ai_audit_log (
    id           TEXT PRIMARY KEY,
    created_at   TEXT NOT NULL,                 -- UTC RFC 3339
    client       TEXT NOT NULL,                 -- cliente MCP (ex.: "claude-code via treasury mcp")
    tool         TEXT NOT NULL,                 -- ferramenta ou "resources/read"
    params       TEXT NOT NULL DEFAULT '{}',    -- JSON resumido (textos e listas cortados)
    ok           INTEGER NOT NULL CHECK (ok IN (0, 1)),
    error        TEXT,                          -- "código: mensagem" quando ok = 0
    duration_ms  INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE TABLE index_series (
    id               TEXT PRIMARY KEY,      -- código estável: ipca, igpm, cdi, selic, selic_meta, tr, m2_br, us_cpi, us_m2, fed_funds
    name             TEXT NOT NULL,
    kind             TEXT NOT NULL CHECK (kind IN ('monthly_rate', 'daily_rate', 'level', 'annual_rate')),
    unit             TEXT NOT NULL,         -- '% a.m.', '% a.d.', 'índice', 'US$ bi'
    frequency        TEXT NOT NULL CHECK (frequency IN ('daily', 'monthly')),
    country          TEXT NOT NULL,         -- BR, US
    source           TEXT NOT NULL,         -- bcb_sgs:433, fred:CPIAUCSL, ibge_sidra:1737
    last_fetched_at  TEXT,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
) STRICT;

CREATE TABLE index_value (
    series_id   TEXT NOT NULL REFERENCES index_series (id) ON DELETE CASCADE,
    date        TEXT NOT NULL,
    value       TEXT NOT NULL,              -- decimal exato
    estimated   INTEGER NOT NULL DEFAULT 0 CHECK (estimated IN (0, 1)),  -- projeção Focus / último dado repetido
    source      TEXT NOT NULL,
    fetched_at  TEXT NOT NULL,
    PRIMARY KEY (series_id, date)
) STRICT;

CREATE TABLE holiday (
    date        TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    is_holiday  INTEGER NOT NULL DEFAULT 1 CHECK (is_holiday IN (0, 1)),  -- 0 = dia útil apesar do cálculo
    source      TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE trade (
    id            TEXT PRIMARY KEY,
    asset_id      TEXT NOT NULL REFERENCES asset (id),
    account_id    TEXT NOT NULL REFERENCES account (id),
    date          TEXT NOT NULL,
    side          TEXT NOT NULL CHECK (side IN ('buy', 'sell')),
    quantity      TEXT NOT NULL,            -- decimal (frações, cripto)
    price         TEXT NOT NULL,            -- preço unitário na moeda do ativo, decimal
    fees          INTEGER NOT NULL DEFAULT 0,  -- corretagem + emolumentos (menor unidade da moeda do ativo)
    taxes         INTEGER NOT NULL DEFAULT 0,  -- IRRF "dedo-duro" e outros impostos retidos
    currency      TEXT NOT NULL,
    fx_rate       TEXT,                     -- câmbio do dia: 1 moeda do ativo = fx_rate BRL
    day_trade     INTEGER NOT NULL DEFAULT 0 CHECK (day_trade IN (0, 1)),
    txn_id        TEXT REFERENCES txn (id), -- movimentação do caixa na conta da corretora
    note_number   TEXT,                     -- nota de corretagem
    import_id     TEXT REFERENCES import_batch (id),
    external_id   TEXT,
    notes         TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
) STRICT;

CREATE TABLE corporate_action (
    id            TEXT PRIMARY KEY,
    asset_id      TEXT NOT NULL REFERENCES asset (id),
    date          TEXT NOT NULL,            -- data "ex"/efetiva
    kind          TEXT NOT NULL CHECK (kind IN ('split', 'reverse_split', 'bonus', 'subscription', 'merger',
                                                'ticker_change', 'spin_off', 'capital_return')),
    factor        TEXT,                     -- nova qtd = antiga × factor (split/grupamento/bonificação)
    unit_cost     TEXT,                     -- custo atribuído por ação (bonificação) ou preço (subscrição)
    amount        INTEGER,                  -- devolução de capital (total, menor unidade)
    new_asset_id  TEXT REFERENCES asset (id),  -- mudança de ticker, incorporação, cisão
    notes         TEXT,
    source        TEXT NOT NULL DEFAULT 'manual',
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
) STRICT;

CREATE TABLE income_event (
    id            TEXT PRIMARY KEY,
    asset_id      TEXT NOT NULL REFERENCES asset (id),
    account_id    TEXT NOT NULL REFERENCES account (id),
    kind          TEXT NOT NULL CHECK (kind IN ('dividend', 'jcp', 'fii_income', 'coupon', 'interest', 'rent',
                                                'amortization', 'other')),
    ex_date       TEXT,
    pay_date      TEXT NOT NULL,
    quantity      TEXT,                     -- base (quantidade na data-com)
    gross         INTEGER NOT NULL,
    tax_withheld  INTEGER NOT NULL DEFAULT 0,
    net           INTEGER NOT NULL,
    currency      TEXT NOT NULL,
    fx_rate       TEXT,
    status        TEXT NOT NULL DEFAULT 'received' CHECK (status IN ('announced', 'received')),
    txn_id        TEXT REFERENCES txn (id),
    source        TEXT NOT NULL DEFAULT 'manual',
    import_id     TEXT REFERENCES import_batch (id),
    external_id   TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
) STRICT;

CREATE TABLE price (
    asset_id    TEXT NOT NULL REFERENCES asset (id) ON DELETE CASCADE,
    date        TEXT NOT NULL,
    open        TEXT,
    high        TEXT,
    low         TEXT,
    close       TEXT NOT NULL,
    adj_close   TEXT,                       -- ajustado por proventos (só para gráficos de retorno total)
    volume      TEXT,
    source      TEXT NOT NULL,
    fetched_at  TEXT NOT NULL,
    PRIMARY KEY (asset_id, date)
) STRICT;

CREATE TABLE target_allocation (
    id             TEXT PRIMARY KEY,
    scope          TEXT NOT NULL CHECK (scope IN ('class', 'asset')),
    key            TEXT NOT NULL,           -- classe (stock_br…) ou asset_id
    target_pct     TEXT NOT NULL,           -- decimal, % da carteira
    tolerance_pct  TEXT NOT NULL DEFAULT '5',
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL,
    UNIQUE (scope, key)
) STRICT;

CREATE TABLE watchlist_item (
    id            TEXT PRIMARY KEY,
    asset_id      TEXT NOT NULL REFERENCES asset (id) ON DELETE CASCADE UNIQUE,
    target_price  TEXT,                     -- meu preço-alvo
    alert_above   TEXT,
    alert_below   TEXT,
    notes         TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
) STRICT;

CREATE TABLE position_snapshot (
    asset_id    TEXT NOT NULL REFERENCES asset (id) ON DELETE CASCADE,
    account_id  TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    date        TEXT NOT NULL,
    quantity    TEXT NOT NULL,
    cost        INTEGER NOT NULL,           -- custo total (moeda do ativo)
    value       INTEGER,                    -- valor de mercado (moeda do ativo)
    PRIMARY KEY (asset_id, account_id, date)
) STRICT;

CREATE TABLE darf (
    id              TEXT PRIMARY KEY,
    member_id       TEXT REFERENCES member (id),
    period          TEXT NOT NULL,              -- mês de apuração YYYY-MM
    revenue_code    TEXT NOT NULL,              -- 6015: ganhos líquidos em bolsa (PF)
    principal       INTEGER NOT NULL,           -- centavos
    fine            INTEGER NOT NULL DEFAULT 0, -- multa de mora
    interest        INTEGER NOT NULL DEFAULT 0, -- juros (Selic)
    total           INTEGER NOT NULL,
    due_date        TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'paid', 'cancelled')),
    paid_at         TEXT,
    txn_id          TEXT REFERENCES txn (id),
    notes           TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
) STRICT;

CREATE TABLE research_issuer (
    id           TEXT PRIMARY KEY,          -- CNPJ só com dígitos (14) ou "cik:0000320193"
    kind         TEXT NOT NULL CHECK (kind IN ('company_br', 'fii', 'company_us')),
    name         TEXT NOT NULL,
    cvm_code     TEXT,
    sector       TEXT,                      -- FCA (Setor_Atividade) ou SEC
    segment      TEXT,                      -- FII: Segmento_Atuacao
    mandate      TEXT,                      -- FII: Mandato
    -- Leiaute das demonstrações: industrial (padrão), bank (instituição financeira), insurance.
    layout       TEXT NOT NULL DEFAULT 'industrial' CHECK (layout IN ('industrial', 'bank', 'insurance', 'fund')),
    currency     TEXT NOT NULL DEFAULT 'BRL',
    source       TEXT NOT NULL,
    fetched_at   TEXT NOT NULL
) STRICT;

CREATE TABLE research_security (
    ticker            TEXT PRIMARY KEY,
    issuer_id         TEXT NOT NULL REFERENCES research_issuer (id) ON DELETE CASCADE,
    class             TEXT NOT NULL,         -- stock_br, fii, stock_us, bdr…
    share_kind        TEXT,                  -- ON, PN, UNIT, CTF, COMMON
    unit_composition  TEXT,                  -- "1 ON / 2 PN"
    isin              TEXT,
    listing_segment   TEXT,                  -- Novo Mercado, Nível 2…
    active            INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    source            TEXT NOT NULL,
    fetched_at        TEXT NOT NULL
) STRICT;

CREATE TABLE fundamental (
    issuer_id     TEXT NOT NULL,
    item          TEXT NOT NULL,             -- revenue, net_income, equity, shares_outstanding…
    period_type   TEXT NOT NULL CHECK (period_type IN ('instant', 'annual', 'ytd', 'quarter', 'month', 'ltm')),
    period_end    TEXT NOT NULL,
    period_start  TEXT,
    value         TEXT NOT NULL,             -- decimal exato
    currency      TEXT NOT NULL,             -- BRL, USD; "shares" para quantidades de ações/cotas
    source        TEXT NOT NULL,             -- cvm_dfp, cvm_itr, cvm_fii, sec_companyfacts, calculado
    document      TEXT NOT NULL,             -- documento de origem (ex.: "DFP 2025 v1, consolidado")
    priority      INTEGER NOT NULL DEFAULT 0,  -- dado original (2) vence a reapresentação (1)
    fetched_at    TEXT NOT NULL,
    PRIMARY KEY (issuer_id, item, period_type, period_end)
) STRICT;

CREATE TABLE corporate_document (
    id              TEXT PRIMARY KEY,        -- protocolo de entrega na CVM
    issuer_id       TEXT NOT NULL,
    company_name    TEXT NOT NULL,
    category        TEXT NOT NULL,           -- Fato Relevante, Comunicado ao Mercado…
    doc_type        TEXT,
    species         TEXT,
    subject         TEXT,
    reference_date  TEXT,
    delivered_at    TEXT NOT NULL,
    version         TEXT,
    link            TEXT NOT NULL,
    summary         TEXT,                    -- resumo opcional do agente
    source          TEXT NOT NULL,
    fetched_at      TEXT NOT NULL
) STRICT;

CREATE TABLE research_quote (
    ticker      TEXT NOT NULL,
    date        TEXT NOT NULL,
    close       TEXT NOT NULL,
    volume      TEXT NOT NULL,               -- volume financeiro do dia (R$)
    trades      INTEGER,
    source      TEXT NOT NULL,
    fetched_at  TEXT NOT NULL,
    PRIMARY KEY (ticker, date)
) STRICT;

CREATE TABLE research_file (
    url            TEXT PRIMARY KEY,
    etag           TEXT,
    last_modified  TEXT,
    bytes          INTEGER,
    fetched_at     TEXT NOT NULL
) STRICT;

CREATE TABLE research_universe (
    ticker       TEXT PRIMARY KEY,
    asset_id     TEXT,
    issuer_id    TEXT,
    name         TEXT NOT NULL,
    class        TEXT NOT NULL,
    sector       TEXT,
    segment      TEXT,
    currency     TEXT NOT NULL,
    metrics      TEXT NOT NULL,              -- {"pl": 5.2, "dy": 0.08, …}
    as_of        TEXT NOT NULL,
    computed_at  TEXT NOT NULL
) STRICT;

CREATE TABLE screen (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    description  TEXT,
    query        TEXT NOT NULL,              -- JSON (ScreenQuery)
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
, alert INTEGER NOT NULL DEFAULT 0 CHECK (alert IN (0, 1))) STRICT;

CREATE TABLE ai_memory (
    id          TEXT PRIMARY KEY,
    kind        TEXT NOT NULL CHECK (kind IN ('profile', 'family_rule', 'note')),
    content     TEXT NOT NULL,
    created_by  TEXT NOT NULL CHECK (created_by IN ('user', 'ai')),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
, linked_at TEXT, used_count INTEGER NOT NULL DEFAULT 0) STRICT;

CREATE TABLE secret (
    id          TEXT PRIMARY KEY,              -- ex.: ai_engine:<id do motor>
    nonce       BLOB NOT NULL,                 -- 24 bytes aleatórios por valor
    ciphertext  BLOB NOT NULL,                 -- valor cifrado + tag Poly1305
    hint        TEXT NOT NULL,                 -- 4 últimos caracteres (para a interface)
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE ai_engine (
    id                 TEXT PRIMARY KEY,
    name               TEXT NOT NULL,
    kind               TEXT NOT NULL CHECK (kind IN ('api', 'cli', 'local')),
    provider           TEXT NOT NULL,          -- anthropic, openai, gemini, xai, mistral, openrouter,
                                               -- openai_compatible, claude_code, codex, gemini_cli,
                                               -- grok_cli, custom_cli, ollama, lm_studio
    model              TEXT,
    endpoint           TEXT,                   -- URL base (API e modelo local)
    command_template   TEXT,                   -- CLI: {prompt} {mcp_config} {model} {workdir}
    enabled            INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    max_output_tokens  INTEGER,
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL
, vision INTEGER CHECK (vision IN (0, 1))) STRICT;

CREATE TABLE ai_spend_limit (
    provider              TEXT PRIMARY KEY,
    daily_micros          INTEGER,
    monthly_micros        INTEGER,
    confirm_above_micros  INTEGER,              -- aviso antes de tarefas com custo estimado maior
    updated_at            TEXT NOT NULL
) STRICT;

CREATE TABLE ai_model_price (
    provider                  TEXT NOT NULL,
    model                     TEXT NOT NULL,
    input_micros_per_mtok     INTEGER NOT NULL CHECK (input_micros_per_mtok >= 0),
    output_micros_per_mtok    INTEGER NOT NULL CHECK (output_micros_per_mtok >= 0),
    cached_micros_per_mtok    INTEGER,
    source                    TEXT NOT NULL DEFAULT 'user' CHECK (source IN ('default', 'user')),
    updated_at                TEXT NOT NULL,
    PRIMARY KEY (provider, model)
) STRICT;

CREATE TABLE ai_usage (
    id             TEXT PRIMARY KEY,
    created_at     TEXT NOT NULL,
    engine_id      TEXT,                       -- sem FK: o uso continua depois de apagar o motor
    engine_name    TEXT NOT NULL,
    provider       TEXT NOT NULL,
    kind           TEXT NOT NULL CHECK (kind IN ('api', 'cli', 'local')),
    model          TEXT,
    task           TEXT NOT NULL,
    job_id         TEXT,
    input_tokens   INTEGER NOT NULL DEFAULT 0,
    output_tokens  INTEGER NOT NULL DEFAULT 0,
    cached_tokens  INTEGER NOT NULL DEFAULT 0,
    cost_micros    INTEGER NOT NULL DEFAULT 0, -- USD × 10⁻⁶; CLIs e modelos locais: 0
    priced         INTEGER NOT NULL DEFAULT 1 CHECK (priced IN (0, 1)),
    ok             INTEGER NOT NULL DEFAULT 1 CHECK (ok IN (0, 1)),
    error          TEXT
) STRICT;

CREATE TABLE ai_conversation (
    id              TEXT PRIMARY KEY,
    title           TEXT NOT NULL,
    task            TEXT NOT NULL DEFAULT 'chat',
    last_engine_id  TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
, pinned INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0, 1)), asset_id TEXT REFERENCES asset (id) ON DELETE SET NULL) STRICT;

CREATE TABLE ai_message (
    id               TEXT PRIMARY KEY,
    conversation_id  TEXT NOT NULL REFERENCES ai_conversation (id) ON DELETE CASCADE,
    seq              INTEGER NOT NULL,
    role             TEXT NOT NULL CHECK (role IN ('user', 'assistant')),
    content          TEXT NOT NULL,
    engine_id        TEXT,
    engine_name      TEXT,
    tool_calls       TEXT NOT NULL DEFAULT '[]',  -- JSON [{name, summary, ok}]
    proposal_ids     TEXT NOT NULL DEFAULT '[]',  -- JSON [id]
    created_at       TEXT NOT NULL, reasoning TEXT, reasoning_ms INTEGER,
    UNIQUE (conversation_id, seq)
) STRICT;

CREATE TABLE alert (
    id           TEXT PRIMARY KEY,
    kind         TEXT NOT NULL,                -- budget_over, budget_pace, due_soon, rebalance, monthly_close…
    severity     TEXT NOT NULL CHECK (severity IN ('info', 'warning', 'critical')),
    title        TEXT NOT NULL,
    body         TEXT NOT NULL DEFAULT '',
    status       TEXT NOT NULL DEFAULT 'new' CHECK (status IN ('new', 'seen', 'resolved')),
    source       TEXT NOT NULL,                -- rotina ou motor que gerou
    dedupe_key   TEXT,                         -- a mesma condição não gera dois alertas
    ref_table    TEXT,
    ref_id       TEXT,
    data         TEXT NOT NULL DEFAULT '{}',   -- JSON com os números usados
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    seen_at      TEXT,
    resolved_at  TEXT
, title_key TEXT, args TEXT) STRICT;

CREATE TABLE loan_installment (
    id                TEXT PRIMARY KEY,
    loan_id           TEXT NOT NULL REFERENCES loan (id) ON DELETE CASCADE,
    number            INTEGER NOT NULL,
    due_date          TEXT NOT NULL,
    amortization      INTEGER NOT NULL,
    interest          INTEGER NOT NULL,
    adjustment        INTEGER NOT NULL DEFAULT 0,   -- correção monetária do saldo no mês
    insurance         INTEGER NOT NULL DEFAULT 0,
    fees              INTEGER NOT NULL DEFAULT 0,
    total             INTEGER NOT NULL,
    balance_after     INTEGER NOT NULL,
    estimated         INTEGER NOT NULL DEFAULT 0 CHECK (estimated IN (0, 1)),  -- índice futuro projetado
    extra_payment     INTEGER NOT NULL DEFAULT 0,   -- amortização extraordinária no mês
    status            TEXT NOT NULL DEFAULT 'planned' CHECK (status IN ('planned', 'paid')),
    paid_date         TEXT,
    paid_amount       INTEGER,
    txn_id            TEXT REFERENCES txn (id),
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL, index_factor TEXT, business_days INTEGER, insurance_mip INTEGER NOT NULL DEFAULT 0, insurance_dfi INTEGER NOT NULL DEFAULT 0, source TEXT,
    UNIQUE (loan_id, number)
) STRICT;

CREATE TABLE loan_extra_payment (
    id          TEXT PRIMARY KEY,
    loan_id     TEXT NOT NULL REFERENCES loan (id) ON DELETE CASCADE,
    date        TEXT NOT NULL,
    amount      INTEGER NOT NULL,
    mode        TEXT NOT NULL CHECK (mode IN ('reduce_term', 'reduce_installment')),
    txn_id      TEXT REFERENCES txn (id),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE physical_asset (
    id                    TEXT PRIMARY KEY,
    kind                  TEXT NOT NULL CHECK (kind IN ('real_estate', 'vehicle', 'land', 'equipment',
                                                        'collectible', 'other')),
    name                  TEXT NOT NULL,
    purchase_date         TEXT NOT NULL,
    purchase_price        INTEGER NOT NULL CHECK (purchase_price >= 0),  -- menor unidade
    currency              TEXT NOT NULL DEFAULT 'BRL',
    -- Estimativa automática do valor atual (entre avaliações).
    valuation_method      TEXT NOT NULL DEFAULT 'none' CHECK (valuation_method IN ('none', 'index', 'declining',
                                                                                    'straight_line', 'fipe')),
    index_series          TEXT CHECK (index_series IN ('ipca', 'igpm', 'incc')),
    depreciation_rate     TEXT,                    -- % a.a. (decimal texto), depreciação composta
    useful_life_years     INTEGER CHECK (useful_life_years BETWEEN 1 AND 100),
    residual_pct          TEXT,                    -- % do custo ao fim da vida útil (linear)
    -- Tabela FIPE (parallelum.com.br/fipe/api/v1): tipo, marca, modelo e ano-combustível.
    fipe_vehicle_type     TEXT CHECK (fipe_vehicle_type IN ('carros', 'motos', 'caminhoes')),
    fipe_brand_code       TEXT,
    fipe_model_code       TEXT,
    fipe_year_code        TEXT,
    fipe_code             TEXT,                    -- código FIPE (ex.: 005340-6), conferido na consulta
    address               TEXT,
    description           TEXT,
    notes                 TEXT,
    include_in_net_worth  INTEGER NOT NULL DEFAULT 1 CHECK (include_in_net_worth IN (0, 1)),
    archived              INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    sale_date             TEXT,
    sale_price            INTEGER,
    sale_costs            INTEGER NOT NULL DEFAULT 0,  -- corretagem, impostos da venda
    created_at            TEXT NOT NULL,
    updated_at            TEXT NOT NULL
) STRICT;

CREATE TABLE physical_asset_member (
    asset_id   TEXT NOT NULL REFERENCES physical_asset (id) ON DELETE CASCADE,
    member_id  TEXT NOT NULL REFERENCES member (id),
    PRIMARY KEY (asset_id, member_id)
) STRICT;

CREATE TABLE physical_acquisition_cost (
    id           TEXT PRIMARY KEY,
    asset_id     TEXT NOT NULL REFERENCES physical_asset (id) ON DELETE CASCADE,
    kind         TEXT NOT NULL CHECK (kind IN ('itbi', 'deed', 'registry', 'brokerage', 'other')),
    description  TEXT,
    amount       INTEGER NOT NULL CHECK (amount >= 0),
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
) STRICT;

CREATE TABLE physical_improvement (
    id           TEXT PRIMARY KEY,
    asset_id     TEXT NOT NULL REFERENCES physical_asset (id) ON DELETE CASCADE,
    date         TEXT NOT NULL,
    description  TEXT NOT NULL,
    category     TEXT NOT NULL CHECK (category IN ('renovation', 'expansion', 'improvement', 'maintenance')),
    amount       INTEGER NOT NULL CHECK (amount > 0),
    txn_id       TEXT REFERENCES txn (id) ON DELETE SET NULL,
    notes        TEXT,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
, project_id TEXT REFERENCES project (id) ON DELETE SET NULL) STRICT;

CREATE TABLE physical_valuation (
    id          TEXT PRIMARY KEY,
    asset_id    TEXT NOT NULL REFERENCES physical_asset (id) ON DELETE CASCADE,
    date        TEXT NOT NULL,
    value       INTEGER NOT NULL CHECK (value >= 0),
    source      TEXT NOT NULL CHECK (source IN ('manual', 'appraisal', 'report', 'fipe', 'index')),
    notes       TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE physical_asset_loan (
    asset_id  TEXT NOT NULL REFERENCES physical_asset (id) ON DELETE CASCADE,
    loan_id   TEXT NOT NULL UNIQUE REFERENCES loan (id) ON DELETE CASCADE,
    PRIMARY KEY (asset_id, loan_id)
) STRICT;

CREATE TABLE country_profile (
    country      TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    currency     TEXT NOT NULL,
    fx_provider  TEXT NOT NULL,             -- bcb_ptax, ecb, boc, boe
    notes        TEXT,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
, cost_method TEXT NOT NULL DEFAULT 'average'
    CHECK (cost_method IN ('average', 'fifo', 'uk_pool'))) STRICT;

CREATE TABLE "asset" (
    id               TEXT PRIMARY KEY,
    ticker           TEXT,                  -- PETR4, HGLG11, AAPL, VOD, 7203, BTC; NULL para renda fixa sem código
    name             TEXT NOT NULL,
    class            TEXT NOT NULL CHECK (class IN ('stock_br', 'fii', 'etf_br', 'bdr', 'fixed_income', 'treasury_br',
                                                    'fund', 'stock_us', 'etf_us', 'reit', 'crypto', 'pension', 'other',
                                                    'stock_intl', 'etf_intl')),
    market           TEXT,                  -- código da bolsa (tabela `exchange`: B3, NYSE, LSE, TSE…) ou CRYPTO, TESOURO, CVM
    currency         TEXT NOT NULL,
    country          TEXT,
    isin             TEXT,
    cnpj             TEXT,                  -- fundos
    treasury_code    TEXT,                  -- Tesouro Direto (ex.: "Tesouro IPCA+ 2035")
    sector           TEXT,
    segment          TEXT,
    price_provider   TEXT,                  -- provider preferido (yahoo, tesouro, cvm, manual, none)
    provider_symbol  TEXT,                  -- símbolo no provider (PETR4.SA, VOD.L, BTC-USD…)
    archived         INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    notes            TEXT,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
, official_name TEXT, account_id TEXT REFERENCES account (id) ON DELETE SET NULL, concentration_limit_pct REAL) STRICT;

CREATE TABLE exchange (
    code          TEXT PRIMARY KEY,         -- B3, NYSE, LSE, TSE…
    name          TEXT NOT NULL,
    country       TEXT NOT NULL,            -- ISO 3166-1 alfa-2
    currency      TEXT NOT NULL,            -- moeda das cotações (ISO 4217; LSE cota em GBp, convertido para GBP)
    yahoo_suffix  TEXT NOT NULL DEFAULT '', -- ".SA", ".L"; vazio nos EUA
    mic           TEXT,                     -- ISO 10383 (BVMF, XNYS…)
    timezone      TEXT NOT NULL,            -- America/Sao_Paulo, Europe/London…
    open_time     TEXT NOT NULL,            -- HH:MM local
    close_time    TEXT NOT NULL,
    lunch_start   TEXT,                     -- intervalo de almoço (Tóquio, Hong Kong, Xangai…)
    lunch_end     TEXT,
    trading_days  TEXT NOT NULL DEFAULT '12345',
    aliases       TEXT NOT NULL DEFAULT '', -- outros nomes e códigos do Yahoo, separados por vírgula
    builtin       INTEGER NOT NULL DEFAULT 0 CHECK (builtin IN (0, 1)),
    notes         TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
) STRICT;

CREATE TABLE market_quote (
    asset_id        TEXT PRIMARY KEY REFERENCES asset (id) ON DELETE CASCADE,
    symbol          TEXT NOT NULL,
    price           TEXT NOT NULL,          -- decimal, na moeda do ativo
    previous_close  TEXT,
    day_high        TEXT,
    day_low         TEXT,
    volume          TEXT,
    currency        TEXT NOT NULL,
    market_time     TEXT NOT NULL,          -- instante da cotação (UTC RFC 3339)
    market_date     TEXT NOT NULL,          -- data do pregão no fuso da bolsa
    market_state    TEXT NOT NULL CHECK (market_state IN ('pre', 'open', 'lunch', 'post', 'closed')),
    timezone        TEXT,                   -- fuso informado pelo Yahoo
    delay_minutes   INTEGER NOT NULL DEFAULT 15,
    source          TEXT NOT NULL,
    fetched_at      TEXT NOT NULL
) STRICT;

CREATE TABLE intraday_price (
    asset_id  TEXT NOT NULL REFERENCES asset (id) ON DELETE CASCADE,
    ts        INTEGER NOT NULL,             -- início da barra (segundos Unix, UTC)
    price     TEXT NOT NULL,
    volume    TEXT,
    PRIMARY KEY (asset_id, ts)
) STRICT, WITHOUT ROWID;

CREATE TABLE mcp_api_key (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    prefix        TEXT NOT NULL,              -- início da chave, para reconhecer na lista
    key_hash      TEXT NOT NULL UNIQUE,       -- SHA-256 hex da chave inteira
    scopes        TEXT NOT NULL,              -- JSON: ["read", "write", "browser"]
    created_at    TEXT NOT NULL,
    expires_at    TEXT,
    last_used_at  TEXT,
    revoked_at    TEXT
) STRICT;

CREATE TABLE ai_procedure (
    id               TEXT PRIMARY KEY,
    topic            TEXT NOT NULL CHECK (topic IN ('statement', 'card_bill', 'investments', 'income',
                                                    'prices', 'tax', 'registration', 'other')),
    title            TEXT NOT NULL,
    steps            TEXT NOT NULL,
    site             TEXT,                  -- domínio, sem caminho nem parâmetros
    file_format      TEXT,                  -- OFX, CSV, PDF, XLSX…
    institution_id   TEXT,
    account_id       TEXT,
    asset_id         TEXT,
    created_by       TEXT NOT NULL CHECK (created_by IN ('ai', 'user')),
    client           TEXT,                  -- motor/cliente que gravou (ex.: "agente (Claude Code)")
    conversation_id  TEXT,                  -- conversa de origem (pode ter sido apagada)
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
) STRICT;

CREATE TABLE ai_procedure_use (
    procedure_id  TEXT PRIMARY KEY,
    worked        INTEGER NOT NULL DEFAULT 0,
    failed        INTEGER NOT NULL DEFAULT 0,
    last_used_at  TEXT,
    last_outcome  TEXT CHECK (last_outcome IN ('worked', 'failed')),
    last_note     TEXT
) STRICT;

CREATE TABLE "loan" (
    id                 TEXT PRIMARY KEY,
    name               TEXT NOT NULL,
    kind               TEXT NOT NULL DEFAULT 'personal' CHECK (kind IN ('mortgage', 'vehicle', 'personal', 'payroll',
                                                                         'student', 'other')),
    institution_id     TEXT REFERENCES institution (id),
    principal          INTEGER NOT NULL,            -- valor financiado (centavos)
    currency           TEXT NOT NULL DEFAULT 'BRL',
    annual_rate        TEXT NOT NULL,               -- pré: taxa efetiva % a.a.; pós: spread % a.a.
    -- `bullet`: só juros a cada vencimento e o principal no último.
    system             TEXT NOT NULL CHECK (system IN ('sac', 'price', 'bullet')),
    -- Já aceita os índices das próximas fases (INPC, INCC, poupança), sem nova reconstrução.
    indexer            TEXT NOT NULL DEFAULT 'none' CHECK (indexer IN ('none', 'tr', 'ipca', 'igpm', 'inpc', 'incc',
                                                                       'cdi', 'selic', 'poupanca')),
    -- Percentual do índice (100 = 100% do CDI). Só nos índices de taxa diária.
    index_pct          TEXT NOT NULL DEFAULT '100',
    -- Spread nos pós de taxa diária: exponencial base 252 ou linear base 360 (dias corridos).
    spread_mode        TEXT NOT NULL DEFAULT 'exp252' CHECK (spread_mode IN ('exp252', 'lin360')),
    -- Defasagem do índice em dias úteis (alguns bancos usam o DI de D-1 ou D-2).
    index_lag_days     INTEGER NOT NULL DEFAULT 0 CHECK (index_lag_days BETWEEN 0 AND 10),
    contract_date      TEXT NOT NULL,
    first_due_date     TEXT NOT NULL,
    term_months        INTEGER NOT NULL CHECK (term_months BETWEEN 1 AND 600),
    grace_months       INTEGER NOT NULL DEFAULT 0,  -- carência: paga só juros
    insurance_monthly  INTEGER NOT NULL DEFAULT 0,  -- seguros MIP + DFI (centavos/mês)
    fees_monthly       INTEGER NOT NULL DEFAULT 0,  -- taxa de administração etc.
    cet_annual         TEXT,                        -- CET informado no contrato (% a.a.)
    debit_account_id   TEXT REFERENCES account (id),
    member_id          TEXT REFERENCES member (id),
    archived           INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    notes              TEXT,
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL,
    contract_number    TEXT
, correction_lag_months INTEGER NOT NULL DEFAULT 1
    CHECK (correction_lag_months BETWEEN 0 AND 3), correction_prorata TEXT NOT NULL DEFAULT 'none'
    CHECK (correction_prorata IN ('none', 'calendar', 'business')), deflation_floor INTEGER NOT NULL DEFAULT 0 CHECK (deflation_floor IN (0, 1)), interest_basis TEXT NOT NULL DEFAULT 'monthly'
    CHECK (interest_basis IN ('monthly', 'bd252', 'dc365')), correction_reference TEXT NOT NULL DEFAULT 'month'
    CHECK (correction_reference IN ('month', 'anniversary')), category_id TEXT REFERENCES category (id) ON DELETE SET NULL, property_value INTEGER, mip_rate TEXT, dfi_rate TEXT, mip_amount INTEGER, dfi_amount INTEGER, rebase_number INTEGER, rebase_balance INTEGER, rebase_term INTEGER) STRICT;

CREATE TABLE ai_attachment (
    id               TEXT PRIMARY KEY,
    message_id       TEXT NOT NULL REFERENCES ai_message (id) ON DELETE CASCADE,
    conversation_id  TEXT NOT NULL REFERENCES ai_conversation (id) ON DELETE CASCADE,
    position         INTEGER NOT NULL,
    name             TEXT NOT NULL,
    mime             TEXT NOT NULL,
    kind             TEXT NOT NULL CHECK (kind IN ('image', 'pdf', 'text')),
    size             INTEGER NOT NULL,           -- bytes do arquivo como foi enviado
    sha256           TEXT NOT NULL,
    width            INTEGER,
    height           INTEGER,
    pages            INTEGER,
    data             BLOB,                       -- só com retenção "7 dias" ou "sempre"
    text             TEXT,                       -- idem (texto extraído)
    expires_at       TEXT,                       -- retenção "7 dias"
    created_at       TEXT NOT NULL
) STRICT;

CREATE TABLE account_overdraft (
    account_id           TEXT PRIMARY KEY REFERENCES account (id),
    limit_amount         INTEGER NOT NULL CHECK (limit_amount >= 0),  -- centavos
    monthly_rate         TEXT,     -- taxa de juros ao mês (%)
    discount_pct         TEXT,     -- desconto sobre a taxa (%)
    annual_rate          TEXT,     -- taxa de juros cobrada ao ano (%)
    iof_daily_rate       TEXT,     -- IOF alíquota básica, ao dia (%), ex.: 0,0082
    iof_additional_rate  TEXT,     -- IOF alíquota adicional, sobre o valor usado (%), ex.: 0,38
    cet_month            TEXT,     -- CET ao mês (%)
    cet_year             TEXT,     -- CET ao ano (%)
    created_at           TEXT NOT NULL,
    updated_at           TEXT NOT NULL
, indexer TEXT NOT NULL DEFAULT 'none'
    CHECK (indexer IN ('none', 'tr', 'ipca', 'igpm', 'inpc', 'incc', 'cdi', 'selic', 'poupanca'))) STRICT;

CREATE TABLE bank_registry (
    ispb          TEXT PRIMARY KEY,
    code          TEXT,                 -- COMPE (3 dígitos); vazio para quem não tem
    short_name    TEXT NOT NULL,
    full_name     TEXT NOT NULL,
    compe         INTEGER NOT NULL DEFAULT 0,
    started_on    TEXT,
    updated_at    TEXT NOT NULL
) STRICT;

CREATE TABLE fundamentus_cache (
    ticker      TEXT NOT NULL,
    page        TEXT NOT NULL CHECK (page IN ('details', 'dividends')),
    fetched_at  TEXT NOT NULL,              -- UTC RFC 3339
    found       INTEGER NOT NULL DEFAULT 1 CHECK (found IN (0, 1)),
    payload     TEXT NOT NULL,              -- JSON já interpretado
    PRIMARY KEY (ticker, page)
) STRICT;

CREATE TABLE balance_sync (
    id                 TEXT PRIMARY KEY,
    account_id         TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    date               TEXT NOT NULL,
    statement_balance  INTEGER NOT NULL,     -- saldo informado (centavos)
    computed_balance   INTEGER NOT NULL,     -- saldo do Treasury na data, antes do ajuste
    difference         INTEGER NOT NULL,     -- informado − calculado
    adjustment_txn_id  TEXT REFERENCES txn (id) ON DELETE SET NULL,
    status             TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'explained', 'ignored')),
    note               TEXT,
    explanation        TEXT,                 -- o motivo, quando achado
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL
) STRICT;

CREATE TABLE fx_operation (
    id                TEXT PRIMARY KEY,
    transfer_id       TEXT NOT NULL,
    date              TEXT NOT NULL,
    from_account_id   TEXT NOT NULL REFERENCES account (id),
    to_account_id     TEXT NOT NULL REFERENCES account (id),
    from_currency     TEXT NOT NULL,
    to_currency       TEXT NOT NULL,
    amount_from       INTEGER NOT NULL,      -- principal que sai (sem encargos)
    amount_to         INTEGER NOT NULL,      -- o que entra no destino
    rate              TEXT NOT NULL,         -- taxa aplicada (destino → origem, ex.: 5.42 BRL por USD)
    reference_rate    TEXT,                  -- taxa de referência na data
    reference_source  TEXT,                  -- ex.: bcb_ptax_venda, ecb
    spread_pct        TEXT,
    iof_pct           TEXT,
    iof_amount        INTEGER NOT NULL DEFAULT 0,
    fee_amount        INTEGER NOT NULL DEFAULT 0,
    fees_txn_ids      TEXT NOT NULL DEFAULT '[]',  -- lançamentos de IOF e tarifa
    vet               TEXT,                  -- valor efetivo total por unidade estrangeira
    notes             TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
) STRICT;

CREATE TABLE capital_share (
    account_id            TEXT PRIMARY KEY REFERENCES account (id) ON DELETE CASCADE,
    started_on            TEXT,
    contribution_pattern  TEXT,              -- texto dos aportes no extrato (ex.: "COTA CAPITAL")
    contribution_category_id TEXT REFERENCES category (id) ON DELETE SET NULL,
    include_in_net_worth  INTEGER NOT NULL DEFAULT 1 CHECK (include_in_net_worth IN (0, 1)),
    notes                 TEXT,
    created_at            TEXT NOT NULL,
    updated_at            TEXT NOT NULL
) STRICT;

CREATE TABLE capital_share_balance (
    id          TEXT PRIMARY KEY,
    account_id  TEXT NOT NULL REFERENCES capital_share (account_id) ON DELETE CASCADE,
    date        TEXT NOT NULL,
    balance     INTEGER NOT NULL,
    note        TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    UNIQUE (account_id, date)
) STRICT;

CREATE TABLE dev_feedback (
    id              TEXT PRIMARY KEY,
    title           TEXT NOT NULL,
    body            TEXT NOT NULL,
    area            TEXT NOT NULL CHECK (area IN ('agente', 'mcp', 'importacao', 'lancamentos',
                                                  'carteira', 'navegador', 'ui', 'dados', 'outro')),
    kind            TEXT NOT NULL CHECK (kind IN ('bug', 'melhoria', 'ferramenta_faltando', 'dados',
                                                  'documentacao')),
    importance      INTEGER NOT NULL DEFAULT 1 CHECK (importance >= 1),
    status          TEXT NOT NULL DEFAULT 'aberto'
                         CHECK (status IN ('aberto', 'em_andamento', 'resolvido', 'descartado')),
    created_by      TEXT NOT NULL,          -- cliente que criou (ex.: "agente (Claude Code)") ou "user"
    created_at      TEXT NOT NULL,          -- UTC RFC 3339
    updated_at      TEXT NOT NULL,
    last_bumped_at  TEXT,
    bumps           INTEGER NOT NULL DEFAULT 0
, position INTEGER CHECK (position IS NULL OR position >= 1), proposal TEXT, acceptance_criteria TEXT, code_pointers TEXT NOT NULL DEFAULT '[]', labels TEXT NOT NULL DEFAULT '[]', assignee TEXT, expected TEXT, actual TEXT, repro_steps TEXT, user_impact TEXT
    CHECK (user_impact IS NULL OR user_impact IN ('baixo', 'medio', 'alto')), confirmed_by_user INTEGER NOT NULL DEFAULT 0
    CHECK (confirmed_by_user IN (0, 1)), client_version TEXT, app_version TEXT, tool_calls TEXT NOT NULL DEFAULT '[]', closed_at TEXT) STRICT;

CREATE TABLE dev_feedback_note (
    id           TEXT PRIMARY KEY,
    feedback_id  TEXT NOT NULL REFERENCES dev_feedback (id) ON DELETE CASCADE,
    note         TEXT,
    client       TEXT NOT NULL,
    created_at   TEXT NOT NULL
, kind TEXT NOT NULL DEFAULT 'reforco'
    CHECK (kind IN ('reforco', 'status', 'mescla', 'comentario'))) STRICT;

CREATE TABLE research_filing (
    issuer_id   TEXT NOT NULL,              -- CNPJ só com dígitos
    kind        TEXT NOT NULL CHECK (kind IN ('dfp', 'itr', 'fii_mensal', 'fii_trimestral')),
    reference   TEXT NOT NULL,              -- data de referência (YYYY-MM-DD)
    version     INTEGER NOT NULL,
    document    TEXT,                       -- número do documento na CVM (ID_DOC), quando há
    filed_at    TEXT NOT NULL,              -- data da entrega (YYYY-MM-DD)
    link        TEXT,
    fetched_at  TEXT NOT NULL,
    PRIMARY KEY (issuer_id, kind, reference)
) STRICT;

CREATE TABLE research_auditor (
    issuer_id     TEXT NOT NULL,
    auditor       TEXT NOT NULL,
    auditor_cnpj  TEXT NOT NULL,            -- CPF/CNPJ do auditor só com dígitos
    start         TEXT NOT NULL,            -- início da atuação (YYYY-MM-DD; vazio se não informado)
    end           TEXT,                     -- fim da atuação (NULL = ainda atua)
    reference     TEXT NOT NULL,            -- data de referência do FCA de onde veio
    fetched_at    TEXT NOT NULL,
    PRIMARY KEY (issuer_id, auditor_cnpj, start)
) STRICT;

CREATE TABLE bank_institution (
    cod_inst         TEXT PRIMARY KEY,      -- CodInst (ex.: C0080099)
    name             TEXT NOT NULL,
    cnpj8            TEXT,
    tcb              TEXT,                  -- tipo de consolidado bancário (b1, b2, n1…)
    segment          TEXT,                  -- segmento prudencial (S1 a S5)
    prudential_code  TEXT,                  -- CodConglomeradoPrudencial
    active           INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    fetched_at       TEXT NOT NULL
) STRICT;

CREATE TABLE bank_prudential (
    cod_inst    TEXT NOT NULL,
    period      TEXT NOT NULL,              -- YYYY-MM
    item        TEXT NOT NULL,              -- basel_ratio, cet1_ratio, net_income…
    value       TEXT NOT NULL,              -- decimal exato
    source      TEXT NOT NULL,              -- bacen_ifdata
    fetched_at  TEXT NOT NULL,
    PRIMARY KEY (cod_inst, period, item)
) STRICT;

CREATE TABLE bank_ifdata_fetch (
    period      TEXT NOT NULL,              -- YYYY-MM
    report      TEXT NOT NULL,              -- cadastro, 1, 2, 4, 5, 16
    codes       TEXT,                       -- JSON com os CodInst pedidos (NULL = todos)
    rows        INTEGER NOT NULL DEFAULT 0,
    fetched_at  TEXT NOT NULL,
    PRIMARY KEY (period, report)
) STRICT;

CREATE TABLE score_universe (
    class                TEXT NOT NULL,          -- equity_br, reit, bank_ifdata…
    id                   TEXT NOT NULL,          -- emissor (CNPJ) ou instituição (IF.data)
    ticker               TEXT,
    name                 TEXT NOT NULL,
    sector               TEXT,                   -- setor fino (FCA normalizado, subsegmento do FII)
    segment              TEXT NOT NULL,          -- segmento da metodologia
    metrics              TEXT NOT NULL,          -- JSON {chave: {value, avg, trend, worst…}}
    as_of                TEXT NOT NULL,
    computed_at          TEXT NOT NULL,
    methodology_version  TEXT NOT NULL,
    PRIMARY KEY (class, id)
) STRICT;

CREATE TABLE score_snapshot (
    subject              TEXT NOT NULL,          -- id do ativo cadastrado ou "ticker:XXXX3"
    asset_id             TEXT,
    ticker               TEXT,
    as_of                TEXT NOT NULL,
    methodology_version  TEXT NOT NULL,
    status               TEXT NOT NULL CHECK (status IN ('ok', 'partial', 'insufficient_data', 'not_supported')),
    score                REAL,
    band                 TEXT NOT NULL,
    confidence           REAL NOT NULL,
    payload              TEXT NOT NULL,          -- AssetScore completo (JSON)
    computed_at          TEXT NOT NULL,
    PRIMARY KEY (subject, as_of, methodology_version)
) STRICT;

CREATE TABLE score_event (
    id                   TEXT PRIMARY KEY,
    asset_id             TEXT REFERENCES asset (id) ON DELETE CASCADE,
    ticker               TEXT,
    kind                 TEXT NOT NULL,          -- recuperacao_judicial, fraude, litigio, ambiental, governanca, regulacao, outro
    title                TEXT NOT NULL,
    event_date           TEXT NOT NULL,
    severity             TEXT NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    cap                  REAL,
    penalty              REAL,
    half_life_quarters   REAL,
    evidence_url         TEXT,
    source               TEXT NOT NULL,          -- user, agente, cvm_ipe…
    status               TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected', 'resolved')),
    resolved_at          TEXT,
    note                 TEXT,
    created_at           TEXT NOT NULL,
    updated_at           TEXT NOT NULL,
    CHECK (asset_id IS NOT NULL OR ticker IS NOT NULL)
) STRICT;

CREATE TABLE score_feedback (
    id                   TEXT PRIMARY KEY,
    subject              TEXT NOT NULL,
    asset_id             TEXT REFERENCES asset (id) ON DELETE CASCADE,
    ticker               TEXT,
    as_of                TEXT NOT NULL,
    methodology_version  TEXT NOT NULL,
    score                REAL,
    band                 TEXT NOT NULL,
    verdict              TEXT NOT NULL CHECK (verdict IN ('discordo', 'concordo')),
    expected_band        TEXT,
    reason               TEXT NOT NULL,
    created_at           TEXT NOT NULL
) STRICT;

CREATE TABLE research_dividend (
    ticker          TEXT NOT NULL,           -- código de negociação (PETR4, HGLG11)
    kind            TEXT NOT NULL,           -- dividend, jcp, fii_income, amortization, other
    ex_date         TEXT NOT NULL,           -- último dia com direito (data-com)
    value           TEXT NOT NULL,           -- valor por ação/cota, decimal exato (na moeda do papel)
    per_shares      INTEGER NOT NULL DEFAULT 1,  -- valor por quantas ações (fator da B3)
    approved_on     TEXT,
    pay_date        TEXT,
    label           TEXT,                    -- como a fonte descreve (ex.: "JRS CAP PROPRIO")
    -- Ordem entre linhas iguais (mesmo papel, tipo, data-com e valor): a B3 anuncia JCP em
    -- parcelas de mesmo valor pagas em datas diferentes.
    seq             INTEGER NOT NULL DEFAULT 0,
    source          TEXT NOT NULL,           -- b3_listados
    fetched_at      TEXT NOT NULL,
    PRIMARY KEY (ticker, kind, ex_date, value, seq, source)
) STRICT;

CREATE TABLE research_stock_event (
    ticker          TEXT NOT NULL,
    kind            TEXT NOT NULL,           -- bonus, split, reverse_split, other
    ex_date         TEXT NOT NULL,
    factor          TEXT NOT NULL,           -- fator ou percentual como a fonte informa, decimal
    approved_on     TEXT,
    label           TEXT,
    source          TEXT NOT NULL,
    fetched_at      TEXT NOT NULL,
    PRIMARY KEY (ticker, kind, ex_date, source)
) STRICT;

CREATE TABLE research_fetch_log (
    source          TEXT NOT NULL,
    key             TEXT NOT NULL,           -- ticker, código da CVM…
    fetched_at      TEXT NOT NULL,
    status          TEXT NOT NULL,           -- ok, not_found, error
    detail          TEXT,
    PRIMARY KEY (source, key)
) STRICT;

CREATE TABLE project (
    id                 TEXT PRIMARY KEY,
    name               TEXT NOT NULL,
    physical_asset_id  TEXT REFERENCES physical_asset (id) ON DELETE SET NULL,
    status             TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('planned', 'active', 'done')),
    budget             INTEGER CHECK (budget IS NULL OR budget >= 0),
    start_date         TEXT,
    end_date           TEXT,
    notes              TEXT,
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL
) STRICT;

CREATE TABLE project_item (
    id           TEXT PRIMARY KEY,
    project_id   TEXT NOT NULL REFERENCES project (id) ON DELETE CASCADE,
    date         TEXT NOT NULL,
    amount       INTEGER NOT NULL CHECK (amount <> 0),
    description  TEXT NOT NULL,
    category_id  TEXT REFERENCES category (id) ON DELETE SET NULL,
    notes        TEXT,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
) STRICT;

CREATE TABLE "ai_proposal" (
    id             TEXT PRIMARY KEY,
    change_set_id  TEXT NOT NULL REFERENCES change_set (id) ON DELETE CASCADE,
    client         TEXT NOT NULL,
    command        TEXT NOT NULL,
    args           TEXT NOT NULL DEFAULT '{}',
    level          TEXT NOT NULL CHECK (level IN ('reversible', 'sensitive')),
    status         TEXT NOT NULL DEFAULT 'pending'
                   CHECK (status IN ('pending', 'approved', 'rejected', 'failed', 'withdrawn')),
    reason         TEXT NOT NULL,
    created_at     TEXT NOT NULL,
    reviewed_at    TEXT,
    error          TEXT,
    note           TEXT
) STRICT;

CREATE TABLE ai_review_seen (
    txn_id       TEXT PRIMARY KEY,
    fingerprint  TEXT NOT NULL,
    seen_at      TEXT NOT NULL
) STRICT;

CREATE TABLE tax_category (
    category_id  TEXT PRIMARY KEY REFERENCES category (id) ON DELETE CASCADE,
    group_key    TEXT,
    rate         REAL CHECK (rate IS NULL OR (rate >= 0 AND rate < 100)),
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
) STRICT;

CREATE TABLE tax_income_report (
    id                TEXT PRIMARY KEY,
    year              INTEGER NOT NULL,
    member_id         TEXT REFERENCES member (id) ON DELETE SET NULL,
    payer             TEXT NOT NULL,
    gross             INTEGER NOT NULL CHECK (gross >= 0),
    social_security   INTEGER NOT NULL DEFAULT 0 CHECK (social_security >= 0),
    income_tax        INTEGER NOT NULL DEFAULT 0 CHECK (income_tax >= 0),
    thirteenth_gross  INTEGER NOT NULL DEFAULT 0 CHECK (thirteenth_gross >= 0),
    thirteenth_tax    INTEGER NOT NULL DEFAULT 0 CHECK (thirteenth_tax >= 0),
    notes             TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
) STRICT;

CREATE TABLE "pension_plan" (
    asset_id         TEXT PRIMARY KEY REFERENCES asset (id) ON DELETE CASCADE,
    plan_type        TEXT NOT NULL DEFAULT 'vgbl' CHECK (plan_type IN ('vgbl', 'pgbl', 'efpc')),
    tax_regime       TEXT NOT NULL DEFAULT 'regressive' CHECK (tax_regime IN ('progressive', 'regressive')),
    insurer          TEXT,
    admin_fee_pct    TEXT,
    loading_fee_pct  TEXT,
    start_date       TEXT,
    notes            TEXT,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
) STRICT;

CREATE TABLE "import_batch" (
    id            TEXT PRIMARY KEY,
    filename      TEXT NOT NULL,
    format        TEXT NOT NULL CHECK (format IN ('ofx', 'csv', 'pdf')),
    file_sha256   TEXT NOT NULL,
    account_id    TEXT NOT NULL REFERENCES account (id),
    total_rows    INTEGER NOT NULL DEFAULT 0,
    inserted      INTEGER NOT NULL DEFAULT 0,
    duplicates    INTEGER NOT NULL DEFAULT 0,
    errors        TEXT NOT NULL DEFAULT '[]',  -- JSON
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
) STRICT;

CREATE TABLE "dividend_event" (
    id                TEXT PRIMARY KEY,
    asset_id          TEXT NOT NULL REFERENCES asset (id) ON DELETE CASCADE,
    kind              TEXT NOT NULL CHECK (kind IN ('dividend', 'jcp', 'fii_income', 'coupon',
                                                    'amortization', 'other')),
    declared_date     TEXT,
    ex_date           TEXT,                 -- data-com
    pay_date          TEXT,
    amount_per_share  TEXT NOT NULL,
    currency          TEXT NOT NULL,
    estimated         INTEGER NOT NULL DEFAULT 0 CHECK (estimated IN (0, 1)),  -- radar (projeção)
    source            TEXT NOT NULL,
    fetched_at        TEXT NOT NULL,
    UNIQUE (asset_id, kind, ex_date, amount_per_share)
) STRICT;

CREATE TABLE ai_memory_link (
    memory_id    TEXT NOT NULL REFERENCES ai_memory (id) ON DELETE CASCADE,
    entity_kind  TEXT NOT NULL CHECK (entity_kind IN ('account', 'asset', 'member', 'category', 'goal', 'institution', 'topic')),
    entity_id    TEXT NOT NULL,
    label        TEXT NOT NULL,
    PRIMARY KEY (memory_id, entity_kind, entity_id)
) STRICT;

CREATE TABLE credit_rate (
    period_start  TEXT NOT NULL,                 -- início do período (AAAA-MM-DD)
    period_end    TEXT NOT NULL,
    modality      TEXT NOT NULL,                 -- como o BCB escreve ("Crédito pessoal não consignado - Prefixado")
    institution   TEXT NOT NULL,
    cnpj8         TEXT NOT NULL,
    rate_month    REAL NOT NULL,                 -- % a.m.
    rate_year     REAL NOT NULL,                 -- % a.a.
    position      INTEGER,
    fetched_at    TEXT NOT NULL,
    PRIMARY KEY (period_start, modality, cnpj8)
) STRICT;

CREATE TABLE income_source (
    id                TEXT PRIMARY KEY,
    member_id         TEXT NOT NULL REFERENCES member (id),
    name              TEXT NOT NULL,
    kind              TEXT NOT NULL CHECK (kind IN ('clt', 'public_servant', 'retirement', 'pension',
                                                    'pj', 'self_employed', 'informal', 'rental',
                                                    'benefit', 'other')),
    employer          TEXT,
    employer_cnpj     TEXT,
    -- Valores mensais de referência (centavos); o líquido pode vir do holerite.
    gross_monthly     INTEGER,
    net_monthly       INTEGER,
    -- Renda que muda de mês a mês (informal, comissão): a previsão usa a média recente.
    variable          INTEGER NOT NULL DEFAULT 0 CHECK (variable IN (0, 1)),
    pay_day           INTEGER CHECK (pay_day BETWEEN 1 AND 31),
    has_thirteenth    INTEGER NOT NULL DEFAULT 0 CHECK (has_thirteenth IN (0, 1)),
    vacation_month    INTEGER CHECK (vacation_month BETWEEN 1 AND 12),
    -- Dependentes no IRRF desta fonte.
    dependents        INTEGER NOT NULL DEFAULT 0,
    account_id        TEXT REFERENCES account (id),
    category_id       TEXT REFERENCES category (id),
    start_date        TEXT,
    end_date          TEXT,
    archived          INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    notes             TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
) STRICT;

CREATE TABLE payslip (
    id                TEXT PRIMARY KEY,
    source_id         TEXT NOT NULL REFERENCES income_source (id) ON DELETE CASCADE,
    -- Competência (YYYY-MM) e tipo: mensal, adiantamento, 13º (1ª e 2ª parcela), férias, PLR.
    reference_month   TEXT NOT NULL,
    kind              TEXT NOT NULL CHECK (kind IN ('monthly', 'advance', 'thirteenth_1', 'thirteenth_2',
                                                    'vacation', 'plr', 'other')),
    pay_date          TEXT,
    gross             INTEGER NOT NULL,
    social_security   INTEGER NOT NULL DEFAULT 0,  -- INSS (ou RPPS)
    income_tax        INTEGER NOT NULL DEFAULT 0,  -- IRRF
    other_deductions  INTEGER NOT NULL DEFAULT 0,
    net               INTEGER NOT NULL,
    fgts              INTEGER,
    -- Linhas do holerite: [{"description", "amount", "deduction"}] (JSON).
    items             TEXT NOT NULL DEFAULT '[]',
    layout            TEXT,
    file_sha256       TEXT,
    txn_id            TEXT REFERENCES txn (id) ON DELETE SET NULL,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    UNIQUE (source_id, reference_month, kind)
) STRICT;

CREATE TABLE txn_link (
    txn_id                TEXT PRIMARY KEY REFERENCES txn (id) ON DELETE CASCADE,
    target_kind           TEXT NOT NULL CHECK (target_kind IN ('trade', 'capital_share', 'goal')),
    target_id             TEXT NOT NULL,
    previous_kind         TEXT NOT NULL,
    previous_category_id  TEXT,
    note                  TEXT,
    created_at            TEXT NOT NULL
) STRICT;

CREATE TABLE receipt_item (
    id            TEXT PRIMARY KEY,
    receipt_id    TEXT NOT NULL REFERENCES receipt (id) ON DELETE CASCADE,
    line          INTEGER NOT NULL,
    code          TEXT,
    description   TEXT NOT NULL,
    quantity      TEXT,
    unit          TEXT,
    unit_price    INTEGER,
    total         INTEGER NOT NULL,
    item_group    TEXT NOT NULL,
    group_origin  TEXT NOT NULL DEFAULT 'auto' CHECK (group_origin IN ('auto', 'manual')),
    -- Produto da lista do IBPT que deu a alíquota, e o imposto do item (centavos).
    ibpt_product  TEXT,
    tax_rate      REAL,
    tax           INTEGER NOT NULL DEFAULT 0,
    -- `nfce`: rateio do imposto impresso; `ibpt`: estimado pela lista do IBPT.
    tax_origin    TEXT NOT NULL CHECK (tax_origin IN ('nfce', 'ibpt')),
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
, product_id TEXT REFERENCES product (id) ON DELETE SET NULL, gtin TEXT, base_qty TEXT, base_unit TEXT CHECK (base_unit IN ('kg', 'l', 'un')), base_price INTEGER, discount INTEGER NOT NULL DEFAULT 0) STRICT;

CREATE TABLE mail_account (
    id                  TEXT PRIMARY KEY,
    name                TEXT NOT NULL,
    kind                TEXT NOT NULL CHECK (kind IN ('folder', 'imap')),
    -- Pasta local vigiada.
    folder_path         TEXT,
    -- IMAP.
    provider            TEXT,
    host                TEXT,
    port                INTEGER,
    security            TEXT NOT NULL DEFAULT 'tls' CHECK (security IN ('tls', 'starttls')),
    username            TEXT,
    -- SHA-256 do certificado fixado (Proton Bridge local, com certificado próprio).
    pinned_cert         TEXT,
    -- authserv-id do provedor no Authentication-Results (ex.: mx.google.com).
    authserv_id         TEXT,
    -- `dedicated`: caixa só para o Treasury; `main`: a caixa principal de alguém.
    mailbox_kind        TEXT NOT NULL DEFAULT 'dedicated' CHECK (mailbox_kind IN ('dedicated', 'main')),
    -- JSON: pastas permitidas e bloqueadas (nomes com curinga ou marcas RFC 6154).
    allowed_folders     TEXT NOT NULL DEFAULT '["INBOX"]',
    blocked_folders     TEXT NOT NULL DEFAULT '["\\Junk","\\Trash","\\Sent","\\Drafts","\\All"]',
    -- JSON: remetentes permitidos e bloqueados (globais) e os endereços da família.
    allow_senders       TEXT NOT NULL DEFAULT '[]',
    block_senders       TEXT NOT NULL DEFAULT '[]',
    family_addresses    TEXT NOT NULL DEFAULT '[]',
    -- Sem regra casada: `ignore` (padrão) ou `review`.
    default_action      TEXT NOT NULL DEFAULT 'ignore' CHECK (default_action IN ('ignore', 'review')),
    -- JSON: {mode: never|after_success|after_days|both, days, grace_days}.
    retention           TEXT NOT NULL DEFAULT '{"mode":"never","days":30,"grace_days":7}',
    -- Modo de teste da exclusão: até quando só mostra o que teria apagado; depois exige
    -- confirmação explícita.
    retention_test_until TEXT,
    retention_confirmed INTEGER NOT NULL DEFAULT 0,
    keep_original       INTEGER NOT NULL DEFAULT 1,
    interval_minutes    INTEGER NOT NULL DEFAULT 15 CHECK (interval_minutes BETWEEN 5 AND 120),
    enabled             INTEGER NOT NULL DEFAULT 1,
    -- Backup restaurado: a conexão precisa ser confirmada de novo antes de buscar.
    needs_reconfirm     INTEGER NOT NULL DEFAULT 0,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
) STRICT;

CREATE TABLE mail_rule (
    id          TEXT PRIMARY KEY,
    -- Nula: vale para todas as contas.
    account_id  TEXT REFERENCES mail_account (id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    priority    INTEGER NOT NULL DEFAULT 100,
    enabled     INTEGER NOT NULL DEFAULT 1,
    -- JSON: remetente, assunto e condições avançadas (docs/EMAIL-INGEST.md).
    conditions  TEXT NOT NULL,
    action      TEXT NOT NULL DEFAULT 'review' CHECK (action IN ('review', 'auto', 'ignore')),
    ai          TEXT NOT NULL DEFAULT 'never' CHECK (ai IN ('never', 'on_request', 'auto')),
    -- JSON: retenção própria da regra (sobrescreve a da conta) e o modelo do corpo do e-mail.
    retention   TEXT,
    template    TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE mail_folder_state (
    account_id   TEXT NOT NULL REFERENCES mail_account (id) ON DELETE CASCADE,
    folder       TEXT NOT NULL,
    uidvalidity  INTEGER NOT NULL,
    last_uid     INTEGER NOT NULL,
    updated_at   TEXT NOT NULL,
    PRIMARY KEY (account_id, folder)
) STRICT;

CREATE TABLE mail_message (
    id               TEXT PRIMARY KEY,
    account_id       TEXT NOT NULL REFERENCES mail_account (id) ON DELETE CASCADE,
    folder           TEXT NOT NULL,
    uid              INTEGER,
    -- UIDVALIDITY da pasta quando a mensagem foi lida (a retenção só apaga se ainda for a mesma).
    uidvalidity      INTEGER,
    -- Pasta local: o nome do arquivo (nunca vira caminho fora da pasta da conta).
    file_name        TEXT,
    message_id       TEXT,
    sender           TEXT,
    original_sender  TEXT,
    subject          TEXT,
    date             TEXT,
    size             INTEGER NOT NULL DEFAULT 0,
    authenticated    INTEGER NOT NULL DEFAULT 0,
    -- Remetente confiável para "aplicar sozinho" (autenticado, ou encaminhado por membro
    -- autenticado com o remetente original nos permitidos da regra).
    trusted          INTEGER NOT NULL DEFAULT 0,
    auth_detail      TEXT,
    rule_id          TEXT REFERENCES mail_rule (id) ON DELETE SET NULL,
    rule_name        TEXT,
    status           TEXT NOT NULL CHECK (status IN ('awaiting_review', 'imported', 'duplicate', 'failed', 'unrecognized', 'ignored')),
    raw_hash         TEXT,
    raw              BLOB,
    error            TEXT,
    received_at      TEXT NOT NULL,
    decided_at       TEXT,
    deleted_at       TEXT,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
) STRICT;

CREATE TABLE mail_part (
    id          TEXT PRIMARY KEY,
    message_id  TEXT NOT NULL REFERENCES mail_message (id) ON DELETE CASCADE,
    idx         INTEGER NOT NULL,
    filename    TEXT NOT NULL,
    mime        TEXT,
    size        INTEGER NOT NULL,
    sha256      TEXT NOT NULL,
    -- `statement`, `brokerage_note`, `b3_report`, `pension_statement`, `card_statement`,
    -- `payslip`, `receipt`, `image`, `purchase`, `pix`, `template`, `unknown`.
    kind        TEXT NOT NULL,
    label       TEXT NOT NULL,
    layout_id   TEXT,
    status      TEXT NOT NULL CHECK (status IN ('awaiting_review', 'imported', 'duplicate', 'failed', 'unrecognized', 'ignored', 'ai_pending')),
    -- JSON: o que a leitura achou (o lançamento do aviso, o resumo da prévia).
    detail      TEXT,
    dedup_key   TEXT,
    result_ref  TEXT,
    error       TEXT,
    content     BLOB,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE mail_seen (
    account_id  TEXT NOT NULL REFERENCES mail_account (id) ON DELETE CASCADE,
    key         TEXT NOT NULL,
    at          TEXT NOT NULL,
    PRIMARY KEY (account_id, key)
) STRICT;

CREATE TABLE mail_audit (
    id          TEXT PRIMARY KEY,
    account_id  TEXT REFERENCES mail_account (id) ON DELETE CASCADE,
    at          TEXT NOT NULL,
    action      TEXT NOT NULL,
    message_id  TEXT,
    detail      TEXT NOT NULL
) STRICT;

CREATE TABLE property_tax_bill (
    id                   TEXT PRIMARY KEY,
    asset_id             TEXT NOT NULL REFERENCES physical_asset (id) ON DELETE CASCADE,
    year                 INTEGER NOT NULL CHECK (year BETWEEN 1990 AND 2100),
    registration         TEXT,               -- cadastro / inscrição / indicação fiscal / SQL
    city                 TEXT,
    land_area            TEXT,               -- m² (decimal em texto)
    built_area           TEXT,
    land_value           INTEGER CHECK (land_value >= 0),      -- valor venal do terreno
    building_value       INTEGER CHECK (building_value >= 0),  -- valor venal predial
    land_tax             INTEGER CHECK (land_tax >= 0),        -- imposto territorial
    building_tax         INTEGER CHECK (building_tax >= 0),    -- imposto predial
    taxes_total          INTEGER CHECK (taxes_total >= 0),     -- total de impostos
    lighting_fee         INTEGER NOT NULL DEFAULT 0 CHECK (lighting_fee >= 0),
    waste_fee            INTEGER NOT NULL DEFAULT 0 CHECK (waste_fee >= 0),
    other_fees           INTEGER NOT NULL DEFAULT 0 CHECK (other_fees >= 0),
    discount             INTEGER NOT NULL DEFAULT 0 CHECK (discount >= 0),
    good_payer_discount  INTEGER NOT NULL DEFAULT 0 CHECK (good_payer_discount >= 0),
    total_due            INTEGER NOT NULL CHECK (total_due >= 0),
    currency             TEXT NOT NULL DEFAULT 'BRL',
    plan                 TEXT CHECK (plan IN ('single', 'installments')),
    notes                TEXT,
    created_at           TEXT NOT NULL,
    updated_at           TEXT NOT NULL,
    UNIQUE (asset_id, year)
) STRICT;

CREATE TABLE property_tax_installment (
    id         TEXT PRIMARY KEY,
    bill_id    TEXT NOT NULL REFERENCES property_tax_bill (id) ON DELETE CASCADE,
    number     INTEGER NOT NULL CHECK (number BETWEEN 0 AND 24),  -- 0 = cota única
    due_date   TEXT NOT NULL,
    amount     INTEGER NOT NULL CHECK (amount > 0),
    paid_on    TEXT,
    txn_id     TEXT REFERENCES txn (id) ON DELETE SET NULL,
    UNIQUE (bill_id, number)
) STRICT;

CREATE TABLE asset_field_meta (
    asset_id    TEXT NOT NULL REFERENCES asset (id) ON DELETE CASCADE,
    field       TEXT NOT NULL,
    origin      TEXT NOT NULL CHECK (origin IN ('user', 'legacy', 'import', 'enrichment', 'search')),
    source_id   TEXT,                       -- cvm_fca, cvm_fii, cvm_registro, sec, fundamentus…
    source_ref  TEXT,                       -- link ou documento da fonte
    as_of       TEXT,                       -- data do dado na fonte (AAAA-MM-DD)
    confidence  INTEGER CHECK (confidence BETWEEN 0 AND 100),
    locked      INTEGER NOT NULL DEFAULT 0 CHECK (locked IN (0, 1)),
    updated_at  TEXT NOT NULL,
    PRIMARY KEY (asset_id, field)
) STRICT;

CREATE TABLE asset_attribute (
    asset_id    TEXT NOT NULL REFERENCES asset (id) ON DELETE CASCADE,
    field       TEXT NOT NULL,
    value       TEXT NOT NULL,
    scheme      TEXT,
    updated_at  TEXT NOT NULL,
    PRIMARY KEY (asset_id, field)
) STRICT;

CREATE TABLE asset_observation (
    id            TEXT PRIMARY KEY,
    asset_id      TEXT NOT NULL REFERENCES asset (id) ON DELETE CASCADE,
    field         TEXT NOT NULL,
    value         TEXT NOT NULL,
    value_norm    TEXT NOT NULL,
    scheme        TEXT,
    source_id     TEXT NOT NULL,
    tier          INTEGER NOT NULL CHECK (tier BETWEEN 1 AND 4),
    source_ref    TEXT,
    published_at  TEXT,
    observed_at   TEXT NOT NULL
) STRICT;

CREATE TABLE asset_study_context (
    asset_id    TEXT PRIMARY KEY REFERENCES asset (id) ON DELETE CASCADE,
    text        TEXT NOT NULL,
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE asset_study (
    id                  TEXT PRIMARY KEY,
    asset_id            TEXT NOT NULL REFERENCES asset (id) ON DELETE CASCADE,
    created_at          TEXT NOT NULL,
    engine_id           TEXT,
    engine_name         TEXT NOT NULL,
    provider            TEXT NOT NULL,
    model               TEXT,
    question            TEXT,
    context_snapshot    TEXT,
    package_hash        TEXT NOT NULL,
    include_position    INTEGER NOT NULL DEFAULT 0 CHECK (include_position IN (0, 1)),
    include_values      INTEGER NOT NULL DEFAULT 0 CHECK (include_values IN (0, 1)),
    output_json         TEXT NOT NULL,
    checks_json         TEXT NOT NULL,
    confidence          TEXT CHECK (confidence IN ('low', 'medium', 'high')),
    disclaimer_version  TEXT NOT NULL
) STRICT;

CREATE TABLE "ai_task_route" (
    task       TEXT NOT NULL CHECK (task IN ('chat', 'categorize', 'pdf', 'monthly_close', 'alerts', 'study')),
    position   INTEGER NOT NULL,
    engine_id  TEXT NOT NULL REFERENCES ai_engine (id) ON DELETE CASCADE,
    PRIMARY KEY (task, position),
    UNIQUE (task, engine_id)
) STRICT;

CREATE TABLE fgts_account (
    account_id            TEXT PRIMARY KEY REFERENCES account (id) ON DELETE CASCADE,
    employer_name         TEXT,
    employer_cnpj         TEXT,
    admission_date        TEXT,
    option_date           TEXT,
    leave_date            TEXT,
    leave_code            TEXT,
    category              TEXT,
    -- Juros ao ano do extrato ('3' na maioria; progressiva só para optantes antes de 1971).
    interest_rate         TEXT NOT NULL DEFAULT '3',
    -- SHA-256 do nº da conta (estabelecimento/conta + unidade).
    account_hash          TEXT,
    -- "Valor para fins rescisórios" do último extrato (centavos) e a data do extrato.
    rescission_base       INTEGER,
    rescission_base_date  TEXT,
    last_statement_date   TEXT,
    -- Renda CLT ligada, para conferir os depósitos com o holerite.
    income_source_id      TEXT REFERENCES income_source (id) ON DELETE SET NULL,
    created_at            TEXT NOT NULL,
    updated_at            TEXT NOT NULL
) STRICT;

CREATE TABLE fgts_entry (
    txn_id             TEXT PRIMARY KEY REFERENCES txn (id) ON DELETE CASCADE,
    account_id         TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    kind               TEXT NOT NULL CHECK (kind IN ('deposit', 'deposit_extra', 'jam',
                                                     'jam_adjustment', 'distribution',
                                                     'withdrawal', 'reversal')),
    -- Mês de referência do depósito (AAAA-MM).
    competence         TEXT,
    -- Coeficiente do JAM como no extrato ("0,002466" → "0.002466").
    coefficient        TEXT,
    -- Ano-base da distribuição de resultado.
    base_year          INTEGER,
    withdrawal_code    TEXT,
    -- rescisao, aposentadoria, moradia, aniversario, doenca, emergencial, outros
    withdrawal_reason  TEXT,
    withdrawal_ref     TEXT,
    -- Saldo impresso na linha do extrato (centavos), para reconciliar.
    statement_total    INTEGER,
    -- Ordem da linha no extrato (o saldo impresso segue essa ordem, não a das datas).
    statement_seq      INTEGER,
    -- 1 = projetado pelo app; 0 = veio do extrato (ou foi lançado à mão).
    computed           INTEGER NOT NULL DEFAULT 0 CHECK (computed IN (0, 1)),
    created_at         TEXT NOT NULL
) STRICT;

CREATE TABLE fgts_modality (
    member_id             TEXT PRIMARY KEY REFERENCES member (id) ON DELETE CASCADE,
    modality              TEXT NOT NULL CHECK (modality IN ('rescisao', 'aniversario')),
    since                 TEXT NOT NULL,
    -- Pedido de volta ao saque-rescisão (vale no 1º dia do 25º mês depois).
    return_requested_at   TEXT,
    -- Empréstimo de antecipação do saque-aniversário, quando houver.
    anticipation_debt_id  TEXT REFERENCES loan (id) ON DELETE SET NULL,
    updated_at            TEXT NOT NULL
) STRICT;

CREATE TABLE product_category (
    id             TEXT PRIMARY KEY,
    parent_id      TEXT REFERENCES product_category (id) ON DELETE CASCADE,
    -- Chave da semente (`hortifruti`, `mercearia.cereais`…); `NULL`: criada pela pessoa.
    seed_key       TEXT UNIQUE,
    name           TEXT NOT NULL,
    is_food        INTEGER NOT NULL DEFAULT 1 CHECK (is_food IN (0, 1)),
    -- `NULL`: herda a do pai.
    class          TEXT CHECK (class IN ('essential', 'complementary', 'superfluous')),
    -- Grupo de imposto sobre consumo (o que o `item_group` faz).
    tax_group      TEXT,
    -- Subitem do IPCA ou classe COICOP (fase 3).
    inflation_key  TEXT,
    sort           INTEGER NOT NULL DEFAULT 0,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
) STRICT;

CREATE TABLE product (
    id               TEXT PRIMARY KEY,
    -- GTIN/EAN com dígito verificador conferido; `NULL` para granel e balança.
    gtin             TEXT UNIQUE,
    name             TEXT NOT NULL,
    brand            TEXT,
    -- Tamanho da embalagem em kg, L ou unidades (decimal exato em texto).
    pack_qty         TEXT,
    pack_unit        TEXT CHECK (pack_unit IN ('kg', 'l', 'un')),
    category_id      TEXT REFERENCES product_category (id) ON DELETE SET NULL,
    category_origin  TEXT NOT NULL DEFAULT 'auto' CHECK (category_origin IN ('auto', 'manual')),
    -- Classe do próprio produto (`NULL`: a da categoria).
    class            TEXT CHECK (class IN ('essential', 'complementary', 'superfluous')),
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
, line_key TEXT) STRICT;

CREATE TABLE product_alias (
    id           TEXT PRIMARY KEY,
    product_id   TEXT NOT NULL REFERENCES product (id) ON DELETE CASCADE,
    -- CNPJ da loja (só dígitos) ou o nome normalizado.
    store_key    TEXT NOT NULL,
    code         TEXT NOT NULL DEFAULT '',
    description  TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    UNIQUE (store_key, code, description)
) STRICT;

CREATE TABLE receipt_payment (
    id          TEXT PRIMARY KEY,
    receipt_id  TEXT NOT NULL REFERENCES receipt (id) ON DELETE CASCADE,
    txn_id      TEXT NOT NULL REFERENCES txn (id) ON DELETE CASCADE,
    amount      INTEGER NOT NULL,
    created_at  TEXT NOT NULL,
    UNIQUE (receipt_id, txn_id)
) STRICT;

CREATE TABLE "receipt" (
    id             TEXT PRIMARY KEY,
    access_key     TEXT UNIQUE,
    date           TEXT NOT NULL,
    issued_at      TEXT,
    store_name     TEXT,
    store_cnpj     TEXT,
    uf             TEXT,
    items_total    INTEGER NOT NULL,
    discount       INTEGER NOT NULL DEFAULT 0,
    paid           INTEGER NOT NULL,
    tax_total      INTEGER,
    tax_federal    INTEGER,
    tax_state      INTEGER,
    tax_municipal  INTEGER,
    origin         TEXT NOT NULL CHECK (origin IN ('nfce', 'pdf', 'xml', 'text', 'agent', 'manual')),
    url            TEXT,
    txn_id         TEXT REFERENCES txn (id) ON DELETE SET NULL,
    member_id      TEXT REFERENCES member (id) ON DELETE SET NULL,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL,
    -- O lançamento `txn_id` foi criado junto com a compra manual.
    txn_created    INTEGER NOT NULL DEFAULT 0 CHECK (txn_created IN (0, 1))
, currency TEXT) STRICT;

CREATE TABLE "account" (
    id                    TEXT PRIMARY KEY,
    name                  TEXT NOT NULL,
    kind                  TEXT NOT NULL CHECK (kind IN ('checking', 'savings', 'cash', 'credit_card',
                                                        'brokerage', 'foreign', 'benefit', 'fgts', 'retirement',
                                                        'other')),
    currency              TEXT NOT NULL DEFAULT 'BRL',
    institution_id        TEXT REFERENCES institution (id),
    opening_balance       INTEGER NOT NULL DEFAULT 0,
    opening_date          TEXT NOT NULL,
    include_in_net_worth  INTEGER NOT NULL DEFAULT 1 CHECK (include_in_net_worth IN (0, 1)),
    archived              INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1)),
    color                 TEXT,
    notes                 TEXT,
    sort_order            INTEGER NOT NULL DEFAULT 0,
    created_at            TEXT NOT NULL,
    updated_at            TEXT NOT NULL,
    branch                TEXT,
    account_number        TEXT
) STRICT;

CREATE TABLE retirement_account (
    account_id  TEXT PRIMARY KEY REFERENCES account (id) ON DELETE CASCADE,
    -- Plano do catálogo do núcleo (`us_401k`, `us_ira`, `pt_ppr`, `es_plan_pensiones`…).
    plan        TEXT NOT NULL,
    -- HSA: `self` ou `family` (o limite muda); nos outros, `NULL`.
    coverage    TEXT CHECK (coverage IN ('self', 'family')),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE product_gtin (
    gtin        TEXT PRIMARY KEY CHECK (length(gtin) = 14),
    product_id  TEXT NOT NULL REFERENCES product (id) ON DELETE CASCADE,
    -- Como chegou: XML da nota (`cEAN`), código da loja com dígito certo, a pessoa, fonte aberta.
    origin      TEXT NOT NULL CHECK (origin IN ('xml', 'store_code', 'manual', 'open_data')),
    first_seen  TEXT,
    last_seen   TEXT,
    created_at  TEXT NOT NULL
) STRICT;

CREATE TABLE product_open_data (
    gtin           TEXT PRIMARY KEY CHECK (length(gtin) = 14),
    source         TEXT NOT NULL CHECK (source IN ('openfoodfacts')),
    found          INTEGER NOT NULL CHECK (found IN (0, 1)),
    fetched_at     TEXT NOT NULL,
    name           TEXT,
    brand          TEXT,
    quantity       TEXT,
    categories     TEXT,
    -- Grupo NOVA (1 a 4; 4 = ultraprocessado) e Nutri-Score (só como texto, sem o selo).
    nova           INTEGER CHECK (nova BETWEEN 1 AND 4),
    nutriscore     TEXT CHECK (nutriscore IN ('a', 'b', 'c', 'd', 'e')),
    per            TEXT CHECK (per IN ('100g', '100ml')),
    energy_kcal    REAL,
    sugars         REAL,
    added_sugars   REAL,
    saturated_fat  REAL,
    sodium_mg      REAL,
    protein        REAL,
    fiber          REAL,
    -- Etiquetas do Open Food Facts sem o prefixo de idioma, separadas por vírgula
    -- ("gluten,milk"; "vegan,organic").
    allergens      TEXT,
    traces         TEXT,
    labels         TEXT,
    -- Página do produto (crédito "ver o produto") e a última alteração do lado de lá.
    page_url       TEXT,
    source_updated TEXT
) STRICT;

CREATE TABLE member_diet (
    member_id    TEXT NOT NULL REFERENCES member (id) ON DELETE CASCADE,
    restriction  TEXT NOT NULL CHECK (restriction IN (
        'gluten', 'milk', 'eggs', 'peanuts', 'nuts', 'soybeans', 'fish', 'crustaceans',
        'sesame', 'vegan', 'vegetarian')),
    created_at   TEXT NOT NULL,
    PRIMARY KEY (member_id, restriction)
) STRICT;

CREATE TABLE pension_payment (
    id           TEXT PRIMARY KEY,
    asset_id     TEXT NOT NULL REFERENCES asset (id) ON DELETE CASCADE,
    account_id   TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    date         TEXT NOT NULL,
    amount       INTEGER NOT NULL CHECK (amount > 0),
    external_id  TEXT NOT NULL,
    trade_id     TEXT REFERENCES trade (id) ON DELETE CASCADE,
    source       TEXT,                 -- de onde veio (nome do print)
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    UNIQUE (asset_id, external_id)
) STRICT;

CREATE TABLE pension_payment_issue (
    id           TEXT PRIMARY KEY,
    asset_id     TEXT NOT NULL REFERENCES asset (id) ON DELETE CASCADE,
    kind         TEXT NOT NULL,
    date         TEXT,                 -- data lida, ou o mês (YYYY-MM) no mês que falta
    amount       INTEGER,
    source       TEXT,                 -- o print de onde veio
    message      TEXT NOT NULL,
    created_at   TEXT NOT NULL
) STRICT;

CREATE TABLE market_score (
    ticker               TEXT PRIMARY KEY,
    class                TEXT NOT NULL,          -- classe do ativo (stock_br, fii)
    status               TEXT NOT NULL,          -- ok, partial, insufficient_data, not_supported
    score                REAL,
    band                 TEXT,
    confidence           REAL,
    health               REAL,
    attractiveness       REAL,
    red_flags            TEXT NOT NULL DEFAULT '[]', -- JSON com as chaves das red flags
    segment              TEXT,
    piotroski            REAL,
    altman_z             REAL,
    beneish              REAL,
    momentum_12_1        REAL,
    volatility_1y        REAL,
    as_of                TEXT NOT NULL,
    methodology_version  TEXT NOT NULL,
    computed_at          TEXT NOT NULL
) STRICT;

CREATE TABLE screen_match (
    screen_id   TEXT NOT NULL REFERENCES screen (id) ON DELETE CASCADE,
    ticker      TEXT NOT NULL,
    first_seen  TEXT NOT NULL,  -- início do episódio atual
    last_seen   TEXT NOT NULL,
    left_at     TEXT,           -- saiu da tela (NULL enquanto cumpre)
    PRIMARY KEY (screen_id, ticker)
) STRICT;

CREATE TABLE asset_study_request (
    id                TEXT PRIMARY KEY,
    asset_id          TEXT NOT NULL REFERENCES asset (id) ON DELETE CASCADE,
    question          TEXT,
    include_position  INTEGER NOT NULL DEFAULT 0 CHECK (include_position IN (0, 1)),
    created_at        TEXT NOT NULL,
    started_at        TEXT,
    job_id            TEXT,
    error             TEXT
) STRICT;

CREATE TABLE education_plan (
    goal_id            TEXT PRIMARY KEY REFERENCES goal (id) ON DELETE CASCADE,
    member_id          TEXT REFERENCES member (id),             -- NULL: filho que ainda vai nascer
    expected_birth     TEXT,                                    -- AAAA-MM-DD, só sem membro
    start_age          INTEGER NOT NULL DEFAULT 18 CHECK (start_age BETWEEN 0 AND 60),
    start_month        INTEGER NOT NULL DEFAULT 2 CHECK (start_month BETWEEN 1 AND 12),
    start_date         TEXT,                                    -- sobrepõe idade + mês
    institution_type   TEXT NOT NULL CHECK (institution_type IN ('private', 'public', 'abroad')),
    course             TEXT,
    duration_months    INTEGER NOT NULL CHECK (duration_months BETWEEN 1 AND 180),
    payments_per_year  INTEGER NOT NULL DEFAULT 12 CHECK (payments_per_year IN (1, 2, 4, 12)),
    fee                INTEGER NOT NULL DEFAULT 0,              -- por parcela, preço de hoje
    fee_date           TEXT NOT NULL,                           -- data da cotação da mensalidade
    yearly_extras      INTEGER NOT NULL DEFAULT 0,
    monthly_living     INTEGER NOT NULL DEFAULT 0,
    discount_pct       REAL NOT NULL DEFAULT 0 CHECK (discount_pct BETWEEN 0 AND 100),
    inflation_mode     TEXT NOT NULL DEFAULT 'series_10y'
                       CHECK (inflation_mode IN ('series_10y', 'headline', 'fixed_spread')),
    inflation_series   TEXT REFERENCES index_series (id),       -- NULL: a do país
    fixed_spread       REAL,                                    -- fração ao ano
    real_return        REAL NOT NULL DEFAULT 0.03,              -- premissa da família
    contribution_mode  TEXT NOT NULL DEFAULT 'indexed' CHECK (contribution_mode IN ('indexed', 'flat')),
    pool_rule          TEXT NOT NULL DEFAULT 'earliest_first'
                       CHECK (pool_rule IN ('earliest_first', 'proportional')),
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL,
    CHECK (member_id IS NOT NULL OR expected_birth IS NOT NULL)
) STRICT;

CREATE TABLE goal_holding (
    id          TEXT PRIMARY KEY,
    goal_id     TEXT NOT NULL REFERENCES goal (id) ON DELETE CASCADE,
    account_id  TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    asset_id    TEXT REFERENCES asset (id) ON DELETE CASCADE,  -- NULL: a conta inteira
    share       REAL CHECK (share BETWEEN 0.01 AND 1),         -- NULL: pote comum
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL, value_mode TEXT NOT NULL DEFAULT 'net_redemption'
    CHECK (value_mode IN ('market', 'net_redemption')),
    UNIQUE (goal_id, account_id, asset_id)
) STRICT;

CREATE TABLE education_plan_review (
    id               TEXT PRIMARY KEY,
    goal_id          TEXT NOT NULL REFERENCES goal (id) ON DELETE CASCADE,
    reviewed_at      TEXT NOT NULL,
    reason           TEXT NOT NULL CHECK (reason IN ('annual', 'edit', 'fee_update', 'created')),
    spread           REAL NOT NULL,
    spread_from      TEXT,
    spread_to        TEXT,
    fee              INTEGER NOT NULL,
    target_at_start  INTEGER NOT NULL,
    saved            INTEGER NOT NULL,
    monthly_needed   INTEGER NOT NULL,
    assumptions      TEXT NOT NULL
) STRICT;

CREATE TABLE "country_series" (
    country    TEXT NOT NULL REFERENCES country_profile (country) ON DELETE CASCADE,
    role       TEXT NOT NULL CHECK (role IN ('inflation', 'rate', 'fx', 'education_inflation')),
    priority   INTEGER NOT NULL DEFAULT 0,
    series_id  TEXT NOT NULL REFERENCES index_series (id) ON DELETE CASCADE,
    PRIMARY KEY (country, role, series_id)
) STRICT;

CREATE TABLE "fixed_income_terms" (
    asset_id        TEXT PRIMARY KEY REFERENCES asset (id) ON DELETE CASCADE,
    kind            TEXT NOT NULL CHECK (kind IN ('cdb', 'rdb', 'lc', 'lci', 'lca', 'cri', 'cra', 'debenture', 'debenture_incentivada',
                                                  'tesouro_selic', 'tesouro_prefixado', 'tesouro_ipca', 'tesouro_renda',
                                                  'poupanca', 'other')),
    indexer         TEXT NOT NULL CHECK (indexer IN ('cdi', 'selic', 'ipca', 'igpm', 'prefixado')),
    rate            TEXT NOT NULL,          -- % do CDI/Selic (ex.: 110) ou taxa a.a. (prefixado 12,5; IPCA + 6,2)
    issue_date      TEXT,
    maturity_date   TEXT,
    grace_until     TEXT,
    liquidity       TEXT NOT NULL DEFAULT 'at_maturity' CHECK (liquidity IN ('daily', 'at_maturity', 'after_grace')),
    ir_exempt       INTEGER NOT NULL DEFAULT 0 CHECK (ir_exempt IN (0, 1)),
    issuer          TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
) STRICT;

CREATE TABLE dev_feedback_link (
    id           TEXT PRIMARY KEY,
    feedback_id  TEXT NOT NULL REFERENCES dev_feedback (id) ON DELETE CASCADE,
    related_id   TEXT NOT NULL REFERENCES dev_feedback (id) ON DELETE CASCADE,
    kind         TEXT NOT NULL CHECK (kind IN ('duplicado', 'depende_de', 'relacionado')),
    created_by   TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    CHECK (feedback_id <> related_id),
    UNIQUE (feedback_id, related_id)
) STRICT;

CREATE TABLE investment_plan (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    asset_id      TEXT NOT NULL REFERENCES asset (id) ON DELETE CASCADE,
    account_id    TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    amount        INTEGER NOT NULL CHECK (amount > 0),   -- valor de cada aporte, na moeda do ativo (menor unidade)
    day_of_month  INTEGER NOT NULL CHECK (day_of_month BETWEEN 1 AND 28),
    start_date    TEXT NOT NULL,
    end_date      TEXT,
    goal_id       TEXT REFERENCES goal (id) ON DELETE SET NULL,  -- meta para a qual o aporte vai (informativo)
    active        INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    notes         TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
) STRICT;

CREATE TABLE investment_plan_run (
    plan_id     TEXT NOT NULL REFERENCES investment_plan (id) ON DELETE CASCADE,
    due_date    TEXT NOT NULL,
    status      TEXT NOT NULL CHECK (status IN ('confirmed', 'skipped')),
    trade_id    TEXT REFERENCES trade (id) ON DELETE SET NULL,
    created_at  TEXT NOT NULL,
    PRIMARY KEY (plan_id, due_date)
) STRICT;

CREATE TABLE ai_mcp_server (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL UNIQUE,      -- prefixo das ferramentas (`celular__…`)
    label         TEXT NOT NULL,
    transport     TEXT NOT NULL CHECK (transport IN ('stdio', 'http')),
    command       TEXT,                      -- stdio
    args          TEXT NOT NULL DEFAULT '[]', -- stdio: JSON com a lista de argumentos
    url           TEXT,                      -- http
    env           TEXT NOT NULL DEFAULT '[]', -- JSON [{key, value, secret}] (secreto: value null)
    headers       TEXT NOT NULL DEFAULT '[]', -- JSON [{key, value, secret}] (secreto: value null)
    enabled       INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    auto_approve  INTEGER NOT NULL DEFAULT 0 CHECK (auto_approve IN (0, 1)),
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
) STRICT;

CREATE TABLE card_plastic (
    id          TEXT PRIMARY KEY,
    account_id  TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    kind        TEXT NOT NULL CHECK (kind IN ('holder', 'additional', 'virtual', 'temporary')),
    last4       TEXT CHECK (last4 IS NULL OR (length(last4) = 4 AND last4 GLOB '[0-9][0-9][0-9][0-9]')),
    member_id   TEXT REFERENCES member (id) ON DELETE SET NULL,
    label       TEXT,
    spend_limit INTEGER CHECK (spend_limit IS NULL OR spend_limit >= 0), -- sublimite informativo, por ciclo
    active      INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
) STRICT;

CREATE TABLE card_limit_change (
    id            TEXT PRIMARY KEY,
    account_id    TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    date          TEXT NOT NULL,
    credit_limit  INTEGER NOT NULL CHECK (credit_limit >= 0),
    source        TEXT NOT NULL DEFAULT 'manual' CHECK (source IN ('manual', 'import')),
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
) STRICT;

CREATE TABLE card_annual_fee (
    id                TEXT PRIMARY KEY,
    account_id        TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    amount            INTEGER NOT NULL CHECK (amount >= 0),          -- valor cheio do ano
    installments      INTEGER NOT NULL DEFAULT 12 CHECK (installments BETWEEN 1 AND 12),
    anniversary_month INTEGER CHECK (anniversary_month BETWEEN 1 AND 12),
    waiver_kind       TEXT NOT NULL DEFAULT 'none'
                      CHECK (waiver_kind IN ('none', 'full', 'spend', 'investment', 'discount')),
    waiver_threshold  INTEGER CHECK (waiver_threshold IS NULL OR waiver_threshold >= 0),
    waiver_discount   TEXT,                                          -- decimal: 0.5 = 50%
    starts_on         TEXT NOT NULL,
    ends_on           TEXT,
    notes             TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
) STRICT;

CREATE TABLE card_reward_program (
    id                     TEXT PRIMARY KEY,
    account_id             TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    kind                   TEXT NOT NULL CHECK (kind IN ('cashback', 'points')),
    base_rate              TEXT NOT NULL,
    rate_currency          TEXT,
    cycle_cap              INTEGER CHECK (cycle_cap IS NULL OR cycle_cap >= 0),
    -- Onde o cashback cai: crédito na fatura, numa conta ou num investimento.
    destination            TEXT NOT NULL DEFAULT 'statement'
                           CHECK (destination IN ('statement', 'account', 'investment')),
    destination_account_id TEXT REFERENCES account (id) ON DELETE SET NULL,
    -- Dias depois do fechamento em que o crédito costuma cair (para o aviso de "não veio").
    payout_days            INTEGER NOT NULL DEFAULT 40 CHECK (payout_days BETWEEN 0 AND 180),
    starts_on              TEXT NOT NULL,
    ends_on                TEXT,
    notes                  TEXT,
    created_at             TEXT NOT NULL,
    updated_at             TEXT NOT NULL
, loyalty_program_id TEXT REFERENCES loyalty_program (id) ON DELETE SET NULL) STRICT;

CREATE TABLE card_reward_multiplier (
    id           TEXT PRIMARY KEY,
    program_id   TEXT NOT NULL REFERENCES card_reward_program (id) ON DELETE CASCADE,
    category_id  TEXT NOT NULL REFERENCES category (id) ON DELETE CASCADE,
    rate         TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    UNIQUE (program_id, category_id)
) STRICT;

CREATE TABLE loyalty_program (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    kind            TEXT NOT NULL DEFAULT 'bank' CHECK (kind IN ('bank', 'airline', 'retail', 'other')),
    member_id       TEXT REFERENCES member (id) ON DELETE SET NULL,
    unit_label      TEXT,                 -- "pontos", "milhas"
    value_per_1000  INTEGER CHECK (value_per_1000 IS NULL OR value_per_1000 >= 0), -- premissa, centavos
    value_currency  TEXT,
    active          INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    notes           TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
) STRICT;

CREATE TABLE loyalty_entry (
    id             TEXT PRIMARY KEY,
    program_id     TEXT NOT NULL REFERENCES loyalty_program (id) ON DELETE CASCADE,
    date           TEXT NOT NULL,
    points         INTEGER NOT NULL CHECK (points <> 0),
    kind           TEXT NOT NULL CHECK (kind IN ('earn', 'bonus', 'purchase', 'transfer_in',
                                                 'transfer_out', 'redeem', 'expire', 'adjust')),
    expires_on     TEXT,
    statement_id   TEXT REFERENCES card_statement (id) ON DELETE SET NULL,
    transfer_id    TEXT,
    cost           INTEGER CHECK (cost IS NULL OR cost >= 0),
    cost_currency  TEXT,
    notes          TEXT,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
) STRICT;

CREATE TABLE goal_source (
    id          TEXT PRIMARY KEY,
    goal_id     TEXT NOT NULL REFERENCES goal (id) ON DELETE CASCADE,
    kind        TEXT NOT NULL CHECK (kind IN ('asset_tag')),
    ref_id      TEXT NOT NULL,
    value_mode  TEXT NOT NULL DEFAULT 'net_redemption'
                CHECK (value_mode IN ('market', 'net_redemption')),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    UNIQUE (goal_id, kind, ref_id)
) STRICT;

CREATE TABLE asset_label (
    id                      TEXT PRIMARY KEY,
    name                    TEXT NOT NULL UNIQUE COLLATE NOCASE,
    -- Grupo de concentração: o peso somado dos ativos da etiqueta é conferido contra o limite.
    is_concentration_group  INTEGER NOT NULL DEFAULT 0 CHECK (is_concentration_group IN (0, 1)),
    -- % da carteira; null = o padrão de grupo (ajuste concentration.group_default_pct, 30).
    concentration_limit_pct REAL CHECK (concentration_limit_pct IS NULL OR
                                        (concentration_limit_pct > 0 AND concentration_limit_pct <= 100)),
    created_at              TEXT NOT NULL,
    updated_at              TEXT NOT NULL
) STRICT;

CREATE TABLE asset_label_link (
    asset_id  TEXT NOT NULL REFERENCES asset (id) ON DELETE CASCADE,
    label_id  TEXT NOT NULL REFERENCES asset_label (id) ON DELETE CASCADE,
    PRIMARY KEY (asset_id, label_id)
) STRICT;

CREATE TABLE inbox_item (
    id             TEXT PRIMARY KEY,
    filename       TEXT NOT NULL,
    size           INTEGER NOT NULL,
    sha256         TEXT NOT NULL UNIQUE,
    -- `statement`, `brokerage_note`, `b3_report`, `pension_statement`, `receipt`, `image`, `unknown`
    detected_kind  TEXT NOT NULL,
    label          TEXT NOT NULL,
    status         TEXT NOT NULL CHECK (status IN ('pending', 'imported', 'dismissed', 'unrecognized')),
    note           TEXT,
    content        BLOB NOT NULL,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL
) STRICT;

CREATE TABLE detected_installment_ignore (
    series_key   TEXT PRIMARY KEY,
    account_id   TEXT REFERENCES account (id) ON DELETE CASCADE,
    description  TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
) STRICT;

-- Dados iniciais (catálogos)

-- setting
INSERT INTO setting (key, value, updated_at) VALUES ('mcp.require_api_key', 'true', '2026-11-21T13:01:26.565Z');

-- index_series
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('ipca', 'IPCA', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'bcb_sgs:433', NULL, '2026-11-21T13:01:26.365Z', '2026-11-21T13:01:26.365Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('igpm', 'IGP-M', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'bcb_sgs:189', NULL, '2026-11-21T13:01:26.365Z', '2026-11-21T13:01:26.365Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cdi', 'CDI', 'daily_rate', '% a.d.', 'daily', 'BR', 'bcb_sgs:12', NULL, '2026-11-21T13:01:26.365Z', '2026-11-21T13:01:26.365Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('selic', 'Selic diária', 'daily_rate', '% a.d.', 'daily', 'BR', 'bcb_sgs:11', NULL, '2026-11-21T13:01:26.365Z', '2026-11-21T13:01:26.365Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('selic_meta', 'Selic meta', 'annual_rate', '% a.a.', 'daily', 'BR', 'bcb_sgs:432', NULL, '2026-11-21T13:01:26.365Z', '2026-11-21T13:01:26.365Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('tr', 'TR', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'bcb_sgs:226', NULL, '2026-11-21T13:01:26.365Z', '2026-11-21T13:01:26.365Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('m2_br', 'M2 (meios de pagamento amplos)', 'level', 'R$ mil', 'monthly', 'BR', 'bcb_sgs:27810', NULL, '2026-11-21T13:01:26.365Z', '2026-11-21T13:01:26.365Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('us_cpi', 'US CPI-U', 'level', 'índice', 'monthly', 'US', 'fred:CPIAUCSL', NULL, '2026-11-21T13:01:26.365Z', '2026-11-21T13:01:26.365Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('us_m2', 'US M2', 'level', 'US$ bi', 'monthly', 'US', 'fred:M2SL', NULL, '2026-11-21T13:01:26.365Z', '2026-11-21T13:01:26.365Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fed_funds', 'Fed Funds', 'annual_rate', '% a.a.', 'monthly', 'US', 'fred:FEDFUNDS', NULL, '2026-11-21T13:01:26.365Z', '2026-11-21T13:01:26.365Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('ibov', 'Ibovespa', 'level', 'pontos', 'daily', 'BR', 'yahoo:^BVSP', NULL, '2026-11-21T13:01:26.365Z', '2026-11-21T13:01:26.365Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('sp500', 'S&P 500', 'level', 'pontos', 'daily', 'US', 'yahoo:^GSPC', NULL, '2026-11-21T13:01:26.365Z', '2026-11-21T13:01:26.365Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('incc', 'INCC', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'bcb_sgs:192', NULL, '2026-11-21T13:01:26.388Z', '2026-11-21T13:01:26.388Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_usd', 'EUR/USD (BCE)', 'level', 'USD por EUR', 'daily', 'EA', 'ecb:EXR/D.USD.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_jpy', 'EUR/JPY (BCE)', 'level', 'JPY por EUR', 'daily', 'EA', 'ecb:EXR/D.JPY.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_czk', 'EUR/CZK (BCE)', 'level', 'CZK por EUR', 'daily', 'EA', 'ecb:EXR/D.CZK.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_dkk', 'EUR/DKK (BCE)', 'level', 'DKK por EUR', 'daily', 'EA', 'ecb:EXR/D.DKK.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_huf', 'EUR/HUF (BCE)', 'level', 'HUF por EUR', 'daily', 'EA', 'ecb:EXR/D.HUF.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_pln', 'EUR/PLN (BCE)', 'level', 'PLN por EUR', 'daily', 'EA', 'ecb:EXR/D.PLN.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_ron', 'EUR/RON (BCE)', 'level', 'RON por EUR', 'daily', 'EA', 'ecb:EXR/D.RON.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_sek', 'EUR/SEK (BCE)', 'level', 'SEK por EUR', 'daily', 'EA', 'ecb:EXR/D.SEK.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_chf', 'EUR/CHF (BCE)', 'level', 'CHF por EUR', 'daily', 'EA', 'ecb:EXR/D.CHF.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_isk', 'EUR/ISK (BCE)', 'level', 'ISK por EUR', 'daily', 'EA', 'ecb:EXR/D.ISK.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_nok', 'EUR/NOK (BCE)', 'level', 'NOK por EUR', 'daily', 'EA', 'ecb:EXR/D.NOK.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_try', 'EUR/TRY (BCE)', 'level', 'TRY por EUR', 'daily', 'EA', 'ecb:EXR/D.TRY.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_aud', 'EUR/AUD (BCE)', 'level', 'AUD por EUR', 'daily', 'EA', 'ecb:EXR/D.AUD.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_brl', 'EUR/BRL (BCE)', 'level', 'BRL por EUR', 'daily', 'EA', 'ecb:EXR/D.BRL.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_cad', 'EUR/CAD (BCE)', 'level', 'CAD por EUR', 'daily', 'EA', 'ecb:EXR/D.CAD.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_cny', 'EUR/CNY (BCE)', 'level', 'CNY por EUR', 'daily', 'EA', 'ecb:EXR/D.CNY.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_hkd', 'EUR/HKD (BCE)', 'level', 'HKD por EUR', 'daily', 'EA', 'ecb:EXR/D.HKD.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_idr', 'EUR/IDR (BCE)', 'level', 'IDR por EUR', 'daily', 'EA', 'ecb:EXR/D.IDR.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_ils', 'EUR/ILS (BCE)', 'level', 'ILS por EUR', 'daily', 'EA', 'ecb:EXR/D.ILS.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_inr', 'EUR/INR (BCE)', 'level', 'INR por EUR', 'daily', 'EA', 'ecb:EXR/D.INR.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_krw', 'EUR/KRW (BCE)', 'level', 'KRW por EUR', 'daily', 'EA', 'ecb:EXR/D.KRW.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_mxn', 'EUR/MXN (BCE)', 'level', 'MXN por EUR', 'daily', 'EA', 'ecb:EXR/D.MXN.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_myr', 'EUR/MYR (BCE)', 'level', 'MYR por EUR', 'daily', 'EA', 'ecb:EXR/D.MYR.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_nzd', 'EUR/NZD (BCE)', 'level', 'NZD por EUR', 'daily', 'EA', 'ecb:EXR/D.NZD.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_php', 'EUR/PHP (BCE)', 'level', 'PHP por EUR', 'daily', 'EA', 'ecb:EXR/D.PHP.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_sgd', 'EUR/SGD (BCE)', 'level', 'SGD por EUR', 'daily', 'EA', 'ecb:EXR/D.SGD.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_thb', 'EUR/THB (BCE)', 'level', 'THB por EUR', 'daily', 'EA', 'ecb:EXR/D.THB.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_zar', 'EUR/ZAR (BCE)', 'level', 'ZAR por EUR', 'daily', 'EA', 'ecb:EXR/D.ZAR.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_gbp', 'EUR/GBP (BCE)', 'level', 'GBP por EUR', 'daily', 'EA', 'ecb:EXR/D.GBP.EUR.SP00.A', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('estr', '€STR (taxa de curto prazo do euro)', 'annual_rate', '% a.a.', 'daily', 'EA', 'ecb:EST/B.EU000A2X2A25.WT', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_ea', 'HICP zona do euro', 'level', 'índice (2025=100)', 'monthly', 'EA', 'ecb:HICP/M.U2.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_at', 'HICP Áustria', 'level', 'índice (2025=100)', 'monthly', 'AT', 'ecb:HICP/M.AT.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_be', 'HICP Bélgica', 'level', 'índice (2025=100)', 'monthly', 'BE', 'ecb:HICP/M.BE.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_bg', 'HICP Bulgária', 'level', 'índice (2025=100)', 'monthly', 'BG', 'ecb:HICP/M.BG.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_hr', 'HICP Croácia', 'level', 'índice (2025=100)', 'monthly', 'HR', 'ecb:HICP/M.HR.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_cy', 'HICP Chipre', 'level', 'índice (2025=100)', 'monthly', 'CY', 'ecb:HICP/M.CY.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_ee', 'HICP Estônia', 'level', 'índice (2025=100)', 'monthly', 'EE', 'ecb:HICP/M.EE.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_fi', 'HICP Finlândia', 'level', 'índice (2025=100)', 'monthly', 'FI', 'ecb:HICP/M.FI.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_fr', 'HICP França', 'level', 'índice (2025=100)', 'monthly', 'FR', 'ecb:HICP/M.FR.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_de', 'HICP Alemanha', 'level', 'índice (2025=100)', 'monthly', 'DE', 'ecb:HICP/M.DE.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_gr', 'HICP Grécia', 'level', 'índice (2025=100)', 'monthly', 'GR', 'ecb:HICP/M.GR.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_ie', 'HICP Irlanda', 'level', 'índice (2025=100)', 'monthly', 'IE', 'ecb:HICP/M.IE.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_it', 'HICP Itália', 'level', 'índice (2025=100)', 'monthly', 'IT', 'ecb:HICP/M.IT.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_lv', 'HICP Letônia', 'level', 'índice (2025=100)', 'monthly', 'LV', 'ecb:HICP/M.LV.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_lt', 'HICP Lituânia', 'level', 'índice (2025=100)', 'monthly', 'LT', 'ecb:HICP/M.LT.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_lu', 'HICP Luxemburgo', 'level', 'índice (2025=100)', 'monthly', 'LU', 'ecb:HICP/M.LU.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_mt', 'HICP Malta', 'level', 'índice (2025=100)', 'monthly', 'MT', 'ecb:HICP/M.MT.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_nl', 'HICP Países Baixos', 'level', 'índice (2025=100)', 'monthly', 'NL', 'ecb:HICP/M.NL.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_pt', 'HICP Portugal', 'level', 'índice (2025=100)', 'monthly', 'PT', 'ecb:HICP/M.PT.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_sk', 'HICP Eslováquia', 'level', 'índice (2025=100)', 'monthly', 'SK', 'ecb:HICP/M.SK.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_si', 'HICP Eslovênia', 'level', 'índice (2025=100)', 'monthly', 'SI', 'ecb:HICP/M.SI.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_es', 'HICP Espanha', 'level', 'índice (2025=100)', 'monthly', 'ES', 'ecb:HICP/M.ES.N.000000.4D0.INX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_cz', 'HICP Tchéquia', 'level', 'índice (2015=100)', 'monthly', 'CZ', 'eurostat:prc_hicp_minr?geo=CZ&unit=I15&coicop18=TOTAL', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_dk', 'HICP Dinamarca', 'level', 'índice (2015=100)', 'monthly', 'DK', 'eurostat:prc_hicp_minr?geo=DK&unit=I15&coicop18=TOTAL', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_hu', 'HICP Hungria', 'level', 'índice (2015=100)', 'monthly', 'HU', 'eurostat:prc_hicp_minr?geo=HU&unit=I15&coicop18=TOTAL', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_pl', 'HICP Polônia', 'level', 'índice (2015=100)', 'monthly', 'PL', 'eurostat:prc_hicp_minr?geo=PL&unit=I15&coicop18=TOTAL', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_ro', 'HICP Romênia', 'level', 'índice (2015=100)', 'monthly', 'RO', 'eurostat:prc_hicp_minr?geo=RO&unit=I15&coicop18=TOTAL', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_se', 'HICP Suécia', 'level', 'índice (2015=100)', 'monthly', 'SE', 'eurostat:prc_hicp_minr?geo=SE&unit=I15&coicop18=TOTAL', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_us_oecd', 'CPI Estados Unidos (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'US', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/USA.M.N.CPI.IX._T.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/USA.M.N.CPI.IX._T.N._Z', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_jp', 'CPI Japão (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'JP', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/JPN.M.N.CPI.IX._T.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/JPN.M.N.CPI.IX._T.N._Z', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_kr', 'CPI Coreia do Sul (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'KR', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/KOR.M.N.CPI.IX._T.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/KOR.M.N.CPI.IX._T.N._Z', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_mx', 'CPI México (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'MX', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/MEX.M.N.CPI.IX._T.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/MEX.M.N.CPI.IX._T.N._Z', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_cl', 'CPI Chile (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'CL', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/CHL.M.N.CPI.IX._T.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/CHL.M.N.CPI.IX._T.N._Z', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_co', 'CPI Colômbia (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'CO', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/COL.M.N.CPI.IX._T.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/COL.M.N.CPI.IX._T.N._Z', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_cr', 'CPI Costa Rica (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'CR', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/CRI.M.N.CPI.IX._T.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/CRI.M.N.CPI.IX._T.N._Z', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_il', 'CPI Israel (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'IL', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/ISR.M.N.CPI.IX._T.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/ISR.M.N.CPI.IX._T.N._Z', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_tr', 'CPI Turquia (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'TR', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/TUR.M.N.CPI.IX._T.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/TUR.M.N.CPI.IX._T.N._Z', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_au', 'CPI Austrália (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'AU', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/AUS.M.N.CPI.IX._T.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/AUS.M.N.CPI.IX._T.N._Z', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_ch', 'CPI Suíça (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'CH', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/CHE.M.N.CPI.IX._T.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/CHE.M.N.CPI.IX._T.N._Z', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_no', 'CPI Noruega (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'NO', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/NOR.M.N.CPI.IX._T.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/NOR.M.N.CPI.IX._T.N._Z', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_is', 'CPI Islândia (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'IS', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/ISL.M.N.CPI.IX._T.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/ISL.M.N.CPI.IX._T.N._Z', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_jp', 'Juros de 3 meses Japão (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'JP', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/JPN.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_kr', 'Juros de 3 meses Coreia do Sul (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'KR', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/KOR.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_mx', 'Juros de 3 meses México (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'MX', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/MEX.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_cl', 'Juros de 3 meses Chile (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'CL', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/CHL.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_co', 'Juros de 3 meses Colômbia (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'CO', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/COL.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_cr', 'Juros de 3 meses Costa Rica (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'CR', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/CRI.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_il', 'Juros de 3 meses Israel (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'IL', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/ISR.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_au', 'Juros de 3 meses Austrália (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'AU', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/AUS.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_ch', 'Juros de 3 meses Suíça (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'CH', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/CHE.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_no', 'Juros de 3 meses Noruega (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'NO', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/NOR.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_is', 'Juros de 3 meses Islândia (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'IS', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/ISL.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_cz', 'Juros de 3 meses Tchéquia (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'CZ', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/CZE.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_dk', 'Juros de 3 meses Dinamarca (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'DK', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/DNK.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_hu', 'Juros de 3 meses Hungria (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'HU', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/HUN.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_pl', 'Juros de 3 meses Polônia (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'PL', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/POL.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_se', 'Juros de 3 meses Suécia (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'SE', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/SWE.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('rate3m_nz', 'Juros de 3 meses Nova Zelândia (OCDE)', 'annual_rate', '% a.a.', 'monthly', 'NZ', 'oecd:OECD.SDD.STES,DSD_STES@DF_FINMARK,/NZL.M.IR3TIB.PA._Z._Z._Z._Z.N', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_ca', 'CPI Canadá', 'level', 'índice (2002=100)', 'monthly', 'CA', 'boc:V41690973', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('corra', 'CORRA (Canadá)', 'annual_rate', '% a.a.', 'daily', 'CA', 'boc:AVG.INTWO', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_usd_cad_boc', 'USD/CAD (Bank of Canada)', 'level', 'CAD por USD', 'daily', 'CA', 'boc:FXUSDCAD', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_eur_cad_boc', 'EUR/CAD (Bank of Canada)', 'level', 'CAD por EUR', 'daily', 'CA', 'boc:FXEURCAD', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_gbp_cad_boc', 'GBP/CAD (Bank of Canada)', 'level', 'CAD por GBP', 'daily', 'CA', 'boc:FXGBPCAD', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_brl_cad_boc', 'BRL/CAD (Bank of Canada)', 'level', 'CAD por BRL', 'daily', 'CA', 'boc:FXBRLCAD', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_gb', 'CPI Reino Unido (ONS)', 'level', 'índice (2015=100)', 'monthly', 'GB', 'ons:mm23/d7bt', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpih_gb', 'CPIH Reino Unido (ONS)', 'level', 'índice (2015=100)', 'monthly', 'GB', 'ons:mm23/l522', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('sonia', 'SONIA (Reino Unido)', 'annual_rate', '% a.a.', 'daily', 'GB', 'boe:IUDSOIA', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('boe_bank_rate', 'Bank Rate (Bank of England)', 'annual_rate', '% a.a.', 'daily', 'GB', 'boe:IUDBEDR', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_gbp_usd', 'GBP/USD (Bank of England)', 'level', 'USD por GBP', 'daily', 'GB', 'boe:XUDLUSS', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('fx_gbp_eur', 'GBP/EUR (Bank of England)', 'level', 'EUR por GBP', 'daily', 'GB', 'boe:XUDLERS', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_us', 'Inflação anual Estados Unidos (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'US', 'worldbank:US', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_at', 'Inflação anual Áustria (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'AT', 'worldbank:AT', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_be', 'Inflação anual Bélgica (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'BE', 'worldbank:BE', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_bg', 'Inflação anual Bulgária (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'BG', 'worldbank:BG', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_hr', 'Inflação anual Croácia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'HR', 'worldbank:HR', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_cy', 'Inflação anual Chipre (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'CY', 'worldbank:CY', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_ee', 'Inflação anual Estônia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'EE', 'worldbank:EE', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_fi', 'Inflação anual Finlândia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'FI', 'worldbank:FI', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_fr', 'Inflação anual França (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'FR', 'worldbank:FR', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_de', 'Inflação anual Alemanha (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'DE', 'worldbank:DE', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_gr', 'Inflação anual Grécia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'GR', 'worldbank:GR', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_ie', 'Inflação anual Irlanda (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'IE', 'worldbank:IE', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_it', 'Inflação anual Itália (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'IT', 'worldbank:IT', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_lv', 'Inflação anual Letônia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'LV', 'worldbank:LV', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_lt', 'Inflação anual Lituânia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'LT', 'worldbank:LT', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_lu', 'Inflação anual Luxemburgo (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'LU', 'worldbank:LU', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_mt', 'Inflação anual Malta (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'MT', 'worldbank:MT', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_nl', 'Inflação anual Países Baixos (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'NL', 'worldbank:NL', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_pt', 'Inflação anual Portugal (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'PT', 'worldbank:PT', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_sk', 'Inflação anual Eslováquia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'SK', 'worldbank:SK', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_si', 'Inflação anual Eslovênia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'SI', 'worldbank:SI', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_es', 'Inflação anual Espanha (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'ES', 'worldbank:ES', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_cz', 'Inflação anual Tchéquia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'CZ', 'worldbank:CZ', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_dk', 'Inflação anual Dinamarca (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'DK', 'worldbank:DK', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_hu', 'Inflação anual Hungria (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'HU', 'worldbank:HU', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_pl', 'Inflação anual Polônia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'PL', 'worldbank:PL', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_ro', 'Inflação anual Romênia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'RO', 'worldbank:RO', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_se', 'Inflação anual Suécia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'SE', 'worldbank:SE', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_jp', 'Inflação anual Japão (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'JP', 'worldbank:JP', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_kr', 'Inflação anual Coreia do Sul (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'KR', 'worldbank:KR', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_mx', 'Inflação anual México (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'MX', 'worldbank:MX', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_cl', 'Inflação anual Chile (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'CL', 'worldbank:CL', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_co', 'Inflação anual Colômbia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'CO', 'worldbank:CO', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_cr', 'Inflação anual Costa Rica (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'CR', 'worldbank:CR', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_il', 'Inflação anual Israel (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'IL', 'worldbank:IL', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_tr', 'Inflação anual Turquia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'TR', 'worldbank:TR', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_au', 'Inflação anual Austrália (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'AU', 'worldbank:AU', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_ch', 'Inflação anual Suíça (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'CH', 'worldbank:CH', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_no', 'Inflação anual Noruega (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'NO', 'worldbank:NO', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_is', 'Inflação anual Islândia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'IS', 'worldbank:IS', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_ca', 'Inflação anual Canadá (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'CA', 'worldbank:CA', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_gb', 'Inflação anual Reino Unido (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'GB', 'worldbank:GB', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_nz', 'Inflação anual Nova Zelândia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'NZ', 'worldbank:NZ', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_ar', 'Inflação anual Argentina (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'AR', 'worldbank:AR', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_uy', 'Inflação anual Uruguai (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'UY', 'worldbank:UY', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_py', 'Inflação anual Paraguai (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'PY', 'worldbank:PY', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_pe', 'Inflação anual Peru (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'PE', 'worldbank:PE', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_bo', 'Inflação anual Bolívia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'BO', 'worldbank:BO', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_ec', 'Inflação anual Equador (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'EC', 'worldbank:EC', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_cn', 'Inflação anual China (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'CN', 'worldbank:CN', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_in', 'Inflação anual Índia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'IN', 'worldbank:IN', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_id', 'Inflação anual Indonésia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'ID', 'worldbank:ID', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_za', 'Inflação anual África do Sul (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'ZA', 'worldbank:ZA', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_ao', 'Inflação anual Angola (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'AO', 'worldbank:AO', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_mz', 'Inflação anual Moçambique (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'MZ', 'worldbank:MZ', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_cv', 'Inflação anual Cabo Verde (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'CV', 'worldbank:CV', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_sg', 'Inflação anual Singapura (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'SG', 'worldbank:SG', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_ph', 'Inflação anual Filipinas (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'PH', 'worldbank:PH', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_th', 'Inflação anual Tailândia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'TH', 'worldbank:TH', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_my', 'Inflação anual Malásia (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'MY', 'worldbank:MY', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_ae', 'Inflação anual Emirados Árabes Unidos (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'AE', 'worldbank:AE', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('wb_cpi_sa', 'Inflação anual Arábia Saudita (Banco Mundial)', 'level', 'índice (estimado)', 'monthly', 'SA', 'worldbank:SA', NULL, '2026-11-21T13:01:26.390Z', '2026-11-21T13:01:26.390Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('inpc', 'INPC', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'bcb_sgs:188', NULL, '2026-11-21T13:01:26.413Z', '2026-11-21T13:01:26.413Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('poupanca', 'Poupança (rentabilidade)', 'monthly_rate', '% a.m.', 'daily', 'BR', 'bcb_sgs:195', NULL, '2026-11-21T13:01:26.413Z', '2026-11-21T13:01:26.413Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('ipca_g1', 'IPCA · Alimentação e bebidas', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'ibge_sidra:7060/7170', NULL, '2026-11-21T13:01:26.489Z', '2026-11-21T13:01:26.489Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('ipca_g2', 'IPCA · Habitação', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'ibge_sidra:7060/7445', NULL, '2026-11-21T13:01:26.489Z', '2026-11-21T13:01:26.489Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('ipca_g3', 'IPCA · Artigos de residência', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'ibge_sidra:7060/7486', NULL, '2026-11-21T13:01:26.489Z', '2026-11-21T13:01:26.489Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('ipca_g4', 'IPCA · Vestuário', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'ibge_sidra:7060/7558', NULL, '2026-11-21T13:01:26.489Z', '2026-11-21T13:01:26.489Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('ipca_g5', 'IPCA · Transportes', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'ibge_sidra:7060/7625', NULL, '2026-11-21T13:01:26.489Z', '2026-11-21T13:01:26.489Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('ipca_g6', 'IPCA · Saúde e cuidados pessoais', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'ibge_sidra:7060/7660', NULL, '2026-11-21T13:01:26.489Z', '2026-11-21T13:01:26.489Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('ipca_g7', 'IPCA · Despesas pessoais', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'ibge_sidra:7060/7712', NULL, '2026-11-21T13:01:26.489Z', '2026-11-21T13:01:26.489Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('ipca_g8', 'IPCA · Educação', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'ibge_sidra:7060/7766', NULL, '2026-11-21T13:01:26.489Z', '2026-11-21T13:01:26.489Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('ipca_g9', 'IPCA · Comunicação', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'ibge_sidra:7060/7786', NULL, '2026-11-21T13:01:26.489Z', '2026-11-21T13:01:26.489Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('us_effr', 'Fed Funds efetiva (NY Fed)', 'annual_rate', '% a.a.', 'daily', 'US', 'nyfed:effr', NULL, '2026-11-21T13:01:26.500Z', '2026-11-21T13:01:26.500Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('ipca_food_home', 'IPCA · Alimentação no domicílio', 'monthly_rate', '% a.m.', 'monthly', 'BR', 'ibge_sidra:7060/7171', NULL, '2026-11-21T13:01:26.564Z', '2026-11-21T13:01:26.564Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('us_cpi_food_home', 'US CPI · Food at home', 'level', 'índice (1982-84=100)', 'monthly', 'US', 'fred:CUSR0000SAF11', NULL, '2026-11-21T13:01:26.658Z', '2026-11-21T13:01:26.658Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_food_us_oecd', 'CPI Estados Unidos · Alimentos e bebidas não alcoólicas (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'US', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/USA.M.N.CPI.IX.CP01.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/USA.M.N.CPI.IX.CP01.N._Z', NULL, '2026-11-21T13:01:26.658Z', '2026-11-21T13:01:26.658Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_food_pt', 'HICP Portugal · Alimentos', 'level', 'índice (2025=100)', 'monthly', 'PT', 'ecb:HICP/M.PT.N.011000.4D0.INX', NULL, '2026-11-21T13:01:26.658Z', '2026-11-21T13:01:26.658Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_food_es', 'HICP Espanha · Alimentos', 'level', 'índice (2025=100)', 'monthly', 'ES', 'ecb:HICP/M.ES.N.011000.4D0.INX', NULL, '2026-11-21T13:01:26.658Z', '2026-11-21T13:01:26.658Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_food_fr', 'HICP França · Alimentos', 'level', 'índice (2025=100)', 'monthly', 'FR', 'ecb:HICP/M.FR.N.011000.4D0.INX', NULL, '2026-11-21T13:01:26.658Z', '2026-11-21T13:01:26.658Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('hicp_food_de', 'HICP Alemanha · Alimentos', 'level', 'índice (2025=100)', 'monthly', 'DE', 'ecb:HICP/M.DE.N.011000.4D0.INX', NULL, '2026-11-21T13:01:26.658Z', '2026-11-21T13:01:26.658Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_food_gb_oecd', 'CPI Reino Unido · Alimentos e bebidas não alcoólicas (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'GB', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/GBR.M.N.CPI.IX.CP01.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/GBR.M.N.CPI.IX.CP01.N._Z', NULL, '2026-11-21T13:01:26.658Z', '2026-11-21T13:01:26.658Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_food_ca_oecd', 'CPI Canadá · Alimentos e bebidas não alcoólicas (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'CA', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/CAN.M.N.CPI.IX.CP01.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/CAN.M.N.CPI.IX.CP01.N._Z', NULL, '2026-11-21T13:01:26.658Z', '2026-11-21T13:01:26.658Z');
INSERT INTO index_series (id, name, kind, unit, frequency, country, source, last_fetched_at, created_at, updated_at) VALUES ('cpi_food_cl_oecd', 'CPI Chile · Alimentos e bebidas não alcoólicas (OCDE)', 'level', 'índice (2015=100)', 'monthly', 'CL', 'oecd:OECD.SDD.TPS,DSD_PRICES_COICOP2018@DF_PRICES_C2018_ALL,/CHL.M.N.CPI.IX.CP01.N._Z|OECD.SDD.TPS,DSD_PRICES@DF_PRICES_ALL,/CHL.M.N.CPI.IX.CP01.N._Z', NULL, '2026-11-21T13:01:26.658Z', '2026-11-21T13:01:26.658Z');

-- ai_model_price
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('anthropic', '*', 15000000, 75000000, 1500000, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('anthropic', 'claude-opus', 15000000, 75000000, 1500000, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('anthropic', 'claude-sonnet', 3000000, 15000000, 300000, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('anthropic', 'claude-haiku', 1000000, 5000000, 100000, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('openai', '*', 10000000, 40000000, NULL, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('openai', 'gpt-5', 1250000, 10000000, 125000, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('openai', 'gpt-5-mini', 250000, 2000000, 25000, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('openai', 'gpt-4o', 2500000, 10000000, 1250000, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('openai', 'gpt-4o-mini', 150000, 600000, 75000, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('gemini', '*', 2500000, 15000000, NULL, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('gemini', 'gemini-2.5-pro', 1250000, 10000000, 310000, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('gemini', 'gemini-2.5-flash', 300000, 2500000, 75000, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('xai', '*', 3000000, 15000000, NULL, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('xai', 'grok-4', 3000000, 15000000, 750000, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('mistral', '*', 2000000, 6000000, NULL, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('mistral', 'mistral-large', 2000000, 6000000, NULL, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('mistral', 'mistral-small', 100000, 300000, NULL, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('openrouter', '*', 15000000, 75000000, NULL, 'default', '2026-10-14T00:00:00Z');
INSERT INTO ai_model_price (provider, model, input_micros_per_mtok, output_micros_per_mtok, cached_micros_per_mtok, source, updated_at) VALUES ('openai_compatible', '*', 10000000, 40000000, NULL, 'default', '2026-10-14T00:00:00Z');

-- country_profile
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('BR', 'Brasil', 'BRL', 'bcb_ptax', 'Comportamento de sempre: IPCA, CDI/Selic e PTAX do Banco Central.', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('US', 'Estados Unidos', 'USD', 'ecb', 'CPI-U da FRED exige a chave gratuita; sem ela, o CPI da OCDE (sem chave) é usado.', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'fifo');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('EA', 'Zona do euro', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('AT', 'Áustria', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('BE', 'Bélgica', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('BG', 'Bulgária', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('HR', 'Croácia', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('CY', 'Chipre', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('EE', 'Estônia', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('FI', 'Finlândia', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('FR', 'França', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('DE', 'Alemanha', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'fifo');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('GR', 'Grécia', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('IE', 'Irlanda', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('IT', 'Itália', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('LV', 'Letônia', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('LT', 'Lituânia', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('LU', 'Luxemburgo', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('MT', 'Malta', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('NL', 'Países Baixos', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('PT', 'Portugal', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'fifo');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('SK', 'Eslováquia', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('SI', 'Eslovênia', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('ES', 'Espanha', 'EUR', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'fifo');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('CZ', 'Tchéquia', 'CZK', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('DK', 'Dinamarca', 'DKK', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('HU', 'Hungria', 'HUF', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('PL', 'Polônia', 'PLN', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('RO', 'Romênia', 'RON', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('SE', 'Suécia', 'SEK', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('JP', 'Japão', 'JPY', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('KR', 'Coreia do Sul', 'KRW', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('MX', 'México', 'MXN', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('CL', 'Chile', 'CLP', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('CO', 'Colômbia', 'COP', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('CR', 'Costa Rica', 'CRC', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('IL', 'Israel', 'ILS', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('TR', 'Turquia', 'TRY', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('AU', 'Austrália', 'AUD', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('CH', 'Suíça', 'CHF', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('NO', 'Noruega', 'NOK', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('IS', 'Islândia', 'ISK', 'ecb', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('CA', 'Canadá', 'CAD', 'boc', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('GB', 'Reino Unido', 'GBP', 'boe', NULL, '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'uk_pool');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('NZ', 'Nova Zelândia', 'NZD', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('AR', 'Argentina', 'ARS', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('UY', 'Uruguai', 'UYU', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('PY', 'Paraguai', 'PYG', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('PE', 'Peru', 'PEN', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('BO', 'Bolívia', 'BOB', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('EC', 'Equador', 'USD', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('CN', 'China', 'CNY', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('IN', 'Índia', 'INR', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('ID', 'Indonésia', 'IDR', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('ZA', 'África do Sul', 'ZAR', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('AO', 'Angola', 'AOA', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('MZ', 'Moçambique', 'MZN', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('CV', 'Cabo Verde', 'CVE', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('SG', 'Singapura', 'SGD', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('PH', 'Filipinas', 'PHP', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('TH', 'Tailândia', 'THB', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('MY', 'Malásia', 'MYR', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('AE', 'Emirados Árabes Unidos', 'AED', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');
INSERT INTO country_profile (country, name, currency, fx_provider, notes, created_at, updated_at, cost_method) VALUES ('SA', 'Arábia Saudita', 'SAR', 'ecb', 'Inflação anual do Banco Mundial, interpolada mês a mês (estimada).', '2026-11-21T13:01:26.391Z', '2026-11-21T13:01:26.391Z', 'average');

-- exchange
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('B3', 'B3 (Brasil, Bolsa, Balcão)', 'BR', 'BRL', '.SA', 'BVMF', 'America/Sao_Paulo', '10:00', '18:00', NULL, NULL, '12345', 'SAO,BOVESPA,BMFBOVESPA,BM&FBOVESPA', 1, 'O fechamento muda entre 17h e 18h com o horário de verão dos EUA; o catálogo usa a janela maior.', '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('NYSE', 'New York Stock Exchange', 'US', 'USD', '', 'XNYS', 'America/New_York', '09:30', '16:00', NULL, NULL, '12345', 'NYQ', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('NASDAQ', 'Nasdaq', 'US', 'USD', '', 'XNAS', 'America/New_York', '09:30', '16:00', NULL, NULL, '12345', 'NMS,NGM,NCM,NASDAQGS,NASDAQGM,NASDAQCM', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('AMEX', 'NYSE American', 'US', 'USD', '', 'XASE', 'America/New_York', '09:30', '16:00', NULL, NULL, '12345', 'ASE,NYSE AMERICAN,NYSEAMERICAN', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('ARCA', 'NYSE Arca', 'US', 'USD', '', 'ARCX', 'America/New_York', '09:30', '16:00', NULL, NULL, '12345', 'PCX,NYSE ARCA,NYSEARCA', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('BATS', 'Cboe BZX', 'US', 'USD', '', 'BATS', 'America/New_York', '09:30', '16:00', NULL, NULL, '12345', 'BTS,CBOE', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('TSX', 'Toronto Stock Exchange', 'CA', 'CAD', '.TO', 'XTSE', 'America/Toronto', '09:30', '16:00', NULL, NULL, '12345', 'TOR', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('TSXV', 'TSX Venture Exchange', 'CA', 'CAD', '.V', 'XTSX', 'America/Toronto', '09:30', '16:00', NULL, NULL, '12345', 'VAN', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('BMV', 'Bolsa Mexicana de Valores', 'MX', 'MXN', '.MX', 'XMEX', 'America/Mexico_City', '08:30', '15:00', NULL, NULL, '12345', 'MEX', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('BYMA', 'Bolsas y Mercados Argentinos', 'AR', 'ARS', '.BA', 'XBUE', 'America/Argentina/Buenos_Aires', '11:00', '17:00', NULL, NULL, '12345', 'BUE', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('BCS', 'Bolsa de Santiago', 'CL', 'CLP', '.SN', 'XSGO', 'America/Santiago', '09:30', '16:00', NULL, NULL, '12345', 'SGO', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('LSE', 'London Stock Exchange', 'GB', 'GBP', '.L', 'XLON', 'Europe/London', '08:00', '16:30', NULL, NULL, '12345', 'LON', 1, 'O Yahoo cota em pence (GBp); o Treasury converte para libras.', '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('XETRA', 'Xetra (Deutsche Börse)', 'DE', 'EUR', '.DE', 'XETR', 'Europe/Berlin', '09:00', '17:30', NULL, NULL, '12345', 'GER,ETR', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('FWB', 'Börse Frankfurt', 'DE', 'EUR', '.F', 'XFRA', 'Europe/Berlin', '08:00', '22:00', NULL, NULL, '12345', 'FRA', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('EPA', 'Euronext Paris', 'FR', 'EUR', '.PA', 'XPAR', 'Europe/Paris', '09:00', '17:30', NULL, NULL, '12345', 'PAR,EURONEXT PARIS', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('AMS', 'Euronext Amsterdã', 'NL', 'EUR', '.AS', 'XAMS', 'Europe/Amsterdam', '09:00', '17:30', NULL, NULL, '12345', 'EURONEXT AMSTERDAM', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('EBR', 'Euronext Bruxelas', 'BE', 'EUR', '.BR', 'XBRU', 'Europe/Brussels', '09:00', '17:30', NULL, NULL, '12345', 'BRU,EURONEXT BRUSSELS', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('ELI', 'Euronext Lisboa', 'PT', 'EUR', '.LS', 'XLIS', 'Europe/Lisbon', '08:00', '16:30', NULL, NULL, '12345', 'LIS,EURONEXT LISBON', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('ISE', 'Euronext Dublin', 'IE', 'EUR', '.IR', 'XDUB', 'Europe/Dublin', '08:00', '16:30', NULL, NULL, '12345', 'ISE,EURONEXT DUBLIN', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('MIL', 'Euronext Milão (Borsa Italiana)', 'IT', 'EUR', '.MI', 'XMIL', 'Europe/Rome', '09:00', '17:30', NULL, NULL, '12345', 'BIT,BORSA ITALIANA', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('BME', 'Bolsa de Madri (BME)', 'ES', 'EUR', '.MC', 'XMAD', 'Europe/Madrid', '09:00', '17:30', NULL, NULL, '12345', 'MCE', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('SIX', 'SIX Swiss Exchange', 'CH', 'CHF', '.SW', 'XSWX', 'Europe/Zurich', '09:00', '17:30', NULL, NULL, '12345', 'EBS', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('VIE', 'Wiener Börse', 'AT', 'EUR', '.VI', 'XWBO', 'Europe/Vienna', '09:00', '17:30', NULL, NULL, '12345', '', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('STO', 'Nasdaq Estocolmo', 'SE', 'SEK', '.ST', 'XSTO', 'Europe/Stockholm', '09:00', '17:30', NULL, NULL, '12345', '', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('CPH', 'Nasdaq Copenhague', 'DK', 'DKK', '.CO', 'XCSE', 'Europe/Copenhagen', '09:00', '17:00', NULL, NULL, '12345', '', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('HEL', 'Nasdaq Helsinque', 'FI', 'EUR', '.HE', 'XHEL', 'Europe/Helsinki', '10:00', '18:30', NULL, NULL, '12345', '', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('OSL', 'Oslo Børs', 'NO', 'NOK', '.OL', 'XOSL', 'Europe/Oslo', '09:00', '16:20', NULL, NULL, '12345', '', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('WSE', 'Bolsa de Varsóvia', 'PL', 'PLN', '.WA', 'XWAR', 'Europe/Warsaw', '09:00', '17:00', NULL, NULL, '12345', 'WAR', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('BIST', 'Borsa Istanbul', 'TR', 'TRY', '.IS', 'XIST', 'Europe/Istanbul', '10:00', '18:00', NULL, NULL, '12345', 'IST', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('JSE', 'Johannesburg Stock Exchange', 'ZA', 'ZAR', '.JO', 'XJSE', 'Africa/Johannesburg', '09:00', '17:00', NULL, NULL, '12345', 'JNB', 1, 'O Yahoo cota em centavos (ZAc); o Treasury converte para rands.', '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('TASE', 'Bolsa de Tel Aviv', 'IL', 'ILS', '.TA', 'XTAE', 'Asia/Jerusalem', '09:59', '17:25', NULL, NULL, '12345', 'TLV', 1, 'O Yahoo cota em agorot (ILA); o Treasury converte para shekels.', '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('TADAWUL', 'Saudi Exchange (Tadawul)', 'SA', 'SAR', '.SR', 'XSAU', 'Asia/Riyadh', '10:00', '15:00', NULL, NULL, '71234', 'SAU', 1, 'Pregão de domingo a quinta.', '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('NSE', 'National Stock Exchange of India', 'IN', 'INR', '.NS', 'XNSE', 'Asia/Kolkata', '09:15', '15:30', NULL, NULL, '12345', 'NSI', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('BSE', 'BSE (Bombaim)', 'IN', 'INR', '.BO', 'XBOM', 'Asia/Kolkata', '09:15', '15:30', NULL, NULL, '12345', 'BOM', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('SSE', 'Bolsa de Xangai', 'CN', 'CNY', '.SS', 'XSHG', 'Asia/Shanghai', '09:30', '15:00', '11:30', '13:00', '12345', 'SHH', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('SZSE', 'Bolsa de Shenzhen', 'CN', 'CNY', '.SZ', 'XSHE', 'Asia/Shanghai', '09:30', '15:00', '11:30', '13:00', '12345', 'SHZ', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('HKEX', 'Bolsa de Hong Kong', 'HK', 'HKD', '.HK', 'XHKG', 'Asia/Hong_Kong', '09:30', '16:00', '12:00', '13:00', '12345', 'HKG', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('TSE', 'Bolsa de Tóquio (JPX)', 'JP', 'JPY', '.T', 'XTKS', 'Asia/Tokyo', '09:00', '15:30', '11:30', '12:30', '12345', 'JPX,TYO', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('KRX', 'Korea Exchange (KOSPI)', 'KR', 'KRW', '.KS', 'XKRX', 'Asia/Seoul', '09:00', '15:30', NULL, NULL, '12345', 'KSC', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('KOSDAQ', 'KOSDAQ', 'KR', 'KRW', '.KQ', 'XKOS', 'Asia/Seoul', '09:00', '15:30', NULL, NULL, '12345', 'KOE', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('TWSE', 'Bolsa de Taiwan', 'TW', 'TWD', '.TW', 'XTAI', 'Asia/Taipei', '09:00', '13:30', NULL, NULL, '12345', 'TAI', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('TPEX', 'Taipei Exchange', 'TW', 'TWD', '.TWO', 'ROCO', 'Asia/Taipei', '09:00', '13:30', NULL, NULL, '12345', 'TWO', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('SGX', 'Singapore Exchange', 'SG', 'SGD', '.SI', 'XSES', 'Asia/Singapore', '09:00', '17:00', '12:00', '13:00', '12345', 'SES', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('IDX', 'Bolsa da Indonésia', 'ID', 'IDR', '.JK', 'XIDX', 'Asia/Jakarta', '09:00', '16:00', '12:00', '13:30', '12345', 'JKT', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('SET', 'Bolsa da Tailândia', 'TH', 'THB', '.BK', 'XBKK', 'Asia/Bangkok', '10:00', '16:30', '12:30', '14:30', '12345', 'BKK', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('ASX', 'Australian Securities Exchange', 'AU', 'AUD', '.AX', 'XASX', 'Australia/Sydney', '10:00', '16:00', NULL, NULL, '12345', '', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');
INSERT INTO exchange (code, name, country, currency, yahoo_suffix, mic, timezone, open_time, close_time, lunch_start, lunch_end, trading_days, aliases, builtin, notes, created_at, updated_at) VALUES ('NZX', 'NZX (Nova Zelândia)', 'NZ', 'NZD', '.NZ', 'XNZE', 'Pacific/Auckland', '10:00', '16:45', NULL, NULL, '12345', 'NZE', 1, NULL, '2026-11-21T13:01:26.397Z', '2026-11-21T13:01:26.397Z');

-- country_series
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BR', 'inflation', 0, 'ipca');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BR', 'rate', 0, 'cdi');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BR', 'rate', 1, 'selic_meta');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('US', 'inflation', 0, 'us_cpi');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('US', 'inflation', 1, 'cpi_us_oecd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('US', 'inflation', 2, 'wb_cpi_us');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('US', 'rate', 1, 'fed_funds');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('US', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('US', 'fx', 1, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EA', 'inflation', 0, 'hicp_ea');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EA', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EA', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EA', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EA', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AT', 'inflation', 0, 'hicp_at');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AT', 'inflation', 1, 'wb_cpi_at');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AT', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AT', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AT', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AT', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BE', 'inflation', 0, 'hicp_be');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BE', 'inflation', 1, 'wb_cpi_be');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BE', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BE', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BE', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BE', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BG', 'inflation', 0, 'hicp_bg');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BG', 'inflation', 1, 'wb_cpi_bg');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BG', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BG', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BG', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BG', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('HR', 'inflation', 0, 'hicp_hr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('HR', 'inflation', 1, 'wb_cpi_hr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('HR', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('HR', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('HR', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('HR', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CY', 'inflation', 0, 'hicp_cy');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CY', 'inflation', 1, 'wb_cpi_cy');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CY', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CY', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CY', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CY', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EE', 'inflation', 0, 'hicp_ee');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EE', 'inflation', 1, 'wb_cpi_ee');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EE', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EE', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EE', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EE', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('FI', 'inflation', 0, 'hicp_fi');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('FI', 'inflation', 1, 'wb_cpi_fi');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('FI', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('FI', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('FI', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('FI', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('FR', 'inflation', 0, 'hicp_fr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('FR', 'inflation', 1, 'wb_cpi_fr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('FR', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('FR', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('FR', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('FR', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('DE', 'inflation', 0, 'hicp_de');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('DE', 'inflation', 1, 'wb_cpi_de');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('DE', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('DE', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('DE', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('DE', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('GR', 'inflation', 0, 'hicp_gr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('GR', 'inflation', 1, 'wb_cpi_gr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('GR', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('GR', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('GR', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('GR', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IE', 'inflation', 0, 'hicp_ie');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IE', 'inflation', 1, 'wb_cpi_ie');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IE', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IE', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IE', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IE', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IT', 'inflation', 0, 'hicp_it');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IT', 'inflation', 1, 'wb_cpi_it');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IT', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IT', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IT', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IT', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LV', 'inflation', 0, 'hicp_lv');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LV', 'inflation', 1, 'wb_cpi_lv');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LV', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LV', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LV', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LV', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LT', 'inflation', 0, 'hicp_lt');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LT', 'inflation', 1, 'wb_cpi_lt');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LT', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LT', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LT', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LT', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LU', 'inflation', 0, 'hicp_lu');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LU', 'inflation', 1, 'wb_cpi_lu');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LU', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LU', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LU', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('LU', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MT', 'inflation', 0, 'hicp_mt');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MT', 'inflation', 1, 'wb_cpi_mt');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MT', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MT', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MT', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MT', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NL', 'inflation', 0, 'hicp_nl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NL', 'inflation', 1, 'wb_cpi_nl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NL', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NL', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NL', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NL', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PT', 'inflation', 0, 'hicp_pt');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PT', 'inflation', 1, 'wb_cpi_pt');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PT', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PT', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PT', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PT', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SK', 'inflation', 0, 'hicp_sk');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SK', 'inflation', 1, 'wb_cpi_sk');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SK', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SK', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SK', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SK', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SI', 'inflation', 0, 'hicp_si');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SI', 'inflation', 1, 'wb_cpi_si');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SI', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SI', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SI', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SI', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ES', 'inflation', 0, 'hicp_es');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ES', 'inflation', 1, 'wb_cpi_es');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ES', 'rate', 0, 'estr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ES', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ES', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ES', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CZ', 'inflation', 0, 'hicp_cz');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CZ', 'inflation', 1, 'wb_cpi_cz');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CZ', 'rate', 0, 'rate3m_cz');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CZ', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CZ', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CZ', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CZ', 'fx', 3, 'fx_eur_czk');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('DK', 'inflation', 0, 'hicp_dk');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('DK', 'inflation', 1, 'wb_cpi_dk');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('DK', 'rate', 0, 'rate3m_dk');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('DK', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('DK', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('DK', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('DK', 'fx', 3, 'fx_eur_dkk');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('HU', 'inflation', 0, 'hicp_hu');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('HU', 'inflation', 1, 'wb_cpi_hu');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('HU', 'rate', 0, 'rate3m_hu');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('HU', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('HU', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('HU', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('HU', 'fx', 3, 'fx_eur_huf');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PL', 'inflation', 0, 'hicp_pl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PL', 'inflation', 1, 'wb_cpi_pl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PL', 'rate', 0, 'rate3m_pl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PL', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PL', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PL', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PL', 'fx', 3, 'fx_eur_pln');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('RO', 'inflation', 0, 'hicp_ro');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('RO', 'inflation', 1, 'wb_cpi_ro');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('RO', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('RO', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('RO', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('RO', 'fx', 3, 'fx_eur_ron');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SE', 'inflation', 0, 'hicp_se');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SE', 'inflation', 1, 'wb_cpi_se');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SE', 'rate', 0, 'rate3m_se');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SE', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SE', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SE', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SE', 'fx', 3, 'fx_eur_sek');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('JP', 'inflation', 0, 'cpi_jp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('JP', 'inflation', 1, 'wb_cpi_jp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('JP', 'rate', 0, 'rate3m_jp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('JP', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('JP', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('JP', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('JP', 'fx', 3, 'fx_eur_jpy');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('KR', 'inflation', 0, 'cpi_kr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('KR', 'inflation', 1, 'wb_cpi_kr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('KR', 'rate', 0, 'rate3m_kr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('KR', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('KR', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('KR', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('KR', 'fx', 3, 'fx_eur_krw');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MX', 'inflation', 0, 'cpi_mx');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MX', 'inflation', 1, 'wb_cpi_mx');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MX', 'rate', 0, 'rate3m_mx');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MX', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MX', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MX', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MX', 'fx', 3, 'fx_eur_mxn');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CL', 'inflation', 0, 'cpi_cl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CL', 'inflation', 1, 'wb_cpi_cl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CL', 'rate', 0, 'rate3m_cl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CL', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CL', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CL', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CO', 'inflation', 0, 'cpi_co');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CO', 'inflation', 1, 'wb_cpi_co');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CO', 'rate', 0, 'rate3m_co');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CO', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CO', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CO', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CR', 'inflation', 0, 'cpi_cr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CR', 'inflation', 1, 'wb_cpi_cr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CR', 'rate', 0, 'rate3m_cr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CR', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CR', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CR', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IL', 'inflation', 0, 'cpi_il');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IL', 'inflation', 1, 'wb_cpi_il');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IL', 'rate', 0, 'rate3m_il');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IL', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IL', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IL', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IL', 'fx', 3, 'fx_eur_ils');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('TR', 'inflation', 0, 'cpi_tr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('TR', 'inflation', 1, 'wb_cpi_tr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('TR', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('TR', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('TR', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('TR', 'fx', 3, 'fx_eur_try');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AU', 'inflation', 0, 'cpi_au');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AU', 'inflation', 1, 'wb_cpi_au');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AU', 'rate', 0, 'rate3m_au');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AU', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AU', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AU', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AU', 'fx', 3, 'fx_eur_aud');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CH', 'inflation', 0, 'cpi_ch');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CH', 'inflation', 1, 'wb_cpi_ch');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CH', 'rate', 0, 'rate3m_ch');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CH', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CH', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CH', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CH', 'fx', 3, 'fx_eur_chf');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NO', 'inflation', 0, 'cpi_no');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NO', 'inflation', 1, 'wb_cpi_no');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NO', 'rate', 0, 'rate3m_no');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NO', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NO', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NO', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NO', 'fx', 3, 'fx_eur_nok');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IS', 'inflation', 0, 'cpi_is');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IS', 'inflation', 1, 'wb_cpi_is');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IS', 'rate', 0, 'rate3m_is');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IS', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IS', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IS', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IS', 'fx', 3, 'fx_eur_isk');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CA', 'inflation', 0, 'cpi_ca');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CA', 'inflation', 1, 'wb_cpi_ca');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CA', 'rate', 0, 'corra');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CA', 'fx', 0, 'fx_usd_cad_boc');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CA', 'fx', 1, 'fx_eur_cad_boc');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CA', 'fx', 2, 'fx_gbp_cad_boc');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CA', 'fx', 3, 'fx_brl_cad_boc');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('GB', 'inflation', 0, 'cpi_gb');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('GB', 'inflation', 1, 'cpih_gb');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('GB', 'inflation', 2, 'wb_cpi_gb');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('GB', 'rate', 0, 'sonia');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('GB', 'rate', 1, 'boe_bank_rate');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('GB', 'fx', 0, 'fx_gbp_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('GB', 'fx', 1, 'fx_gbp_eur');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('GB', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NZ', 'inflation', 0, 'wb_cpi_nz');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NZ', 'rate', 0, 'rate3m_nz');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NZ', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NZ', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NZ', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('NZ', 'fx', 3, 'fx_eur_nzd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AR', 'inflation', 0, 'wb_cpi_ar');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AR', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AR', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AR', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('UY', 'inflation', 0, 'wb_cpi_uy');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('UY', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('UY', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('UY', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PY', 'inflation', 0, 'wb_cpi_py');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PY', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PY', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PY', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PE', 'inflation', 0, 'wb_cpi_pe');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PE', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PE', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PE', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BO', 'inflation', 0, 'wb_cpi_bo');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BO', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BO', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BO', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EC', 'inflation', 0, 'wb_cpi_ec');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EC', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EC', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('EC', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CN', 'inflation', 0, 'wb_cpi_cn');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CN', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CN', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CN', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CN', 'fx', 3, 'fx_eur_cny');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IN', 'inflation', 0, 'wb_cpi_in');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IN', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IN', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IN', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('IN', 'fx', 3, 'fx_eur_inr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ID', 'inflation', 0, 'wb_cpi_id');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ID', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ID', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ID', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ID', 'fx', 3, 'fx_eur_idr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ZA', 'inflation', 0, 'wb_cpi_za');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ZA', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ZA', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ZA', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('ZA', 'fx', 3, 'fx_eur_zar');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AO', 'inflation', 0, 'wb_cpi_ao');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AO', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AO', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AO', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MZ', 'inflation', 0, 'wb_cpi_mz');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MZ', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MZ', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MZ', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CV', 'inflation', 0, 'wb_cpi_cv');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CV', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CV', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('CV', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SG', 'inflation', 0, 'wb_cpi_sg');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SG', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SG', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SG', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SG', 'fx', 3, 'fx_eur_sgd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PH', 'inflation', 0, 'wb_cpi_ph');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PH', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PH', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PH', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('PH', 'fx', 3, 'fx_eur_php');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('TH', 'inflation', 0, 'wb_cpi_th');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('TH', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('TH', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('TH', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('TH', 'fx', 3, 'fx_eur_thb');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MY', 'inflation', 0, 'wb_cpi_my');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MY', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MY', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MY', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('MY', 'fx', 3, 'fx_eur_myr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AE', 'inflation', 0, 'wb_cpi_ae');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AE', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AE', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('AE', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SA', 'inflation', 0, 'wb_cpi_sa');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SA', 'fx', 0, 'fx_eur_usd');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SA', 'fx', 1, 'fx_eur_gbp');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('SA', 'fx', 2, 'fx_eur_brl');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('US', 'rate', 0, 'us_effr');
INSERT INTO country_series (country, role, priority, series_id) VALUES ('BR', 'education_inflation', 0, 'ipca_g8');

-- Índices
CREATE INDEX change_set_created ON change_set (created_at);
CREATE INDEX change_item_row ON change_item (table_name, row_id);
CREATE INDEX backup_record_created ON backup_record (created_at);
CREATE INDEX category_parent ON category (parent_id);
CREATE INDEX txn_account_date ON txn (account_id, date);
CREATE INDEX txn_date ON txn (date);
CREATE INDEX txn_category ON txn (category_id);
CREATE INDEX txn_transfer ON txn (transfer_id) WHERE transfer_id IS NOT NULL;
CREATE INDEX txn_statement ON txn (statement_id) WHERE statement_id IS NOT NULL;
CREATE INDEX txn_fingerprint ON txn (account_id, fingerprint);
CREATE UNIQUE INDEX txn_external ON txn (account_id, external_id) WHERE external_id IS NOT NULL;
CREATE INDEX txn_split_txn ON txn_split (txn_id);
CREATE INDEX attachment_txn ON attachment (txn_id);
CREATE INDEX txn_tag_tag ON txn_tag (tag_id);
CREATE INDEX txn_split_category ON txn_split (category_id) WHERE category_id IS NOT NULL;
CREATE INDEX txn_recurrence ON txn (recurrence_id, date) WHERE recurrence_id IS NOT NULL;
CREATE INDEX txn_installment ON txn (installment_plan_id, installment_number) WHERE installment_plan_id IS NOT NULL;
CREATE INDEX ai_audit_log_created ON ai_audit_log (created_at);
CREATE INDEX ai_audit_log_client ON ai_audit_log (client, created_at);
CREATE INDEX trade_asset_date ON trade (asset_id, date);
CREATE INDEX trade_account ON trade (account_id);
CREATE UNIQUE INDEX trade_external ON trade (account_id, external_id) WHERE external_id IS NOT NULL;
CREATE INDEX corporate_action_asset ON corporate_action (asset_id, date);
CREATE INDEX income_event_asset ON income_event (asset_id, pay_date);
CREATE UNIQUE INDEX darf_period ON darf (coalesce(member_id, ''), period, revenue_code) WHERE status <> 'cancelled';
CREATE INDEX research_security_issuer ON research_security (issuer_id);
CREATE INDEX corporate_document_issuer ON corporate_document (issuer_id, delivered_at);
CREATE INDEX corporate_document_date ON corporate_document (delivered_at);
CREATE INDEX research_quote_date ON research_quote (date);
CREATE UNIQUE INDEX screen_name ON screen (lower(name));
CREATE INDEX ai_usage_created ON ai_usage (created_at);
CREATE INDEX ai_usage_provider ON ai_usage (provider, created_at);
CREATE INDEX ai_conversation_updated ON ai_conversation (updated_at);
CREATE UNIQUE INDEX alert_dedupe ON alert (dedupe_key) WHERE dedupe_key IS NOT NULL;
CREATE INDEX alert_status ON alert (status, created_at);
CREATE INDEX loan_installment_due ON loan_installment (loan_id, due_date);
CREATE INDEX physical_acquisition_cost_asset ON physical_acquisition_cost (asset_id);
CREATE INDEX physical_improvement_asset ON physical_improvement (asset_id, date);
CREATE INDEX physical_valuation_asset ON physical_valuation (asset_id, date);
CREATE UNIQUE INDEX asset_market_ticker ON asset (market, ticker) WHERE ticker IS NOT NULL;
CREATE INDEX ai_procedure_links ON ai_procedure (account_id, institution_id, asset_id);
CREATE INDEX ai_attachment_message ON ai_attachment (message_id, position);
CREATE INDEX ai_attachment_conversation ON ai_attachment (conversation_id);
CREATE INDEX ai_attachment_expires ON ai_attachment (expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX bank_registry_code ON bank_registry (code);
CREATE INDEX balance_sync_account ON balance_sync (account_id, date);
CREATE INDEX fx_operation_date ON fx_operation (date);
CREATE INDEX dev_feedback_rank ON dev_feedback (status, importance DESC, last_bumped_at DESC);
CREATE INDEX dev_feedback_note_by_feedback ON dev_feedback_note (feedback_id, created_at);
CREATE INDEX bank_institution_cnpj8 ON bank_institution (cnpj8);
CREATE INDEX bank_prudential_period ON bank_prudential (period, item);
CREATE INDEX score_snapshot_asset ON score_snapshot (asset_id, as_of);
CREATE INDEX score_event_asset ON score_event (asset_id, status);
CREATE INDEX score_event_ticker ON score_event (ticker, status);
CREATE INDEX score_feedback_subject ON score_feedback (subject, created_at);
CREATE INDEX research_dividend_ticker ON research_dividend (ticker, ex_date);
CREATE INDEX project_physical_asset ON project (physical_asset_id) WHERE physical_asset_id IS NOT NULL;
CREATE INDEX txn_project ON txn (project_id) WHERE project_id IS NOT NULL;
CREATE INDEX project_item_project ON project_item (project_id, date);
CREATE INDEX ai_proposal_status ON ai_proposal (status, created_at);
CREATE INDEX tax_income_report_year ON tax_income_report (year);
CREATE INDEX ai_memory_link_entity ON ai_memory_link (entity_kind, entity_id);
CREATE INDEX corporate_action_new_asset ON corporate_action (new_asset_id)
    WHERE new_asset_id IS NOT NULL;
CREATE INDEX card_statement_due ON card_statement (due_date);
CREATE INDEX credit_rate_modality ON credit_rate (modality, period_start);
CREATE INDEX income_source_member ON income_source (member_id);
CREATE INDEX payslip_month ON payslip (reference_month);
CREATE INDEX txn_link_target ON txn_link (target_kind, target_id);
CREATE INDEX receipt_item_receipt ON receipt_item (receipt_id);
CREATE INDEX receipt_item_group ON receipt_item (item_group);
CREATE INDEX mail_rule_account ON mail_rule (account_id, priority);
CREATE INDEX mail_message_account ON mail_message (account_id, status);
CREATE UNIQUE INDEX mail_message_msgid ON mail_message (account_id, message_id) WHERE message_id IS NOT NULL;
CREATE INDEX mail_part_message ON mail_part (message_id);
CREATE INDEX mail_part_sha ON mail_part (sha256);
CREATE INDEX mail_part_dedup ON mail_part (dedup_key);
CREATE INDEX mail_audit_account ON mail_audit (account_id, at);
CREATE INDEX property_tax_installment_due ON property_tax_installment (due_date);
CREATE INDEX property_tax_installment_txn ON property_tax_installment (txn_id);
CREATE INDEX asset_observation_field ON asset_observation (asset_id, field, observed_at);
CREATE INDEX asset_study_asset ON asset_study (asset_id, created_at);
CREATE UNIQUE INDEX fgts_account_hash ON fgts_account (account_hash) WHERE account_hash IS NOT NULL;
CREATE INDEX fgts_entry_account ON fgts_entry (account_id, kind);
CREATE INDEX product_category_parent ON product_category (parent_id);
CREATE INDEX product_category_idx ON product (category_id);
CREATE INDEX product_alias_product ON product_alias (product_id);
CREATE INDEX receipt_item_product ON receipt_item (product_id);
CREATE INDEX receipt_payment_txn ON receipt_payment (txn_id);
CREATE INDEX receipt_date ON receipt (date);
CREATE INDEX receipt_txn ON receipt (txn_id);
CREATE INDEX product_gtin_product ON product_gtin (product_id);
CREATE INDEX pension_payment_pending ON pension_payment (asset_id) WHERE trade_id IS NULL;
CREATE INDEX pension_payment_issue_asset ON pension_payment_issue (asset_id);
CREATE INDEX ai_conversation_asset ON ai_conversation (asset_id) WHERE asset_id IS NOT NULL;
CREATE INDEX asset_study_request_open ON asset_study_request (created_at) WHERE started_at IS NULL;
CREATE INDEX education_plan_member ON education_plan (member_id);
CREATE INDEX goal_holding_goal ON goal_holding (goal_id);
CREATE INDEX goal_holding_account ON goal_holding (account_id);
CREATE INDEX goal_holding_asset ON goal_holding (asset_id);
CREATE INDEX education_plan_review_goal ON education_plan_review (goal_id, reviewed_at);
CREATE INDEX country_series_series ON country_series (series_id);
CREATE INDEX dev_feedback_link_related ON dev_feedback_link (related_id);
CREATE INDEX investment_plan_asset ON investment_plan (asset_id);
CREATE UNIQUE INDEX card_plastic_last4 ON card_plastic (account_id, last4) WHERE last4 IS NOT NULL;
CREATE INDEX card_plastic_account ON card_plastic (account_id);
CREATE INDEX txn_card_plastic ON txn (card_plastic_id) WHERE card_plastic_id IS NOT NULL;
CREATE INDEX card_limit_change_account ON card_limit_change (account_id, date);
CREATE INDEX card_annual_fee_account ON card_annual_fee (account_id, starts_on);
CREATE INDEX card_reward_program_account ON card_reward_program (account_id, starts_on);
CREATE INDEX loyalty_entry_program ON loyalty_entry (program_id, date);
CREATE INDEX loyalty_entry_statement ON loyalty_entry (statement_id) WHERE statement_id IS NOT NULL;
CREATE INDEX goal_source_goal ON goal_source (goal_id);
CREATE INDEX asset_label_link_label ON asset_label_link (label_id);
CREATE INDEX inbox_item_status ON inbox_item (status, created_at);
