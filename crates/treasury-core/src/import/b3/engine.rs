//! Comandos `b3.detect`, `b3.preview` e `b3.commit`.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::apply::{self, ApplyOpts, Approval, ItemOutcome, Outcome};
use super::parse::{self, Parsed};
use super::plan::{self, Op, Plan};
use super::{
    B3ActionProposal, B3AssetProposal, B3CommitArgs, B3CommitError, B3CommitResult, B3Counts,
    B3DetectResult, B3File, B3FileSummary, B3IncomeProposal, B3Preview, B3PreviewArgs,
    B3ReportKind, B3RowStatus, B3TradeOrigin, B3TradeProposal, sheet,
};
use crate::api::{Actor, Ctx};
use crate::backup::BackupKind;
use crate::changes::Author;
use crate::error::{Error, Result};
use crate::i18n::{t, t_args};
use crate::ids::new_id;
use crate::import::engine::{decode_base64, sha256_hex};
use crate::portfolio::records::{dec, to_minor};
use crate::portfolio::{IncomeKind, TradeSide};
use crate::time::{fmt_date, parse_date, today_local};
use rusqlite::OptionalExtension;

/// Prévias guardadas por sessão (cada uma guarda o plano inteiro).
const MAX_PREVIEWS: usize = 4;
/// Arquivos por prévia.
const MAX_FILES: usize = 20;
const SCRATCH_KEY: &str = "b3";

#[derive(Clone)]
struct Stored {
    token: String,
    created: Instant,
    filenames: Vec<String>,
    plan: Plan,
}

#[derive(Default)]
struct B3Scratch {
    previews: Vec<Stored>,
}

fn read_file(index: usize, f: &B3File) -> Result<(Vec<u8>, sheet::Workbook, Parsed)> {
    let filename = f.filename.trim();
    if filename.is_empty() {
        return Err(Error::invalid(t("nucleo-lancamentos-erro-arquivo-nome")));
    }
    let bytes = decode_base64(&f.content_base64)?;
    let wb = sheet::read(&bytes, filename)?;
    let parsed = parse::parse_workbook(index, filename, &wb);
    Ok((bytes, wb, parsed))
}

fn dates(p: &Parsed) -> Vec<NaiveDate> {
    p.trades
        .iter()
        .map(|t| t.date)
        .chain(p.moves.iter().map(|m| m.date))
        .collect()
}

/// `b3.detect`: tipo de relatório de cada aba, instituições e período.
pub fn detect(args: &B3File) -> Result<B3DetectResult> {
    let (_, wb, parsed) = read_file(0, args)?;
    let mut institutions: Vec<String> = vec![];
    let mut seen = HashSet::new();
    let names = parsed
        .trades
        .iter()
        .map(|t| &t.institution)
        .chain(parsed.moves.iter().map(|m| &m.institution))
        .chain(parsed.positions.iter().map(|p| &p.institution));
    for n in names {
        if !n.is_empty() && seen.insert(crate::import::text::fold(n)) {
            institutions.push(n.clone());
        }
    }
    let ds = dates(&parsed);
    Ok(B3DetectResult {
        filename: args.filename.trim().to_string(),
        format: wb.format,
        encoding: wb.encoding.clone(),
        kinds: parsed.kinds(),
        sheets: parsed.sheets,
        institutions,
        period_from: ds.iter().min().map(|d| fmt_date(*d)),
        period_to: ds.iter().max().map(|d| fmt_date(*d)),
        warnings: parsed.warnings,
    })
}

/// Data no nome do arquivo de posição ("posicao-2026-10-14.xlsx", "posicao_14-10-2026").
fn date_in_filename(name: &str) -> Option<NaiveDate> {
    let ymd = regex::Regex::new(r"(\d{4})[-_.](\d{2})[-_.](\d{2})").expect("regex");
    if let Some(c) = ymd.captures(name) {
        return NaiveDate::from_ymd_opt(c[1].parse().ok()?, c[2].parse().ok()?, c[3].parse().ok()?);
    }
    let dmy = regex::Regex::new(r"(\d{2})[-_.](\d{2})[-_.](\d{4})").expect("regex");
    let c = dmy.captures(name)?;
    NaiveDate::from_ymd_opt(c[3].parse().ok()?, c[2].parse().ok()?, c[1].parse().ok()?)
}

/// `b3.preview`: lê os relatórios, monta o plano, simula tudo numa transação desfeita e
/// guarda a prévia na sessão.
pub fn preview(ctx: &mut Ctx<'_>, args: &B3PreviewArgs) -> Result<B3Preview> {
    if args.files.is_empty() {
        return Err(Error::invalid(t("nucleo-importacao-b3-erro-sem-arquivo")));
    }
    if args.files.len() > MAX_FILES {
        return Err(Error::invalid(t_args(
            "nucleo-importacao-b3-erro-maximo-arquivos",
            &crate::i18n::args(&[("max", MAX_FILES.into())]),
        )));
    }
    let mut parsed = Parsed::default();
    let mut files = vec![];
    let mut position_date = None;
    for (i, f) in args.files.iter().enumerate() {
        let (bytes, wb, p) = read_file(i, f)?;
        let kinds = p.kinds();
        if kinds.contains(&B3ReportKind::Position) && position_date.is_none() {
            position_date = date_in_filename(&f.filename);
        }
        files.push(B3FileSummary {
            filename: f.filename.trim().to_string(),
            sha256: sha256_hex(&bytes),
            format: wb.format,
            kinds,
        });
        parsed.extend(p);
    }
    if parsed.trades.is_empty()
        && parsed.moves.is_empty()
        && parsed.positions.is_empty()
        && parsed.bad.is_empty()
    {
        return Err(Error::invalid(t("nucleo-importacao-b3-erro-nenhuma-linha")));
    }
    let position_date = match &args.position_date {
        Some(d) => parse_date(d)?,
        None => position_date.unwrap_or_else(today_local),
    };
    let first_date = dates(&parsed).into_iter().min();

    // Simulação: as mesmas funções do commit, numa transação que é desfeita. The accounts
    // the user asked to create exist only inside it (placeholders, created again on commit).
    let (plan, outcome, check) = {
        let tx = ctx.db.conn_mut().transaction()?;
        let mut choices = args.accounts.clone().unwrap_or_default();
        let mut new_accounts = vec![];
        for c in &mut choices {
            if let Some(n) = &c.new_account {
                let id = create_account(&tx, n, first_date)?;
                c.account_id = Some(id.clone());
                new_accounts.push((id, n.clone()));
            }
        }
        let mut plan = plan::build(
            &tx,
            &parsed,
            &choices,
            args.openings.as_deref().unwrap_or_default(),
            position_date,
        )?;
        plan.new_accounts = new_accounts;
        let before = summary_before(&tx, &plan)?;
        let outcome = apply::run(&tx, &plan, &ApplyOpts::default())?;
        let check = if plan.positions.is_empty() {
            None
        } else {
            Some(apply::position_check(&tx, &plan, &outcome, position_date)?)
        };
        let end = plan
            .period
            .1
            .map_or(position_date, |d| d.max(position_date));
        summarize_accounts(&tx, &mut plan, &outcome, &before, end)?;
        tx.rollback()?;
        (plan, outcome, check)
    };

    let token = new_id();
    let mut warnings = plan.warnings.clone();
    let pending_sales = outcome.items.values().filter(|o| o.oversold).count();
    if pending_sales > 0 {
        warnings.push(t_args(
            "nucleo-importacao-b3-aviso-vendas-maiores",
            &crate::i18n::args(&[("n", pending_sales.into())]),
        ));
    }
    let mut preview = assemble(&plan, &outcome, token.clone(), files, warnings);
    preview.position_check = check;
    let store = ctx.scratch.get_mut::<B3Scratch>(SCRATCH_KEY);
    store.previews.push(Stored {
        token,
        created: Instant::now(),
        filenames: preview.files.iter().map(|f| f.filename.clone()).collect(),
        plan,
    });
    if store.previews.len() > MAX_PREVIEWS {
        store.previews.sort_by_key(|p| p.created);
        let excess = store.previews.len() - MAX_PREVIEWS;
        store.previews.drain(..excess);
    }
    Ok(preview)
}

/// Creates an account for a custodian (in the preview's simulation or in the commit).
fn create_account(
    tx: &rusqlite::Transaction<'_>,
    n: &super::B3NewAccount,
    first_date: Option<NaiveDate>,
) -> Result<crate::ids::Id> {
    let name = n.name.trim();
    if name.is_empty() {
        return Err(Error::invalid(t("nucleo-importacao-b3-erro-conta-nome")));
    }
    let institution = n.institution_name.trim();
    let inst_id = if institution.is_empty() {
        None
    } else {
        let found: Option<String> = tx
            .query_row(
                "SELECT id FROM institution WHERE lower(name) = lower(?1) ORDER BY created_at LIMIT 1",
                [institution],
                |r| r.get(0),
            )
            .optional()?;
        Some(match found {
            Some(id) => id,
            None => {
                crate::household::create_institution(
                    tx,
                    &crate::household::InstitutionInput {
                        name: institution.to_string(),
                        kind: Some(n.institution_kind),
                        code: None,
                        website: None,
                    },
                )?
                .id
            }
        })
    };
    // The opening position, when there is one, is the day before the file.
    let opening = match &n.opening_date {
        Some(d) => Some(parse_date(d)?),
        None => first_date.map(|d| d - chrono::Duration::days(1)),
    };
    let today = today_local();
    let input: crate::accounts::AccountInput = serde_json::from_value(serde_json::json!({
        "name": name,
        "kind": n.kind,
        "institution_id": inst_id,
        "opening_date": opening.map(fmt_date),
    }))
    .map_err(|e| Error::invalid(e.to_string()))?;
    crate::accounts::create(tx, &input, today)
}

/// Quantity of each (account, asset) of the plan before the import, for the summary.
fn summary_before(
    tx: &rusqlite::Transaction<'_>,
    plan: &Plan,
) -> Result<HashMap<(crate::ids::Id, usize), Decimal>> {
    let mut out = HashMap::new();
    for it in &plan.items {
        let Some(acc) = &it.account else { continue };
        if out.contains_key(&(acc.clone(), it.asset)) {
            continue;
        }
        let q = match &plan.assets[it.asset].existing {
            Some(id) => crate::portfolio::engine::quantity_held(
                tx,
                id,
                Some(acc),
                &fmt_date(it.date - chrono::Duration::days(1)),
            )
            .unwrap_or_default(),
            None => Decimal::ZERO,
        };
        out.insert((acc.clone(), it.asset), q);
    }
    Ok(out)
}

/// What the import does in each custodian's account: rows written, period and the
/// position of each asset before → after.
fn summarize_accounts(
    tx: &rusqlite::Transaction<'_>,
    plan: &mut Plan,
    out: &Outcome,
    before: &HashMap<(crate::ids::Id, usize), Decimal>,
    end: NaiveDate,
) -> Result<()> {
    for inst in &mut plan.institutions {
        let Some(acc) = inst.account_id.clone() else {
            continue;
        };
        let mut s = super::B3InstitutionSummary::default();
        let mut assets: Vec<usize> = vec![];
        let mut dates: Vec<NaiveDate> = vec![];
        for it in &plan.items {
            let in_account = it.account.as_ref() == Some(&acc)
                || matches!(&it.op, Op::Action { legs, .. } if legs.iter().any(|l| l.account.as_ref() == Some(&acc)));
            if !in_account {
                continue;
            }
            let written = out.items.get(&it.index).is_some_and(|o| o.written);
            if !written {
                continue;
            }
            match &it.op {
                Op::Trade { .. } => s.trades += 1,
                Op::Income { .. } => s.incomes += 1,
                Op::Action { .. } => s.actions += 1,
                Op::Auction { .. } => {
                    if out.items.get(&it.index).is_some_and(|o| o.auction_income) {
                        s.incomes += 1
                    } else {
                        s.trades += 1
                    }
                }
            }
            dates.push(it.date);
            if !assets.contains(&it.asset) {
                assets.push(it.asset);
            }
        }
        s.period_from = dates.iter().min().map(|d| fmt_date(*d));
        s.period_to = dates.iter().max().map(|d| fmt_date(*d));
        for a in assets {
            let p = &plan.assets[a];
            let after = match out.asset_ids.get(a).cloned().flatten() {
                Some(id) => {
                    crate::portfolio::engine::quantity_held(tx, &id, Some(&acc), &fmt_date(end))
                        .unwrap_or_default()
                }
                None => Decimal::ZERO,
            };
            let b = before.get(&(acc.clone(), a)).copied().unwrap_or_default();
            s.assets.push(super::B3InstitutionAsset {
                asset_key: p.key.clone(),
                ticker: p.ticker.clone(),
                name: p.name.clone(),
                before: b.normalize().to_string(),
                after: after.normalize().to_string(),
            });
        }
        inst.summary = s;
    }
    Ok(())
}

fn join_notes(a: Option<&String>, b: Option<&String>) -> Option<String> {
    match (a, b) {
        (Some(a), Some(b)) if a != b => Some(format!("{a}; {b}")),
        (Some(a), _) => Some(a.clone()),
        (None, b) => b.cloned(),
    }
}

fn assemble(
    plan: &Plan,
    out: &Outcome,
    token: String,
    files: Vec<B3FileSummary>,
    warnings: Vec<String>,
) -> B3Preview {
    let empty = ItemOutcome::default();
    let mut trades = vec![];
    let mut incomes = vec![];
    let mut actions = vec![];
    for it in &plan.items {
        let o = out.items.get(&it.index).unwrap_or(&empty);
        let status = if o.error.is_some() {
            B3RowStatus::Error
        } else if o.review.is_some() {
            B3RowStatus::Review
        } else if o.oversold {
            B3RowStatus::Warning
        } else {
            it.status
        };
        let a = &plan.assets[it.asset];
        match &it.op {
            Op::Trade {
                side,
                qty,
                price,
                origin,
                market,
                ..
            } => trades.push(B3TradeProposal {
                index: it.index,
                source: it.src.clone(),
                origin: *origin,
                date: fmt_date(it.date),
                asset_key: a.key.clone(),
                ticker: a.ticker.clone(),
                institution: it.institution.clone(),
                account_id: it.account.clone(),
                side: *side,
                quantity: qty.normalize().to_string(),
                price: price.normalize().to_string(),
                gross: to_minor(*qty * *price, "BRL"),
                market: market.clone(),
                external_id: it.external_id.clone(),
                status,
                duplicate_of: it.duplicate_of.clone(),
                note: it.note.clone(),
                error: o.error.clone(),
                warning: o.warning.clone(),
            }),
            Op::Auction { value, movement } => match o.auction_sale {
                Some((q, p)) => trades.push(B3TradeProposal {
                    index: it.index,
                    source: it.src.clone(),
                    origin: B3TradeOrigin::FractionAuction,
                    date: fmt_date(it.date),
                    asset_key: a.key.clone(),
                    ticker: a.ticker.clone(),
                    institution: it.institution.clone(),
                    account_id: it.account.clone(),
                    side: TradeSide::Sell,
                    quantity: q.normalize().to_string(),
                    price: p.normalize().to_string(),
                    gross: *value,
                    market: None,
                    external_id: it.external_id.clone(),
                    status,
                    duplicate_of: it.duplicate_of.clone(),
                    note: Some(t("nucleo-importacao-b3-nota-leilao-venda")),
                    error: o.error.clone(),
                    warning: None,
                }),
                None => incomes.push(B3IncomeProposal {
                    index: it.index,
                    source: it.src.clone(),
                    pay_date: fmt_date(it.date),
                    asset_key: a.key.clone(),
                    ticker: a.ticker.clone(),
                    institution: it.institution.clone(),
                    account_id: it.account.clone(),
                    kind: IncomeKind::Other,
                    movement: movement.clone(),
                    quantity: None,
                    gross: *value,
                    tax_withheld: 0,
                    net: *value,
                    external_id: it.external_id.clone(),
                    status,
                    duplicate_of: it.duplicate_of.clone(),
                    note: join_notes(
                        it.note.as_ref(),
                        Some(&t("nucleo-importacao-b3-nota-leilao-provento")),
                    ),
                    error: o.error.clone(),
                }),
            },
            Op::Income {
                kind,
                qty,
                gross,
                tax,
                net,
                movement,
                ..
            } => {
                let (gross, tax, net, tax_note) = match &o.income {
                    Some((g, t, n, note)) => (g, t, n, note.as_ref()),
                    None => (gross, tax, net, None),
                };
                incomes.push(B3IncomeProposal {
                    index: it.index,
                    source: it.src.clone(),
                    pay_date: fmt_date(it.date),
                    asset_key: a.key.clone(),
                    ticker: a.ticker.clone(),
                    institution: it.institution.clone(),
                    account_id: it.account.clone(),
                    kind: *kind,
                    movement: movement.clone(),
                    quantity: qty.map(|q| q.normalize().to_string()),
                    gross: *gross,
                    tax_withheld: *tax,
                    net: *net,
                    external_id: it.external_id.clone(),
                    status,
                    duplicate_of: it.duplicate_of.clone(),
                    note: join_notes(it.note.as_ref(), tax_note),
                    error: o.error.clone(),
                })
            }
            Op::Action {
                kind,
                unit_cost,
                movement,
                sources,
                legs,
            } => actions.push(B3ActionProposal {
                index: it.index,
                sources: sources.clone(),
                date: fmt_date(it.date),
                asset_key: a.key.clone(),
                ticker: a.ticker.clone(),
                kind: *kind,
                movement: movement.clone(),
                file_quantity: o
                    .file_qty
                    .or_else(|| {
                        let s: Decimal = legs
                            .iter()
                            .filter(|l| l.account.is_some())
                            .map(|l| l.qty)
                            .sum();
                        (!s.is_zero()).then_some(s)
                    })
                    .map(|q| q.normalize().to_string()),
                held_quantity: o.held.map(|q| q.normalize().to_string()),
                factor: o.factor.map(|f| f.normalize().to_string()),
                unit_cost: unit_cost.map(|u| u.normalize().to_string()),
                status,
                duplicate_of: it.duplicate_of.clone(),
                note: join_notes(
                    it.note.as_ref(),
                    o.review.as_ref().or(o.factor_note.as_ref()),
                )
                .or_else(|| {
                    (*kind == crate::portfolio::CorporateActionKind::Bonus && unit_cost.is_none())
                        .then(|| t("nucleo-importacao-b3-nota-bonificacao-sem-custo"))
                }),
                error: o.error.clone(),
            }),
        }
    }
    let assets: Vec<B3AssetProposal> = plan
        .assets
        .iter()
        .filter(|a| a.used)
        .map(|a| B3AssetProposal {
            key: a.key.clone(),
            ticker: a.ticker.clone(),
            name: a.name.clone(),
            class: a.class,
            class_reason: a.class_reason.clone(),
            existing_asset_id: a.existing.clone(),
            create: a.existing.is_none(),
            needs_review: a.weak,
            note: a.note.clone(),
            fixed_income: a.fixed_income.clone(),
        })
        .collect();
    let mut counts = B3Counts {
        new_assets: out.assets_created,
        ignored: plan.ignored.len() as u32,
        ..Default::default()
    };
    let statuses = trades
        .iter()
        .map(|t| (t.status, 0))
        .chain(incomes.iter().map(|i| (i.status, 1)))
        .chain(actions.iter().map(|a| (a.status, 2)));
    for (s, what) in statuses {
        match s {
            B3RowStatus::New => match what {
                0 => counts.trades += 1,
                1 => counts.incomes += 1,
                _ => counts.actions += 1,
            },
            B3RowStatus::Duplicate => counts.duplicates += 1,
            B3RowStatus::PossibleDuplicate => counts.possible_duplicates += 1,
            B3RowStatus::NoAccount => counts.no_account += 1,
            B3RowStatus::Review => counts.review += 1,
            // Written like a new row, and counted apart as something to explain.
            B3RowStatus::Warning => {
                counts.trades += 1;
                counts.warnings += 1;
            }
            B3RowStatus::Error => counts.errors += 1,
        }
    }
    let (from, to) = plan.period;
    B3Preview {
        token,
        files,
        period_from: from.map(fmt_date),
        period_to: to.map(fmt_date),
        uses_settlements: plan.uses_settlements,
        institutions: plan.institutions.clone(),
        assets,
        trades,
        incomes,
        actions,
        ignored: plan.ignored.clone(),
        position_check: None,
        openings: plan.openings.clone(),
        counts,
        warnings,
    }
}

/// `b3.commit`: grava a prévia num único conjunto de alterações.
pub fn commit(ctx: &mut Ctx<'_>, args: &B3CommitArgs) -> Result<B3CommitResult> {
    let stored = ctx
        .scratch
        .get_mut::<B3Scratch>(SCRATCH_KEY)
        .previews
        .iter()
        .find(|p| p.token == args.token)
        .cloned()
        .ok_or_else(|| Error::not_found(t("nucleo-importacao-b3-ref-previa")))?;
    let plan = &stored.plan;
    let by_index: HashMap<u32, &plan::Item> = plan.items.iter().map(|i| (i.index, i)).collect();
    let mut approvals = HashMap::new();
    for a in args.approve_actions.iter().flatten() {
        match by_index.get(&a.index) {
            Some(it)
                if matches!(
                    it.op,
                    Op::Action { .. }
                        | Op::Trade {
                            needs_cost: true,
                            ..
                        }
                ) => {}
            _ => {
                return Err(Error::invalid(t_args(
                    "nucleo-importacao-b3-erro-nao-evento",
                    &crate::i18n::args(&[("linha", a.index.into())]),
                )));
            }
        }
        approvals.insert(
            a.index,
            Approval {
                factor: a
                    .factor
                    .as_deref()
                    .map(|f| dec(f, "nucleo-carteira-campo-fator"))
                    .transpose()?,
                unit_cost: a
                    .unit_cost
                    .as_deref()
                    .map(|u| dec(u, "nucleo-carteira-campo-custo"))
                    .transpose()?,
            },
        );
    }
    let mut overrides = HashMap::new();
    for o in args.assets.iter().flatten() {
        if !plan.assets.iter().any(|a| a.key == o.key) {
            return Err(Error::invalid(t_args(
                "nucleo-importacao-b3-erro-ativo-fora",
                &crate::i18n::args(&[("ativo", o.key.clone().into())]),
            )));
        }
        if let Some(id) = &o.asset_id {
            crate::assets::get(ctx.conn(), id)?;
        }
        overrides.insert(o.key.clone(), o.clone());
    }
    let opts = ApplyOpts {
        include_possible: args.include_possible_duplicates.unwrap_or(false),
        skip: args.skip.iter().flatten().copied().collect(),
        approvals,
        overrides,
        cash: args.cash_movement.unwrap_or(false),
    };
    let writable = plan
        .items
        .iter()
        .filter(|it| it.status == B3RowStatus::New && !opts.skip.contains(&it.index))
        .count();
    let backup_id = if writable > crate::import::BACKUP_THRESHOLD {
        Some(crate::backup::create(ctx.db, &ctx.paths.backups_dir, BackupKind::PreImport)?.id)
    } else {
        None
    };
    let files = stored.filenames.join(", ");
    // O autor é sempre "importação"; quando um cliente MCP importa, ele fica no detalhe.
    let author = match ctx.actor {
        Actor::Mcp { client, .. } => Author::import(format!("B3: {files} — {client}")),
        _ => Author::import(format!("B3: {files}")),
    };
    let reason = t_args(
        "nucleo-importacao-b3-historico",
        &crate::i18n::args(&[("arquivos", files.clone().into())]),
    );
    let save_mapping = args.save_mapping.unwrap_or(true);
    let first_date = plan.period.0;
    let rec = ctx.write_as(&author, &reason, |tx| {
        // The accounts asked in the preview, created now in the same change set.
        let mut plan = plan.clone();
        let mut created = vec![];
        for (placeholder, n) in plan.new_accounts.clone() {
            let id = create_account(tx, &n, first_date)?;
            plan.remap_account(&placeholder, &id);
            created.push(id);
        }
        if save_mapping {
            let now = crate::time::now_utc();
            for inst in &plan.institutions {
                let Some(acc) = &inst.account_id else { continue };
                for key in plan::mapping_keys(inst) {
                    tx.execute(
                        "INSERT INTO import_institution_map (source, key, account_id, updated_at) VALUES ('b3', ?1, ?2, ?3)
                         ON CONFLICT (source, key) DO UPDATE SET account_id = excluded.account_id, updated_at = excluded.updated_at
                           WHERE import_institution_map.account_id <> excluded.account_id",
                        rusqlite::params![key, acc, now],
                    )?;
                }
            }
        }
        let out = apply::run(tx, &plan, &opts)?;
        Ok((out, created))
    })?;
    ctx.scratch
        .get_mut::<B3Scratch>(SCRATCH_KEY)
        .previews
        .retain(|p| p.token != args.token);
    let (out, accounts_created) = rec.value;
    let mut errors: Vec<B3CommitError> = out
        .items
        .iter()
        .filter_map(|(i, o)| {
            o.error.as_ref().map(|m| B3CommitError {
                index: *i,
                message: m.clone(),
            })
        })
        .collect();
    errors.sort_by_key(|e| e.index);
    let written = out.items.values().filter(|o| o.written).count() as u32;
    // What the portfolio itself says of the sales just written (all accounts together): a
    // sale another account's purchases cover is not pending.
    let oversold: Vec<crate::ids::Id> = out
        .items
        .values()
        .filter(|o| o.written && o.oversold)
        .filter_map(|o| o.trade_id.clone())
        .collect();
    let pending_sales = if oversold.is_empty() {
        0
    } else {
        crate::portfolio::uncovered::list(
            ctx.conn(),
            &crate::portfolio::uncovered::UncoveredSalesArgs {
                asset_id: None,
                trade_ids: Some(oversold),
            },
        )?
        .open
    };
    Ok(B3CommitResult {
        change_set_id: rec.change_set_id,
        assets_created: out.assets_created,
        trades: out.trades,
        incomes: out.incomes,
        actions: out.actions,
        skipped: plan.items.len() as u32 - written,
        errors,
        backup_id,
        accounts_created,
        pending_sales,
    })
}
