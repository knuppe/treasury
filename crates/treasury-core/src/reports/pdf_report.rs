//! Relatório do mês em PDF (Relatórios › "Baixar PDF"), gerado no próprio computador:
//! resumo do mês (receitas, despesas, sobra, patrimônio), para onde foi a renda pelos grupos de
//! gasto das categorias (com a meta de cada um), para onde foi o dinheiro com a
//! variação sobre o mês anterior, entradas e saídas de 12 meses, a inflação (gasto nominal ×
//! corrigido e categorias acima ou abaixo dela), o patrimônio de 12 meses e os próximos
//! vencimentos. Textos no idioma da instalação (catálogo `pdf.ftl`); valores pela formatação
//! do idioma. O arquivo é gravado onde a pessoa escolheu, como as exportações de Ajustes.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::engine::{month_dates, normalize_month, shift_month};
use super::inflation::{InflationArgs, InflationReport, inflation};
use super::pdf_writer::{A4_W, Document, Font, Page, Rgb, fit, text_width};
use super::spend_groups::{SpendGroupMonth, SpendGroupsArgs, spend_groups};
use super::{Dashboard, NetWorthPoint, Period, compare, dashboard_with, net_worth_history};
use crate::budget::{Basis, GroupTargets};
use crate::categories::CategoryKind;
use crate::categories::SpendGroup;
use crate::error::Result;
use crate::i18n::{self, Locale};
use fluent_bundle::FluentValue;

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct PdfArgs {
    /// Mês do relatório (`YYYY-MM`; padrão: o atual).
    #[ts(optional)]
    pub month: Option<String>,
    #[ts(optional)]
    pub basis: Option<Basis>,
    /// Caminho completo do arquivo a gravar.
    pub path: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct PdfResult {
    pub path: String,
    pub pages: u32,
    #[ts(type = "number")]
    pub bytes: u64,
}

const INK: Rgb = Rgb(27, 31, 36);
const DIM: Rgb = Rgb(107, 114, 128);
const LINE: Rgb = Rgb(226, 229, 234);
const ACCENT: Rgb = Rgb(37, 99, 235);
const POSITIVE: Rgb = Rgb(26, 127, 55);
const NEGATIVE: Rgb = Rgb(198, 40, 40);
const SOFT: Rgb = Rgb(245, 246, 248);
const INCOME_BAR: Rgb = Rgb(52, 160, 110);
const EXPENSE_BAR: Rgb = Rgb(225, 105, 75);
const REAL_LINE: Rgb = Rgb(196, 120, 0);

/// Cor de cada grupo de gasto (barra da parte da renda).
fn group_color(g: SpendGroup) -> Rgb {
    match g {
        SpendGroup::Essential => ACCENT,
        SpendGroup::Debt => NEGATIVE,
        SpendGroup::Future => POSITIVE,
        SpendGroup::NonEssential => REAL_LINE,
    }
}

const LEFT: f32 = 42.0;
const RIGHT: f32 = A4_W - 42.0;
const WIDTH: f32 = RIGHT - LEFT;
const BOTTOM: f32 = 790.0;

struct Ctx {
    l: Locale,
    currency: String,
}

impl Ctx {
    fn t(&self, key: &str) -> String {
        i18n::tr(self.l, key, None)
    }
    fn ta(&self, key: &str, a: &[(&'static str, String)]) -> String {
        let list: Vec<(&'static str, FluentValue<'static>)> = a
            .iter()
            .map(|(k, v)| (*k, FluentValue::from(v.clone())))
            .collect();
        i18n::tr(self.l, key, Some(&i18n::args(&list)))
    }
    fn money(&self, cents: i64) -> String {
        i18n::money(self.l, cents, &self.currency)
    }
    /// Dinheiro sem centavos, para eixos e rótulos curtos.
    fn money_short(&self, cents: i64) -> String {
        let s = self.money((cents / 100) * 100);
        let dec = if self.l == Locale::En { '.' } else { ',' };
        match s.rfind(dec) {
            Some(i) if s[i + 1..].starts_with("00") => {
                let tail = &s[i + 3..];
                format!("{}{tail}", &s[..i])
            }
            _ => s,
        }
    }
    fn pct(&self, f: f64, signed: bool) -> String {
        let v = f * 100.0;
        let mut s = format!("{:.1}", v.abs());
        if self.l != Locale::En {
            s = s.replace('.', ",");
        }
        let sign = if !signed || v.abs() < 0.05 {
            ""
        } else if v > 0.0 {
            "+"
        } else {
            "\u{2212}"
        };
        let neg = if !signed && v < 0.0 { "\u{2212}" } else { "" };
        format!("{sign}{neg}{s}%")
    }
    fn month_label(&self, month: &str, short: bool) -> String {
        let names = i18n::month_names(self.l, short);
        let (y, m) = month.split_once('-').unwrap_or(("", "01"));
        let m: usize = m.parse().unwrap_or(1);
        let name = names.get(m - 1).cloned().unwrap_or_default();
        if short {
            format!("{name}/{}", y.get(2..).unwrap_or(""))
        } else {
            self.ta("pdf-mes-ano", &[("mes", name), ("ano", y.to_string())])
        }
    }
}

fn hex(c: Option<&str>) -> Rgb {
    let Some(s) = c.map(|s| s.trim_start_matches('#')) else {
        return ACCENT;
    };
    if s.len() != 6 {
        return ACCENT;
    }
    let p = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).unwrap_or(128);
    Rgb(p(0), p(2), p(4))
}

fn section(p: &mut Page, y: f32, title: &str, hint: Option<&str>) -> f32 {
    p.text(LEFT, y, 12.5, Font::Bold, INK, title);
    let mut y = y + 6.0;
    if let Some(h) = hint {
        y = p.paragraph(LEFT, y + 9.0, WIDTH, 8.5, Font::Regular, DIM, h) - 4.0;
    }
    p.line(LEFT, y + 2.0, RIGHT, y + 2.0, 0.6, LINE);
    y + 14.0
}

fn kpis(p: &mut Page, c: &Ctx, y: f32, d: &Dashboard) -> f32 {
    let gap = 10.0;
    let w = (WIDTH - gap * 3.0) / 4.0;
    let flow = &d.flow;
    let items: [(String, String, Rgb, Option<String>); 4] = [
        (c.t("pdf-receitas"), c.money(flow.income), INK, None),
        (c.t("pdf-despesas"), c.money(flow.expense), INK, None),
        (
            c.t("pdf-sobra"),
            c.money(flow.net),
            if flow.net < 0 { NEGATIVE } else { POSITIVE },
            flow.savings_rate
                .map(|r| c.ta("pdf-taxa-poupanca", &[("taxa", c.pct(r, false))])),
        ),
        (c.t("pdf-patrimonio"), c.money(d.net_worth.net), INK, None),
    ];
    for (i, (label, value, color, sub)) in items.iter().enumerate() {
        let x = LEFT + i as f32 * (w + gap);
        p.rect(x, y, w, 58.0, SOFT);
        p.text(
            x + 10.0,
            y + 17.0,
            8.0,
            Font::Bold,
            DIM,
            &label.to_uppercase(),
        );
        p.text(
            x + 10.0,
            y + 37.0,
            13.0,
            Font::Bold,
            *color,
            &fit(value, w - 20.0, 13.0, Font::Bold),
        );
        if let Some(s) = sub {
            p.text(
                x + 10.0,
                y + 50.0,
                8.0,
                Font::Regular,
                DIM,
                &fit(s, w - 20.0, 8.0, Font::Regular),
            );
        }
    }
    y + 58.0
}

/// Os quatro grupos de gasto do mês: valor, parte da renda, meta e uma barra da parte (com a
/// marca da meta).
fn spend_group_cards(
    p: &mut Page,
    c: &Ctx,
    y: f32,
    m: &SpendGroupMonth,
    targets: &GroupTargets,
) -> f32 {
    let gap = 10.0;
    let w = (WIDTH - gap * 3.0) / 4.0;
    let h = 72.0;
    let inner = w - 20.0;
    for (i, g) in m.groups.iter().enumerate() {
        let x = LEFT + i as f32 * (w + gap);
        p.rect(x, y, w, h, SOFT);
        p.text(
            x + 10.0,
            y + 15.0,
            8.0,
            Font::Bold,
            DIM,
            &fit(
                &g.group.label_in(c.l).to_uppercase(),
                inner,
                8.0,
                Font::Bold,
            ),
        );
        p.text(
            x + 10.0,
            y + 33.0,
            13.0,
            Font::Bold,
            INK,
            &fit(&c.money(g.amount), inner, 13.0, Font::Bold),
        );
        let share = match g.share_of_income {
            Some(r) => c.ta(
                "nucleo-relatorios-grupos-pdf-parte",
                &[("parte", c.pct(r, false))],
            ),
            None => c.t("nucleo-relatorios-grupos-pdf-sem-renda"),
        };
        p.text(
            x + 10.0,
            y + 45.0,
            7.5,
            Font::Regular,
            INK,
            &fit(&share, inner, 7.5, Font::Regular),
        );
        let target = targets.of(g.group);
        let meta = match target {
            Some(t) => c.ta(
                "nucleo-relatorios-grupos-pdf-meta",
                &[("meta", crate::ai::alerts::pct(c.l, t, 0))],
            ),
            None => c.t("nucleo-relatorios-grupos-pdf-sem-meta"),
        };
        p.text(
            x + 10.0,
            y + 56.0,
            7.5,
            Font::Regular,
            DIM,
            &fit(&meta, inner, 7.5, Font::Regular),
        );
        p.rect(x + 10.0, y + 61.0, inner, 4.0, LINE);
        if let Some(r) = g.share_of_income.filter(|r| *r > 0.0) {
            let fill = (r.min(1.0) as f32 * inner).max(1.0);
            p.rect(x + 10.0, y + 61.0, fill, 4.0, group_color(g.group));
        }
        if let Some(t) = target {
            let tx = x + 10.0 + t.min(1.0) as f32 * inner;
            p.line(tx, y + 59.5, tx, y + 66.5, 1.0, INK);
        }
    }
    y + h
}

fn categories(p: &mut Page, c: &Ctx, y: f32, rows: &[super::ComparisonRow]) -> f32 {
    if rows.is_empty() {
        p.text(LEFT, y, 9.5, Font::Regular, DIM, &c.t("pdf-sem-gastos"));
        return y + 16.0;
    }
    let total: i64 = rows.iter().map(|r| r.b.max(0)).sum();
    let max = rows.iter().map(|r| r.b).max().unwrap_or(1).max(1);
    let label_w = 150.0;
    let bar_x = LEFT + label_w + 8.0;
    let bar_w = 170.0;
    let col_value = RIGHT - 125.0;
    let col_share = RIGHT - 70.0;
    let mut y = y;
    p.text(
        LEFT,
        y,
        7.5,
        Font::Bold,
        DIM,
        &c.t("pdf-col-categoria").to_uppercase(),
    );
    p.text_right(
        col_value,
        y,
        7.5,
        Font::Bold,
        DIM,
        &c.t("pdf-col-valor").to_uppercase(),
    );
    p.text_right(
        col_share,
        y,
        7.5,
        Font::Bold,
        DIM,
        &c.t("pdf-col-parte").to_uppercase(),
    );
    p.text_right(
        RIGHT,
        y,
        7.5,
        Font::Bold,
        DIM,
        &c.t("pdf-col-mes-anterior").to_uppercase(),
    );
    y += 12.0;
    for r in rows.iter().take(12) {
        p.text(
            LEFT,
            y,
            9.5,
            Font::Regular,
            INK,
            &fit(&r.label, label_w, 9.5, Font::Regular),
        );
        let w = (r.b.max(0) as f32 / max as f32) * bar_w;
        p.rect(bar_x, y - 7.5, w.max(1.0), 8.0, hex(r.color.as_deref()));
        p.text_right(col_value, y, 9.5, Font::Regular, INK, &c.money(r.b));
        let share = if total > 0 {
            r.b as f64 / total as f64
        } else {
            0.0
        };
        p.text_right(col_share, y, 9.5, Font::Regular, DIM, &c.pct(share, false));
        let (txt, color) = match r.delta_ratio {
            Some(d) => (
                c.pct(d, true),
                if d > 0.005 {
                    NEGATIVE
                } else if d < -0.005 {
                    POSITIVE
                } else {
                    DIM
                },
            ),
            None if r.a <= 0 && r.b > 0 => (c.t("pdf-novo"), DIM),
            None => ("\u{2014}".to_string(), DIM),
        };
        p.text_right(RIGHT, y, 9.5, Font::Regular, color, &txt);
        y += 15.0;
    }
    y
}

/// Barras de receitas e despesas dos 12 meses.
fn flow_chart(p: &mut Page, c: &Ctx, y: f32, d: &Dashboard) -> f32 {
    let h = 130.0;
    let hist = &d.flow_history;
    let max = hist
        .iter()
        .flat_map(|m| [m.income, m.expense])
        .max()
        .unwrap_or(0)
        .max(1);
    let axis_w = 62.0;
    let x0 = LEFT + axis_w;
    let w = WIDTH - axis_w;
    for k in 0..=4 {
        let gy = y + h - h * k as f32 / 4.0;
        p.line(x0, gy, RIGHT, gy, 0.4, LINE);
        p.text_right(
            x0 - 6.0,
            gy + 3.0,
            7.5,
            Font::Regular,
            DIM,
            &c.money_short(max * k / 4),
        );
    }
    let slot = w / hist.len().max(1) as f32;
    let bw = (slot * 0.32).min(14.0);
    for (i, m) in hist.iter().enumerate() {
        let cx = x0 + slot * i as f32 + slot / 2.0;
        let hi = h * m.income as f32 / max as f32;
        let he = h * m.expense as f32 / max as f32;
        p.rect(cx - bw - 1.0, y + h - hi, bw, hi.max(0.5), INCOME_BAR);
        p.rect(cx + 1.0, y + h - he, bw, he.max(0.5), EXPENSE_BAR);
        let lbl = c.month_label(&m.month, true);
        let tw = text_width(&lbl, 7.0, Font::Regular);
        p.text(cx - tw / 2.0, y + h + 11.0, 7.0, Font::Regular, DIM, &lbl);
    }
    let ly = y + h + 26.0;
    p.rect(LEFT, ly - 7.0, 8.0, 8.0, INCOME_BAR);
    p.text(
        LEFT + 12.0,
        ly,
        8.5,
        Font::Regular,
        INK,
        &c.t("pdf-receitas"),
    );
    let x2 = LEFT + 20.0 + text_width(&c.t("pdf-receitas"), 8.5, Font::Regular) + 12.0;
    p.rect(x2, ly - 7.0, 8.0, 8.0, EXPENSE_BAR);
    p.text(x2 + 12.0, ly, 8.5, Font::Regular, INK, &c.t("pdf-despesas"));
    ly + 8.0
}

/// Linhas do gasto nominal e corrigido pela inflação.
fn inflation_chart(p: &mut Page, c: &Ctx, y: f32, r: &InflationReport) -> f32 {
    let h = 120.0;
    let axis_w = 62.0;
    let x0 = LEFT + axis_w;
    let w = WIDTH - axis_w;
    let max = r
        .nominal
        .iter()
        .copied()
        .chain(r.real.iter().flatten().copied())
        .max()
        .unwrap_or(0)
        .max(1);
    for k in 0..=4 {
        let gy = y + h - h * k as f32 / 4.0;
        p.line(x0, gy, RIGHT, gy, 0.4, LINE);
        p.text_right(
            x0 - 6.0,
            gy + 3.0,
            7.5,
            Font::Regular,
            DIM,
            &c.money_short(max * k / 4),
        );
    }
    let n = r.months.len().max(2);
    let px = |i: usize| x0 + w * i as f32 / (n - 1) as f32;
    let py = |v: i64| y + h - h * v as f32 / max as f32;
    let nominal: Vec<(f32, f32)> = r
        .nominal
        .iter()
        .enumerate()
        .map(|(i, v)| (px(i), py(*v)))
        .collect();
    p.polyline(&nominal, 1.6, ACCENT);
    // Corrigido: contínuo onde o índice é oficial, tracejado onde é estimado.
    let real: Vec<(usize, f32, f32)> = r
        .real
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|v| (i, px(i), py(v))))
        .collect();
    let solid: Vec<(f32, f32)> = real
        .iter()
        .filter(|(i, _, _)| !r.estimated[*i])
        .map(|(_, x, y)| (*x, *y))
        .collect();
    p.polyline(&solid, 1.6, REAL_LINE);
    let first_est = real.iter().position(|(i, _, _)| r.estimated[*i]);
    if let Some(k) = first_est {
        let from = k.saturating_sub(1);
        let tail: Vec<(f32, f32)> = real[from..].iter().map(|(_, x, y)| (*x, *y)).collect();
        p.dashed_polyline(&tail, 1.6, REAL_LINE);
    }
    for (i, m) in r.months.iter().enumerate() {
        if n > 8 && i % 2 == 1 {
            continue;
        }
        let lbl = c.month_label(m, true);
        let tw = text_width(&lbl, 7.0, Font::Regular);
        p.text(
            px(i) - tw / 2.0,
            y + h + 11.0,
            7.0,
            Font::Regular,
            DIM,
            &lbl,
        );
    }
    let ly = y + h + 26.0;
    p.rect(LEFT, ly - 4.0, 12.0, 2.0, ACCENT);
    let a = c.t("pdf-inflacao-nominal");
    p.text(LEFT + 16.0, ly, 8.5, Font::Regular, INK, &a);
    let x2 = LEFT + 30.0 + text_width(&a, 8.5, Font::Regular);
    p.rect(x2, ly - 4.0, 12.0, 2.0, REAL_LINE);
    p.text(
        x2 + 16.0,
        ly,
        8.5,
        Font::Regular,
        INK,
        &c.t("pdf-inflacao-corrigido"),
    );
    ly + 8.0
}

fn inflation_table(p: &mut Page, c: &Ctx, y: f32, r: &InflationReport) -> f32 {
    if r.categories.is_empty() {
        return y;
    }
    let mut y = y;
    let cols = [
        RIGHT - 250.0,
        RIGHT - 175.0,
        RIGHT - 115.0,
        RIGHT - 60.0,
        RIGHT,
    ];
    p.text(
        LEFT,
        y,
        7.5,
        Font::Bold,
        DIM,
        &c.t("pdf-col-categoria").to_uppercase(),
    );
    for (x, key) in cols.iter().zip([
        "pdf-col-inicio",
        "pdf-col-fim",
        "pdf-col-variacao",
        "pdf-col-inflacao",
        "pdf-col-real",
    ]) {
        p.text_right(*x, y, 7.5, Font::Bold, DIM, &c.t(key).to_uppercase());
    }
    y += 12.0;
    for cat in &r.categories {
        p.text(
            LEFT,
            y,
            9.0,
            Font::Regular,
            INK,
            &fit(&cat.label, cols[0] - LEFT - 80.0, 9.0, Font::Regular),
        );
        p.text_right(cols[0], y, 9.0, Font::Regular, INK, &c.money(cat.first));
        p.text_right(cols[1], y, 9.0, Font::Regular, INK, &c.money(cat.last));
        let dash = "\u{2014}".to_string();
        p.text_right(
            cols[2],
            y,
            9.0,
            Font::Regular,
            INK,
            &cat.change.map(|v| c.pct(v, true)).unwrap_or(dash.clone()),
        );
        p.text_right(
            cols[3],
            y,
            9.0,
            Font::Regular,
            DIM,
            &cat.inflation
                .map(|v| c.pct(v, true))
                .unwrap_or(dash.clone()),
        );
        let (txt, color) = match cat.above_inflation {
            Some(v) if v > 0.005 => (c.ta("pdf-acima", &[("pp", c.pct(v, false))]), NEGATIVE),
            Some(v) if v < -0.005 => (c.ta("pdf-abaixo", &[("pp", c.pct(-v, false))]), POSITIVE),
            Some(_) => (c.t("pdf-igual"), DIM),
            None => (dash, DIM),
        };
        p.text_right(cols[4], y, 9.0, Font::Regular, color, &txt);
        y += 14.0;
    }
    y
}

fn net_worth_chart(p: &mut Page, c: &Ctx, y: f32, pts: &[NetWorthPoint]) -> f32 {
    let h = 100.0;
    let axis_w = 70.0;
    let x0 = LEFT + axis_w;
    let w = WIDTH - axis_w;
    let lo = pts.iter().map(|p| p.net).min().unwrap_or(0).min(0);
    let hi = pts.iter().map(|p| p.net).max().unwrap_or(1).max(lo + 1);
    for k in 0..=4 {
        let gy = y + h - h * k as f32 / 4.0;
        p.line(x0, gy, RIGHT, gy, 0.4, LINE);
        p.text_right(
            x0 - 6.0,
            gy + 3.0,
            7.5,
            Font::Regular,
            DIM,
            &c.money_short(lo + (hi - lo) * k / 4),
        );
    }
    let n = pts.len().max(2);
    let line: Vec<(f32, f32)> = pts
        .iter()
        .enumerate()
        .map(|(i, pt)| {
            (
                x0 + w * i as f32 / (n - 1) as f32,
                y + h - h * (pt.net - lo) as f32 / (hi - lo) as f32,
            )
        })
        .collect();
    p.polyline(&line, 1.8, ACCENT);
    for (i, pt) in pts.iter().enumerate() {
        if n > 8 && i % 2 == 1 {
            continue;
        }
        let lbl = c.month_label(&pt.month, true);
        let tw = text_width(&lbl, 7.0, Font::Regular);
        p.text(
            line[i].0 - tw / 2.0,
            y + h + 11.0,
            7.0,
            Font::Regular,
            DIM,
            &lbl,
        );
    }
    y + h + 20.0
}

fn upcoming(p: &mut Page, c: &Ctx, y: f32, d: &Dashboard) -> f32 {
    let mut y = y;
    if d.upcoming.is_empty() {
        p.text(
            LEFT,
            y,
            9.5,
            Font::Regular,
            DIM,
            &c.t("pdf-sem-vencimentos"),
        );
        return y + 16.0;
    }
    for u in d.upcoming.iter().take(10) {
        let date = crate::time::parse_date(&u.date)
            .map(|d| i18n::date(c.l, d))
            .unwrap_or_else(|_| u.date.clone());
        p.text(LEFT, y, 9.0, Font::Regular, DIM, &date);
        p.text(
            LEFT + 70.0,
            y,
            9.0,
            Font::Regular,
            INK,
            &fit(&u.description, 250.0, 9.0, Font::Regular),
        );
        p.text(
            LEFT + 330.0,
            y,
            9.0,
            Font::Regular,
            DIM,
            &fit(&u.account_name, 90.0, 9.0, Font::Regular),
        );
        let color = if u.amount < 0 { INK } else { POSITIVE };
        p.text_right(
            RIGHT,
            y,
            9.0,
            Font::Regular,
            color,
            &i18n::money(c.l, u.amount, &u.currency),
        );
        y += 14.0;
    }
    y
}

fn footer(p: &mut Page, c: &Ctx, page: usize, pages: usize, generated: &str) {
    p.line(LEFT, BOTTOM + 8.0, RIGHT, BOTTOM + 8.0, 0.5, LINE);
    p.text(
        LEFT,
        BOTTOM + 22.0,
        7.5,
        Font::Regular,
        DIM,
        &c.ta(
            "pdf-rodape",
            &[
                ("data", generated.to_string()),
                ("moeda", c.currency.clone()),
            ],
        ),
    );
    p.text_right(
        RIGHT,
        BOTTOM + 22.0,
        7.5,
        Font::Regular,
        DIM,
        &c.ta(
            "pdf-pagina",
            &[("n", page.to_string()), ("total", pages.to_string())],
        ),
    );
}

/// Monta o PDF do mês (sem gravar).
pub fn render(conn: &Connection, month: Option<&str>, basis: Option<Basis>) -> Result<Vec<u8>> {
    let month = normalize_month(
        &month
            .map(str::to_string)
            .unwrap_or_else(super::engine::current_month),
    )?;
    let basis = super::engine::resolve_basis(conn, basis)?;
    let d = dashboard_with(conn, Some(&month), Some(basis))?;
    let c = Ctx {
        l: i18n::current(),
        currency: d.currency.clone(),
    };
    let prev = shift_month(&month, -1)?;
    let (pa, pb) = month_dates(&prev)?;
    let (ma, mb) = month_dates(&month)?;
    let mut cmp = compare(
        conn,
        &Period { from: pa, to: pb },
        &Period { from: ma, to: mb },
        basis,
        CategoryKind::Expense,
    )?;
    cmp.rows.retain(|r| r.b > 0);
    cmp.rows.sort_by_key(|r| std::cmp::Reverse(r.b));
    let infl = inflation(
        conn,
        &InflationArgs {
            months: Some(12),
            end_month: Some(month.clone()),
            basis: Some(basis),
        },
        basis,
    )?;
    let nw = net_worth_history(conn, 12, &month)?;
    let groups = spend_groups(
        conn,
        &SpendGroupsArgs {
            from: month.clone(),
            to: month.clone(),
            basis: Some(basis),
            member_id: None,
        },
    )?;
    let household: String = conn
        .query_row("SELECT name FROM household LIMIT 1", [], |r| r.get(0))
        .unwrap_or_default();
    let generated = i18n::date(c.l, crate::time::today_local());
    let basis_label = c.t(if basis == Basis::Cash {
        "pdf-base-caixa"
    } else {
        "pdf-base-competencia"
    });

    let mut doc = Document {
        title: c.ta("pdf-titulo", &[("mes", c.month_label(&month, false))]),
        ..Default::default()
    };

    // Página 1: resumo, categorias e 12 meses.
    {
        let p = doc.new_page();
        p.text(LEFT, 50.0, 8.0, Font::Bold, ACCENT, "TREASURY");
        p.text_right(RIGHT, 50.0, 8.0, Font::Regular, DIM, &generated);
        p.text(
            LEFT,
            78.0,
            20.0,
            Font::Bold,
            INK,
            &c.ta("pdf-titulo", &[("mes", c.month_label(&month, false))]),
        );
        let sub = if household.is_empty() {
            basis_label.clone()
        } else {
            format!("{household} \u{00B7} {basis_label}")
        };
        p.text(LEFT, 96.0, 9.5, Font::Regular, DIM, &sub);
        let mut y = kpis(p, &c, 112.0, &d);
        if !d.flow.missing_fx.is_empty() {
            y = p.paragraph(
                LEFT,
                y + 16.0,
                WIDTH,
                8.0,
                Font::Regular,
                DIM,
                &c.ta(
                    "pdf-sem-cambio",
                    &[("moedas", d.flow.missing_fx.join(", "))],
                ),
            ) - 6.0;
        }
        y = section(
            p,
            y + 30.0,
            &c.t("nucleo-relatorios-grupos-pdf-titulo"),
            Some(&c.t("nucleo-relatorios-grupos-pdf-dica")),
        );
        y = spend_group_cards(p, &c, y, &groups.total, &groups.targets);
        y = section(
            p,
            y + 30.0,
            &c.t("pdf-secao-categorias"),
            Some(&c.t("pdf-secao-categorias-dica")),
        );
        y = categories(p, &c, y, &cmp.rows);
        // Os 12 meses (seção, gráfico e legenda: ~200 pt) vão para uma página própria quando
        // não cabem mais nesta.
        if y + 22.0 + 200.0 <= BOTTOM {
            y = section(p, y + 22.0, &c.t("pdf-secao-12-meses"), None);
            let _ = flow_chart(p, &c, y + 4.0, &d);
        } else {
            let p = doc.new_page();
            let y = section(p, 60.0, &c.t("pdf-secao-12-meses"), None);
            let _ = flow_chart(p, &c, y + 4.0, &d);
        }
    }

    // Página 2: inflação, patrimônio e vencimentos.
    {
        let p = doc.new_page();
        let hint = match (&infl.index, infl.inflation) {
            (Some(ix), Some(v)) => c.ta(
                "pdf-inflacao-resumo",
                &[
                    ("inflacao", c.pct(v, false)),
                    ("indice", ix.name.clone()),
                    (
                        "data",
                        ix.last_date
                            .as_deref()
                            .and_then(|d| d.get(..7))
                            .map(|m| c.month_label(m, false))
                            .unwrap_or_else(|| "\u{2014}".into()),
                    ),
                ],
            ),
            _ => c.t("pdf-inflacao-sem-indice"),
        };
        let mut y = section(p, 60.0, &c.t("pdf-secao-inflacao"), Some(&hint));
        if infl.estimated.iter().any(|e| *e) {
            y = p.paragraph(
                LEFT,
                y,
                WIDTH,
                8.0,
                Font::Regular,
                DIM,
                &c.t("pdf-inflacao-estimado"),
            ) + 4.0;
        }
        y = inflation_chart(p, &c, y + 4.0, &infl);
        if !infl.categories.is_empty() {
            y = section(
                p,
                y + 20.0,
                &c.t("pdf-secao-inflacao-categorias"),
                Some(&c.t("pdf-secao-inflacao-categorias-dica")),
            );
            y = inflation_table(p, &c, y, &infl);
        }
        y = section(p, y + 20.0, &c.t("pdf-secao-patrimonio"), None);
        y = net_worth_chart(p, &c, y + 4.0, &nw);
        if y < BOTTOM - 120.0 {
            y = section(p, y + 16.0, &c.t("pdf-secao-vencimentos"), None);
            let _ = upcoming(p, &c, y, &d);
        } else {
            let p = doc.new_page();
            let y = section(p, 60.0, &c.t("pdf-secao-vencimentos"), None);
            let _ = upcoming(p, &c, y, &d);
        }
    }

    let total = doc.pages.len();
    for (i, p) in doc.pages.iter_mut().enumerate() {
        footer(p, &c, i + 1, total, &generated);
    }
    Ok(doc.to_bytes())
}

/// Grava o PDF do mês em `path` (mesmas regras de destino das exportações).
pub fn write(
    conn: &Connection,
    a: &PdfArgs,
    paths: &crate::paths::TreasuryPaths,
) -> Result<PdfResult> {
    let bytes = render(conn, a.month.as_deref(), a.basis)?;
    let pages = page_count(&bytes);
    let dest = crate::export::check_destination(&a.path, paths)?;
    let written = crate::export::write_private(&dest, |w| {
        use std::io::Write as _;
        w.write_all(&bytes)?;
        Ok(())
    })?;
    Ok(PdfResult {
        path: dest.display().to_string(),
        pages,
        bytes: written,
    })
}

/// Páginas do PDF gerado (conta os objetos `/Type /Page `).
fn page_count(bytes: &[u8]) -> u32 {
    let needle = b"/Type /Page /Parent";
    bytes.windows(needle.len()).filter(|w| *w == needle).count() as u32
}
