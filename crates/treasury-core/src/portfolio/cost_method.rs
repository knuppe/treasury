//! Método de custo das vendas de renda variável (docs/cost-method.md e
//! docs/sales-real-return.md).
//!
//! O livro de vendas mostra o resultado pelo método do país da família, que é o da regra
//! fiscal de lá: preço médio (Brasil, Canadá, França e o padrão dos demais), PEPS/FIFO
//! (Portugal, Espanha, Alemanha e Estados Unidos) ou o "pool" do Reino Unido (Section 104: as
//! compras do mesmo dia casam primeiro, depois as dos 30 dias seguintes, e o resto sai do pool
//! pelo custo médio). A família pode escolher outro em Ajustes. O preço médio continua sendo a
//! base do IR brasileiro e da carteira (custo da posição, resultado não realizado); renda fixa
//! e Tesouro seguem com os lotes (PEPS) que a carteira já usa.

use std::collections::{HashMap, VecDeque};

use chrono::NaiveDate;
use rusqlite::{Connection, OptionalExtension};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::Result;

/// Ajuste da família (sem ele, o método do país).
pub const SETTING: &str = "portfolio.cost_method";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum CostMethod {
    /// Preço médio ponderado: cada venda leva a mesma fração de todas as compras.
    #[default]
    Average,
    /// Primeiro que entra, primeiro que sai.
    Fifo,
    /// Reino Unido: mesmo dia, depois os 30 dias seguintes, depois o pool pelo custo médio.
    UkPool,
}

impl CostMethod {
    fn parse(s: &str) -> Self {
        match s {
            "fifo" => CostMethod::Fifo,
            "uk_pool" => CostMethod::UkPool,
            _ => CostMethod::Average,
        }
    }
}

/// O método em uso e de onde ele vem.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CostMethodState {
    pub method: CostMethod,
    /// Segue o país da família (sem escolha em Ajustes).
    pub automatic: bool,
    pub country: String,
    /// O método do país.
    pub country_method: CostMethod,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetCostMethodArgs {
    /// `null`: seguir o país da família.
    #[serde(default)]
    #[ts(optional)]
    pub method: Option<CostMethod>,
}

/// O método do país no catálogo (V152); fora do catálogo, preço médio.
pub fn country_method(conn: &Connection, country: &str) -> Result<CostMethod> {
    let m: Option<String> = conn
        .query_row(
            "SELECT cost_method FROM country_profile WHERE country = ?1",
            [country],
            |r| r.get(0),
        )
        .optional()?;
    Ok(m.as_deref().map_or(CostMethod::Average, CostMethod::parse))
}

pub fn state(conn: &Connection) -> Result<CostMethodState> {
    let country = crate::country::family_country(conn);
    let country_method = country_method(conn, &country)?;
    let chosen: Option<CostMethod> = crate::settings::get_or(conn, SETTING, None)?;
    Ok(CostMethodState {
        method: chosen.unwrap_or(country_method),
        automatic: chosen.is_none(),
        country,
        country_method,
    })
}

pub fn set(conn: &Connection, a: &SetCostMethodArgs) -> Result<CostMethodState> {
    crate::settings::set(conn, SETTING, &a.method)?;
    state(conn)
}

/// Um movimento da posição, na ordem da carteira (para o casamento por lote).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Move {
    /// Compra (ou herança, bonificação com custo): quantidade e custo na moeda base.
    Acq {
        date: NaiveDate,
        qty: Decimal,
        cost: Decimal,
    },
    /// Desdobramento, grupamento, bonificação sem custo, come-cotas, amortização: a
    /// quantidade e o custo de tudo o que está na posição mudam na mesma proporção.
    Scale { qty: Decimal, cost: Decimal },
    /// Venda `id` (na ordem das vendas do ativo).
    Disp {
        date: NaiveDate,
        qty: Decimal,
        id: usize,
    },
    /// A posição saiu inteira para outro ativo.
    Reset,
}

/// O custo de uma venda pelo método e de onde ele saiu (data e custo, para o resultado real e
/// o prazo).
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Matched {
    pub cost: Decimal,
    pub out: Vec<(NaiveDate, Decimal)>,
}

/// O custo de cada venda pelo método (PEPS ou pool do Reino Unido). O preço médio não passa
/// por aqui: ele é o da carteira.
pub(crate) fn match_lots(method: CostMethod, moves: &[Move]) -> HashMap<usize, Matched> {
    match method {
        CostMethod::Average => HashMap::new(),
        CostMethod::Fifo => fifo(moves),
        CostMethod::UkPool => uk_pool(moves),
    }
}

fn fifo(moves: &[Move]) -> HashMap<usize, Matched> {
    let mut lots: VecDeque<(NaiveDate, Decimal, Decimal)> = VecDeque::new();
    let mut out = HashMap::new();
    for m in moves {
        match m {
            Move::Acq { date, qty, cost } => {
                if *qty > Decimal::ZERO {
                    lots.push_back((*date, *qty, *cost));
                }
            }
            Move::Scale { qty, cost } => {
                for l in lots.iter_mut() {
                    l.1 *= *qty;
                    l.2 *= *cost;
                }
            }
            Move::Reset => lots.clear(),
            Move::Disp { qty, id, .. } => {
                let mut left = *qty;
                let mut got = Matched::default();
                while left > Decimal::ZERO {
                    let Some(lot) = lots.front_mut() else { break };
                    let take = left.min(lot.1);
                    let part = if lot.1.is_zero() {
                        Decimal::ZERO
                    } else {
                        lot.2 * take / lot.1
                    };
                    got.cost += part;
                    got.out.push((lot.0, part));
                    lot.1 -= take;
                    lot.2 -= part;
                    left -= take;
                    if lot.1 <= Decimal::ZERO {
                        lots.pop_front();
                    }
                }
                out.insert(*id, got);
            }
        }
    }
    out
}

/// Reino Unido (TCGA 1992 s105 e s106A; HMRC, Capital Gains Manual): cada venda casa primeiro
/// com as compras do mesmo dia, depois com as dos 30 dias seguintes (da mais antiga para a mais
/// nova), e o resto sai do pool (Section 104) pelo custo médio. As compras casadas assim não
/// entram no pool.
fn uk_pool(moves: &[Move]) -> HashMap<usize, Matched> {
    // Compras: (posição em `moves`, data, quantidade, custo, quantidade ainda livre).
    let mut acqs: Vec<(usize, NaiveDate, Decimal, Decimal, Decimal)> = moves
        .iter()
        .enumerate()
        .filter_map(|(i, m)| match m {
            Move::Acq { date, qty, cost } if *qty > Decimal::ZERO => {
                Some((i, *date, *qty, *cost, *qty))
            }
            _ => None,
        })
        .collect();
    // Peças casadas por venda: (data da compra, custo).
    let mut reserved: HashMap<usize, (Decimal, Vec<(NaiveDate, Decimal)>)> = HashMap::new();
    for m in moves {
        let Move::Disp { date, qty, id } = m else {
            continue;
        };
        let mut need = *qty;
        let mut pieces = vec![];
        let horizon = *date + chrono::Duration::days(30);
        // Mesmo dia, depois os 30 dias seguintes em ordem de data.
        for pass in 0..2 {
            for a in acqs.iter_mut() {
                if need <= Decimal::ZERO {
                    break;
                }
                let fits = if pass == 0 {
                    a.1 == *date
                } else {
                    a.1 > *date && a.1 <= horizon
                };
                if !fits || a.4 <= Decimal::ZERO {
                    continue;
                }
                let take = need.min(a.4);
                pieces.push((a.1, a.3 * take / a.2));
                a.4 -= take;
                need -= take;
            }
        }
        reserved.insert(*id, (*qty - need, pieces));
    }
    let free: HashMap<usize, Decimal> = acqs.iter().map(|a| (a.0, a.4)).collect();
    // Pool, com as camadas (data, custo) para o resultado real e o prazo.
    let mut pool_qty = Decimal::ZERO;
    let mut layers: Vec<(NaiveDate, Decimal)> = vec![];
    let mut out = HashMap::new();
    for (i, m) in moves.iter().enumerate() {
        match m {
            Move::Acq { date, qty, cost } => {
                let q = free.get(&i).copied().unwrap_or(Decimal::ZERO);
                if q > Decimal::ZERO && *qty > Decimal::ZERO {
                    pool_qty += q;
                    layers.push((*date, *cost * q / *qty));
                }
            }
            Move::Scale { qty, cost } => {
                pool_qty *= *qty;
                for l in layers.iter_mut() {
                    l.1 *= *cost;
                }
            }
            Move::Reset => {
                pool_qty = Decimal::ZERO;
                layers.clear();
            }
            Move::Disp { date, qty, id } => {
                let (taken, pieces) = reserved.remove(id).unwrap_or_default();
                let mut got = Matched::default();
                for (d, c) in pieces {
                    // Compra depois da venda (regra dos 30 dias): conta como da data da venda.
                    got.out.push((d.min(*date), c));
                    got.cost += c;
                }
                let rest = *qty - taken;
                if rest > Decimal::ZERO && pool_qty > Decimal::ZERO {
                    let share = (rest / pool_qty).min(Decimal::ONE);
                    for l in layers.iter_mut() {
                        let part = l.1 * share;
                        got.out.push((l.0, part));
                        got.cost += part;
                        l.1 -= part;
                    }
                    layers.retain(|l| !l.1.is_zero());
                    pool_qty -= rest.min(pool_qty);
                }
                out.insert(*id, got);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        crate::time::parse_date(s).unwrap()
    }
    fn dec(s: &str) -> Decimal {
        s.parse().unwrap()
    }
    fn acq(date: &str, qty: &str, cost: &str) -> Move {
        Move::Acq {
            date: d(date),
            qty: dec(qty),
            cost: dec(cost),
        }
    }
    fn sell(date: &str, qty: &str, id: usize) -> Move {
        Move::Disp {
            date: d(date),
            qty: dec(qty),
            id,
        }
    }

    #[test]
    fn fifo_takes_the_oldest_lots_first() {
        let moves = vec![
            acq("2024-01-10", "100", "1000"),
            acq("2024-06-10", "100", "2000"),
            sell("2025-01-10", "150", 0),
            sell("2025-02-10", "50", 1),
        ];
        let m = match_lots(CostMethod::Fifo, &moves);
        // 100 da primeira (1.000) + 50 da segunda (1.000).
        assert_eq!(m[&0].cost, dec("2000"));
        assert_eq!(
            m[&0].out,
            vec![
                (d("2024-01-10"), dec("1000")),
                (d("2024-06-10"), dec("1000"))
            ]
        );
        assert_eq!(m[&1].cost, dec("1000"));
    }

    #[test]
    fn fifo_follows_splits_and_capital_returns() {
        let moves = vec![
            acq("2024-01-10", "10", "1000"),
            // Desdobramento 1 para 2: 20 cotas, o mesmo custo.
            Move::Scale {
                qty: dec("2"),
                cost: Decimal::ONE,
            },
            acq("2024-03-10", "10", "600"),
            // Amortização de 10% do custo.
            Move::Scale {
                qty: Decimal::ONE,
                cost: dec("0.9"),
            },
            sell("2024-06-10", "25", 0),
        ];
        let m = match_lots(CostMethod::Fifo, &moves);
        // 20 cotas da primeira (900) + 5 da segunda (270).
        assert_eq!(m[&0].cost, dec("1170"));
    }

    #[test]
    fn uk_matches_same_day_then_thirty_days_then_the_pool() {
        let moves = vec![
            acq("2024-01-10", "100", "1000"), // pool: 10 por cota
            acq("2024-02-01", "100", "3000"), // pool: 20 por cota → média 20
            sell("2024-03-01", "50", 0),      // 10 no mesmo dia, 30 dos 30 dias, 10 do pool
            acq("2024-03-01", "10", "500"),   // mesmo dia: 50 por cota
            acq("2024-03-15", "30", "1200"),  // 30 dias: 40 por cota
            acq("2024-05-01", "10", "100"),   // fora dos 30 dias: vai para o pool
            sell("2024-06-01", "20", 1),
        ];
        let m = match_lots(CostMethod::UkPool, &moves);
        // 500 + 1.200 + 10 cotas do pool a 20 = 200.
        assert_eq!(m[&0].cost, dec("1900"));
        // A compra dos 30 dias conta como da data da venda.
        assert!(m[&0].out.contains(&(d("2024-03-01"), dec("1200"))));
        // Pool depois: 190 cotas a 20 (3.800) + 10 a 10 (100) = 200 cotas, 3.900 → 19,5.
        assert_eq!(m[&1].cost, dec("390"));
    }

    #[test]
    fn average_is_left_to_the_portfolio() {
        let moves = vec![acq("2024-01-10", "1", "1"), sell("2024-02-10", "1", 0)];
        assert!(match_lots(CostMethod::Average, &moves).is_empty());
    }
}
