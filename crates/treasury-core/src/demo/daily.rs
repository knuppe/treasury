//! Dia a dia dos cofres de demonstração (docs/demo.md): o mercado da semana no sábado (com o
//! cupom fiscal item a item no Brasil), a feira e o açougue no meio da semana, a padaria do fim
//! de semana, o café dos dias úteis, os restaurantes da sexta ao domingo, as assinaturas de cada
//! perfil (com um reajuste), as transferências instantâneas (Pix no Brasil) para a diarista e a
//! mesada, a compra parcelada no cartão e o que cada perfil tem de próprio: gás de cozinha,
//! academia, pet, salão, doações, presentes e a ajuda da família no fim do mês.
//!
//! Os gastos do dia a dia saem dos valores que a pessoa informou no assistente; o que é próprio
//! de cada perfil sai de uma fração da renda, para valer em qualquer moeda.

use chrono::{Datelike, NaiveDate, Weekday};
use serde_json::json;

use super::{Accounts, DemoKind, Gen, Rng, add_months, ages, input, on, seed_from, split};
use crate::error::Result;
use crate::ids::Id;
use crate::receipts::{ManualItem, ManualReceipt, ReceiptArgs};
use crate::time::{fmt_date, month_bounds};

const WEEKDAYS: [Weekday; 5] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
];

/// Dias do mês que caem num destes dias da semana.
fn days_of(month: NaiveDate, which: &[Weekday]) -> Vec<NaiveDate> {
    let last = month_bounds(month.year(), month.month()).1.day();
    (1..=last)
        .filter_map(|d| month.with_day(d))
        .filter(|d| which.contains(&d.weekday()))
        .collect()
}

/// Até `n` dias diferentes do mês nestes dias da semana, em ordem.
fn pick_days(g: &mut Gen<'_, '_>, month: NaiveDate, which: &[Weekday], n: usize) -> Vec<NaiveDate> {
    let mut pool = days_of(month, which);
    let mut out = Vec::with_capacity(n);
    while out.len() < n && !pool.is_empty() {
        let i = (g.rng.next() % pool.len() as u64) as usize;
        out.push(pool.remove(i));
    }
    out.sort();
    out
}

/// Renda do mês da família (base das despesas próprias de cada perfil).
fn family_income(g: &Gen<'_, '_>) -> i64 {
    let p = g.p;
    p.adults.iter().map(|a| a.income.max(0)).sum::<i64>() + p.other_income + p.benefit.unwrap_or(0)
}

/// Fração `per_mille` (por mil) da renda, com variação.
fn of_income(g: &mut Gen<'_, '_>, per_mille: i64, vary: f64) -> i64 {
    let base = family_income(g) * per_mille / 1000;
    g.rng.vary(base, vary).max(0)
}

fn kind(g: &Gen<'_, '_>) -> DemoKind {
    g.p.kind.unwrap_or(DemoKind::Custom)
}

fn adults(g: &Gen<'_, '_>) -> Vec<Option<Id>> {
    g.p.adults.iter().map(|x| g.member(&x.name)).collect()
}

fn someone(g: &mut Gen<'_, '_>) -> Option<Id> {
    let list = adults(g);
    g.rng.pick(&list).clone()
}

/// Transferência instantânea enviada, com o nome do jeito do país (Pix, MB WAY, Zelle).
fn sent(g: &Gen<'_, '_>, name: &str) -> String {
    let key = if g.s.brazil() {
        "demodados-envio-br"
    } else if g.s.portugal() {
        "demodados-envio-pt"
    } else if g.s.usa() {
        "demodados-envio-us"
    } else {
        "demodados-envio-outro"
    };
    g.ta(key, &[("nome", name.to_string())])
}

fn received(g: &Gen<'_, '_>, name: &str) -> String {
    let key = if g.s.brazil() {
        "demodados-recebido-br"
    } else if g.s.portugal() {
        "demodados-recebido-pt"
    } else if g.s.usa() {
        "demodados-recebido-us"
    } else {
        "demodados-recebido-outro"
    };
    g.ta(key, &[("nome", name.to_string())])
}

/// Cartão, se houver; senão, a conta (débito).
fn card_or_debit(a: &Accounts) -> Id {
    a.card.clone().unwrap_or_else(|| a.checking.clone())
}

// ---------------------------------------------------------------------------
// Mercado, feira e padaria

/// O mercado da semana (70%), a feira, o hortifrúti e o açougue (15%) e a padaria (15%). Os
/// cupons das compras de mercado recentes saem depois, em `super::receipts`.
pub(super) fn groceries(g: &mut Gen<'_, '_>, a: &Accounts, month: NaiveDate) -> Result<()> {
    let p = g.p;
    if p.groceries <= 0 {
        return Ok(());
    }
    let total = g.rng.vary(p.groceries, 0.08);
    let big = total * 70 / 100;
    let local = total * 15 / 100;
    let bakery = total - big - local;
    // O mercado de sempre; no primeiro fim de semana, o atacarejo do mês.
    let stores = g.s.list("demodados-mercados");
    let main = stores
        .get((seed_from(p) % stores.len().max(1) as u64) as usize)
        .cloned()
        .unwrap_or_default();
    let bulk = g.t("demodados-atacarejo");
    let saturdays = days_of(month, &[Weekday::Sat]);
    let mut weights: Vec<i64> = saturdays.iter().map(|_| 100).collect();
    if let Some(first) = weights.first_mut() {
        *first = 180;
    }
    let sum: i64 = weights.iter().sum();
    let mut left = big;
    for (i, (sat, w)) in saturdays.iter().zip(&weights).enumerate() {
        let v = if i + 1 == saturdays.len() {
            left
        } else {
            g.rng.vary(big * w / sum, 0.12)
        };
        left -= v;
        // Às vezes fica para o domingo.
        let day = if g.rng.int(0, 3) == 0 {
            sat.succ_opt()
                .filter(|d| d.month() == sat.month())
                .unwrap_or(*sat)
        } else {
            *sat
        };
        let store = if i == 0 { bulk.clone() } else { main.clone() };
        let acc = if i % 2 == 0 {
            card_or_debit(a)
        } else {
            a.checking.clone()
        };
        let who = someone(g);
        g.txn(&acc, day, -v, "Alimentação › Mercado", &store, who.as_ref())?;
    }
    let n = g.rng.int(3, 5) as usize;
    let days = pick_days(g, month, &WEEKDAYS, n);
    let parts = split(&mut g.rng, local, days.len());
    for (d, v) in days.into_iter().zip(parts) {
        let name = g.pick_text("demodados-mercado-bairro");
        let who = someone(g);
        g.txn(
            &a.checking,
            d,
            -v,
            "Alimentação › Mercado",
            &name,
            who.as_ref(),
        )?;
    }
    // Padaria: quase sempre no fim de semana, às vezes num dia útil.
    let (nw, nd) = (g.rng.int(5, 8) as usize, g.rng.int(1, 3) as usize);
    let weekend = pick_days(g, month, &[Weekday::Sat, Weekday::Sun], nw);
    let weekday = pick_days(g, month, &WEEKDAYS, nd);
    let mut days: Vec<NaiveDate> = weekend.into_iter().chain(weekday).collect();
    days.sort();
    let parts = split(&mut g.rng, bakery, days.len());
    let name = g.t("demodados-padaria");
    for (d, v) in days.into_iter().zip(parts) {
        g.txn(&a.checking, d, -v, "Alimentação › Padaria", &name, None)?;
    }
    Ok(())
}

/// Item do cupom: descrição como sai na NFC-e, unidade, preço (centavos) e quantidade máxima
/// (em unidades, ou décimos de quilo para o que é pesado).
struct Item {
    name: &'static str,
    unit: &'static str,
    price: i64,
    max: u32,
}

const fn un(name: &'static str, price: i64, max: u32) -> Item {
    Item {
        name,
        unit: "UN",
        price,
        max,
    }
}

const fn kg(name: &'static str, price: i64, max_tenths: u32) -> Item {
    Item {
        name,
        unit: "KG",
        price,
        max: max_tenths,
    }
}

/// Compra de mercado típica no Brasil, com preços de 2026.
const ITEMS: &[Item] = &[
    un("ARROZ BRANCO TIPO 1 5KG", 2899, 1),
    un("FEIJAO CARIOCA TIPO 1 1KG", 899, 2),
    un("LEITE UHT INTEGRAL 1L", 549, 12),
    un("CAFE TORRADO E MOIDO 500G", 2490, 2),
    un("ACUCAR REFINADO 1KG", 479, 2),
    un("OLEO DE SOJA 900ML", 799, 2),
    un("MACARRAO ESPAGUETE 500G", 459, 3),
    un("MOLHO DE TOMATE TRADICIONAL 340G", 329, 4),
    un("PAO DE FORMA TRADICIONAL 500G", 899, 2),
    un("MANTEIGA COM SAL 200G", 1290, 1),
    un("OVOS BRANCOS GRANDES C/12", 1190, 2),
    un("IOGURTE NATURAL 170G", 329, 6),
    un("REQUEIJAO CREMOSO 200G", 899, 1),
    un("SUCO DE LARANJA INTEGRAL 1L", 1299, 2),
    un("REFRIGERANTE COLA 2L", 999, 2),
    un("AGUA MINERAL S/GAS 1,5L", 299, 6),
    un("BISCOITO RECHEADO CHOCOLATE 130G", 399, 3),
    un("BOLACHA AGUA E SAL 350G", 599, 2),
    un("ACHOCOLATADO EM PO 400G", 899, 1),
    un("FARINHA DE TRIGO 1KG", 599, 1),
    un("SAL REFINADO 1KG", 299, 1),
    un("ALFACE CRESPA", 399, 2),
    un("DETERGENTE LIQUIDO 500ML", 289, 3),
    un("SABAO EM PO 1,6KG", 2690, 1),
    un("AMACIANTE CONCENTRADO 500ML", 1690, 1),
    un("PAPEL HIGIENICO FOLHA DUPLA C/12", 2290, 1),
    un("AGUA SANITARIA 2L", 799, 1),
    un("ESPONJA MULTIUSO C/3", 549, 1),
    un("CREME DENTAL 90G", 549, 2),
    un("SABONETE EM BARRA 85G", 249, 6),
    un("SHAMPOO 350ML", 1890, 1),
    un("DESODORANTE AEROSOL 150ML", 1590, 1),
    un("CERVEJA LATA 350ML", 399, 12),
    kg("PEITO DE FRANGO RESFRIADO", 2199, 25),
    kg("CARNE MOIDA PATINHO", 4590, 10),
    kg("ALCATRA BOVINA", 5990, 12),
    kg("LINGUICA TOSCANA", 2790, 10),
    kg("QUEIJO MUSSARELA FATIADO", 4990, 5),
    kg("PRESUNTO COZIDO FATIADO", 3990, 4),
    kg("BANANA PRATA", 699, 20),
    kg("TOMATE ITALIANO", 899, 15),
    kg("CEBOLA", 549, 15),
    kg("BATATA INGLESA", 599, 20),
    kg("MACA GALA", 1099, 15),
    kg("LARANJA PERA", 499, 30),
    kg("CENOURA", 599, 10),
];

/// Cupom fiscal da compra de mercado (itens até chegar ao valor pago; o último, pesado, fecha a
/// conta), ligado ao lançamento. `factor` sobe os preços de partida (inflação do exemplo), e os
/// mesmos produtos voltam em cupons diferentes, para a tela de Alimentação mostrar o preço por
/// kg ao longo do tempo.
pub(super) fn receipt(
    g: &mut Gen<'_, '_>,
    txn_id: &Id,
    day: NaiveDate,
    store: &str,
    paid: i64,
    factor: f64,
) -> Result<()> {
    if paid < 2_000 {
        return Ok(());
    }
    let mut order: Vec<usize> = (0..ITEMS.len()).collect();
    // Embaralha (Fisher–Yates) com o sorteio do cofre.
    for i in (1..order.len()).rev() {
        let j = (g.rng.next() % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    let mut items: Vec<ManualItem> = vec![];
    let mut sum = 0i64;
    let mut used_kg = None;
    for &i in &order {
        let it = &ITEMS[i];
        let price = (it.price as f64 * factor).round() as i64;
        if it.unit == "KG" && used_kg.is_none() {
            // O primeiro item pesado fica para fechar a conta.
            used_kg = Some(i);
            continue;
        }
        let (qty, total) = if it.unit == "KG" {
            let tenths = g.rng.int(3, it.max.max(3));
            let q = f64::from(tenths) / 10.0 + f64::from(g.rng.int(0, 99)) / 1000.0;
            (q, (q * price as f64).round() as i64)
        } else {
            let q = g.rng.int(1, it.max);
            (f64::from(q), price * i64::from(q))
        };
        // Deixa pelo menos R$ 8 para o item pesado do fim.
        if sum + total > paid - 800 {
            continue;
        }
        sum += total;
        items.push(item(it, qty, price, total));
        if sum > paid * 85 / 100 {
            break;
        }
    }
    let Some(last) = used_kg.map(|i| &ITEMS[i]) else {
        return Ok(());
    };
    let last_price = (last.price as f64 * factor).round() as i64;
    let rest = paid - sum;
    let qty = ((rest as f64 / last_price as f64) * 1000.0).floor() / 1000.0;
    if qty < 0.05 {
        return Ok(());
    }
    items.push(ManualItem {
        description: last.name.to_string(),
        quantity: Some(decimal(qty, 3)),
        unit: Some(last.unit.to_string()),
        unit_price: Some(last_price),
        total: rest,
        taxable: None,
    });
    crate::receipts::import(
        g.tx,
        &ReceiptArgs {
            manual: Some(ManualReceipt {
                date: fmt_date(day),
                store_name: Some(store.to_string()),
                items,
                discount: None,
                paid: Some(paid),
                tax_total: None,
                from_photo: None,
            }),
            txn_id: Some(txn_id.clone()),
            ..Default::default()
        },
        day,
    )?;
    Ok(())
}

fn item(it: &Item, qty: f64, price: i64, total: i64) -> ManualItem {
    ManualItem {
        description: it.name.to_string(),
        quantity: Some(if it.unit == "KG" {
            decimal(qty, 3)
        } else {
            decimal(qty, 0)
        }),
        unit: Some(it.unit.to_string()),
        unit_price: Some(price),
        total,
        taxable: None,
    }
}

/// Número com vírgula decimal, como sai na NFC-e.
fn decimal(v: f64, places: usize) -> String {
    format!("{v:.places$}").replace('.', ",")
}

// ---------------------------------------------------------------------------
// Lazer: café nos dias úteis, restaurante no fim de semana

/// Restaurantes da sexta ao domingo (25%), café e lanche nos dias úteis de quem trabalha (15%),
/// delivery à noite (25%), cinema e shows (20%) e passeios no fim de semana (15%).
pub(super) fn leisure(g: &mut Gen<'_, '_>, a: &Accounts, month: NaiveDate) -> Result<()> {
    let p = g.p;
    if p.leisure <= 0 {
        return Ok(());
    }
    let total = g.rng.vary(p.leisure, 0.2);
    let card = card_or_debit(a);
    let working = p
        .adults
        .iter()
        .any(|x| x.retired != Some(true) && x.income > 0);
    let weekend = [Weekday::Fri, Weekday::Sat, Weekday::Sun];
    let snacks = if working { total * 15 / 100 } else { 0 };
    let parts: [(&str, i64, &[Weekday], u32, &str); 5] = [
        (
            "Alimentação › Restaurantes",
            total * 25 / 100 + (total * 15 / 100 - snacks),
            &weekend,
            3,
            "demodados-restaurantes",
        ),
        (
            "Alimentação › Restaurantes",
            snacks,
            &WEEKDAYS,
            8,
            "demodados-lanches",
        ),
        (
            "Alimentação › Delivery",
            total * 25 / 100,
            &[
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
                Weekday::Sat,
                Weekday::Sun,
            ],
            3,
            "demodados-delivery",
        ),
        (
            "Lazer › Cinema e shows",
            total * 20 / 100,
            &weekend,
            2,
            "demodados-cinema",
        ),
        (
            "Lazer › Passeios",
            total - total * 85 / 100,
            &[Weekday::Sat, Weekday::Sun],
            1,
            "demodados-passeios",
        ),
    ];
    for (cat, amount, days, n, names) in parts {
        if amount <= 0 {
            continue;
        }
        let n = if names == "demodados-lanches" {
            g.rng.int(6, n + 2) as usize
        } else {
            n as usize
        };
        let days = pick_days(g, month, days, n);
        let values = split(&mut g.rng, amount, days.len());
        for (d, v) in days.into_iter().zip(values) {
            let name = g.pick_text(names);
            let (acc, who) = if names == "demodados-lanches" {
                (a.checking.clone(), someone(g))
            } else {
                (card.clone(), None)
            };
            g.txn(&acc, d, -v, cat, &name, who.as_ref())?;
        }
    }
    // Uma viagem por ano, em julho.
    if month.month() == 7 {
        let text = g.t("demodados-viagem");
        g.txn(
            &card,
            on(month, 12),
            -(p.leisure * 3),
            "Lazer › Viagens",
            &text,
            None,
        )?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Assinaturas

/// Serviços de cada perfil, sempre os mesmos e no mesmo dia (para o app reconhecer a
/// assinatura), com um reajuste de 12% na última cobrança do primeiro serviço.
pub(super) fn subscriptions(g: &mut Gen<'_, '_>, a: &Accounts, month: NaiveDate) -> Result<()> {
    let p = g.p;
    if p.subscriptions <= 0 {
        return Ok(());
    }
    let n = match kind(g) {
        DemoKind::Comfortable => 4,
        DemoKind::Single => 3,
        DemoKind::LowIncome => 1,
        DemoKind::Struggling | DemoKind::Retiree | DemoKind::Custom => 2,
    };
    // Sorteio próprio: os mesmos serviços, valores e dias em todos os meses.
    let mut r = Rng(seed_from(p) ^ 0x5B5C_21F7);
    let mut list = g.s.list("demodados-streaming");
    let mut chosen = vec![];
    while chosen.len() < n && !list.is_empty() {
        let i = (r.next() % list.len() as u64) as usize;
        chosen.push(list.remove(i));
    }
    let weights: Vec<i64> = chosen.iter().map(|_| i64::from(r.int(6, 14))).collect();
    let days: Vec<u32> = chosen.iter().map(|_| r.int(2, 26)).collect();
    let sum: i64 = weights.iter().sum::<i64>().max(1);
    // O reajuste vale a partir da última cobrança já feita do primeiro serviço.
    let this_month = NaiveDate::from_ymd_opt(g.today.year(), g.today.month(), 1).unwrap_or(g.today);
    let raise_from = match days.first() {
        Some(d) if on(this_month, *d) <= g.today => this_month,
        _ => add_months(this_month, -1),
    };
    let card = card_or_debit(a);
    for (i, name) in chosen.iter().enumerate() {
        let mut v = p.subscriptions * weights[i] / sum;
        if i == 0 && month >= raise_from {
            v = v * 112 / 100;
        }
        // Centavos de preço de vitrine (…,90).
        let v = (v / 100) * 100 + 90;
        g.txn(
            &card,
            on(month, days[i]),
            -v,
            "Assinaturas › Streaming",
            name,
            None,
        )?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// O próprio de cada perfil

/// Diarista, academia, pet, salão, mesada, gás de cozinha, doações, presentes e a ajuda da
/// família, conforme o perfil (fração da renda).
pub(super) fn extras(
    g: &mut Gen<'_, '_>,
    a: &Accounts,
    month: NaiveDate,
    index: u32,
) -> Result<()> {
    let k = kind(g);
    if k == DemoKind::Custom || family_income(g) <= 0 {
        return Ok(());
    }
    let card = card_or_debit(a);
    let people = g.s.list("demodados-pessoas");
    let person = |i: usize| {
        people
            .get(i % people.len().max(1))
            .cloned()
            .unwrap_or_default()
    };
    match k {
        DemoKind::Comfortable => {
            // Diarista a cada duas semanas, por transferência no mesmo dia.
            let maid = person(0);
            for d in pick_days(g, month, &[Weekday::Wed], 2) {
                let v = of_income(g, 12, 0.0);
                let text = sent(g, &maid);
                g.txn(
                    &a.checking,
                    d,
                    -v,
                    "Moradia › Diarista e serviços domésticos",
                    &text,
                    None,
                )?;
            }
            gym(g, &card, month, 8)?;
            pet(g, &card, month)?;
            salon(g, a, month, 6, 2, index)?;
            allowance(g, a, month)?;
            gifts(g, &card, month, 40)?;
        }
        DemoKind::LowIncome => {
            cooking_gas(g, a, month, index)?;
            salon(g, a, month, 7, 1, index)?;
            // Farmácia sem plano de saúde, no débito.
            let v = of_income(g, 10, 0.4);
            let store = g.pick_text("demodados-farmacias");
            let d = on(month, g.rng.int(2, 27));
            let who = someone(g);
            g.txn(&a.checking, d, -v, "Saúde › Farmácia", &store, who.as_ref())?;
            kids_school_and_clothes(g, &card, month)?;
            gifts(g, &card, month, 30)?;
        }
        DemoKind::Struggling => {
            cooking_gas(g, a, month, index)?;
            // Tarifa do pacote de serviços do banco, todo mês.
            let fee = (family_income(g) * 6 / 1000 / 100) * 100 + 90;
            let text = g.t("demodados-tarifa-pacote");
            g.txn(
                &a.checking,
                on(month, 1),
                -fee,
                "Impostos e taxas › Tarifas bancárias",
                &text,
                None,
            )?;
            // A ajuda da família no fim do mês, em metade dos meses.
            if index % 2 == 1 {
                let v = of_income(g, 60, 0.3);
                let who = person(3);
                let text = received(g, &who);
                let d = on(month, g.rng.int(22, 27));
                g.txn(
                    &a.checking,
                    d,
                    v,
                    "Presentes recebidos › Ajuda de familiares",
                    &text,
                    None,
                )?;
            }
        }
        DemoKind::Single => {
            gym(g, &card, month, 20)?;
            salon(g, a, month, 6, 1, index)?;
            // Happy hour na sexta, pago no cartão; os amigos devolvem a parte deles.
            let n = g.rng.int(1, 2) as usize;
            for d in pick_days(g, month, &[Weekday::Fri], n) {
                let v = of_income(g, 20, 0.3);
                let bar = g.pick_text("demodados-bares");
                g.txn(&card, d, -v, "Alimentação › Restaurantes", &bar, None)?;
                let friend = person(1 + g.rng.int(0, 1) as usize);
                let text = received(g, &friend);
                let back = d.succ_opt().unwrap_or(d);
                g.txn(
                    &a.checking,
                    back,
                    v / 2,
                    "Reembolsos › Divisão de contas",
                    &text,
                    None,
                )?;
            }
        }
        DemoKind::Retiree => {
            // Farmácia toda semana.
            let n = g.rng.int(3, 4) as usize;
            let days = pick_days(g, month, &WEEKDAYS, n);
            let total = of_income(g, 25, 0.2);
            let values = split(&mut g.rng, total, days.len());
            for (d, v) in days.into_iter().zip(values) {
                let store = g.pick_text("demodados-farmacias");
                let who = someone(g);
                g.txn(&card, d, -v, "Saúde › Farmácia", &store, who.as_ref())?;
            }
            if index.is_multiple_of(3) {
                let v = of_income(g, 30, 0.2);
                let text = g.t("demodados-consulta");
                let d = on(month, g.rng.int(5, 20));
                g.txn(&a.checking, d, -v, "Saúde › Consultas", &text, None)?;
            }
            let v = of_income(g, 10, 0.0);
            let text = g.t("demodados-doacao");
            g.txn(
                &a.checking,
                on(month, 10),
                -v,
                "Presentes e doações › Doações",
                &text,
                None,
            )?;
            salon(g, a, month, 10, 1, index)?;
            if matches!(month.month(), 4 | 10) {
                let v = of_income(g, 40, 0.2);
                let text = g.t("demodados-presente-neto");
                g.txn(
                    &card,
                    on(month, 15),
                    -v,
                    "Presentes e doações › Presentes",
                    &text,
                    None,
                )?;
            }
            gifts(g, &card, month, 50)?;
        }
        DemoKind::Custom => {}
    }
    Ok(())
}

/// Academia no cartão, todo mês no mesmo dia e pelo mesmo valor.
fn gym(g: &mut Gen<'_, '_>, card: &str, month: NaiveDate, per_mille: i64) -> Result<()> {
    let v = (family_income(g) * per_mille / 1000 / 100) * 100 + 90;
    let name = g.t("demodados-academia");
    g.txn(
        card,
        on(month, 5),
        -v,
        "Cuidados pessoais › Academia",
        &name,
        None,
    )?;
    Ok(())
}

/// Ração todo mês e o veterinário duas vezes por ano.
fn pet(g: &mut Gen<'_, '_>, card: &str, month: NaiveDate) -> Result<()> {
    let v = of_income(g, 5, 0.1);
    let shop = g.t("demodados-pet-shop");
    let d = on(month, g.rng.int(3, 25));
    g.txn(card, d, -v, "Pets › Ração", &shop, None)?;
    if matches!(month.month(), 4 | 10) {
        let v = of_income(g, 15, 0.3);
        let vet = g.t("demodados-veterinario");
        g.txn(card, on(month, 18), -v, "Pets › Veterinário", &vet, None)?;
    }
    Ok(())
}

/// Salão ou barbearia a cada `every` meses.
fn salon(
    g: &mut Gen<'_, '_>,
    a: &Accounts,
    month: NaiveDate,
    per_mille: i64,
    every: u32,
    index: u32,
) -> Result<()> {
    if !index.is_multiple_of(every) {
        return Ok(());
    }
    let v = of_income(g, per_mille, 0.15);
    let d = pick_days(g, month, &[Weekday::Sat], 1)
        .first()
        .copied()
        .unwrap_or(on(month, 15));
    let name = g.t("demodados-salao");
    let who = someone(g);
    g.txn(
        &a.checking,
        d,
        -v,
        "Cuidados pessoais › Cabelo e estética",
        &name,
        who.as_ref(),
    )?;
    Ok(())
}

/// Mesada dos filhos de 6 anos ou mais, no dia 1, por transferência.
fn allowance(g: &mut Gen<'_, '_>, a: &Accounts, month: NaiveDate) -> Result<()> {
    let (_, kids) = ages(g.p);
    for (child, age) in g.p.children.iter().zip(kids) {
        if age < 6 {
            continue;
        }
        let v = (family_income(g) * i64::from(age) / 2500 / 100) * 100;
        let text = sent(g, child.trim());
        let who = g.member(child);
        g.txn(
            &a.checking,
            on(month, 1),
            -v,
            "Filhos › Mesada",
            &text,
            who.as_ref(),
        )?;
    }
    Ok(())
}

/// Botijão de gás a cada 45 dias, mais ou menos (dois meses em cada três).
fn cooking_gas(g: &mut Gen<'_, '_>, a: &Accounts, month: NaiveDate, index: u32) -> Result<()> {
    if index % 3 == 2 {
        return Ok(());
    }
    let v = of_income(g, 32, 0.05);
    let name = g.t("demodados-gas");
    let d = on(month, g.rng.int(3, 26));
    g.txn(&a.checking, d, -v, "Moradia › Gás", &name, None)?;
    Ok(())
}

/// Escola pública: material escolar em fevereiro e roupas e calçados das crianças em março e
/// setembro, no cartão.
fn kids_school_and_clothes(g: &mut Gen<'_, '_>, card: &str, month: NaiveDate) -> Result<()> {
    let p = g.p;
    for child in &p.children {
        let who = g.member(child);
        if month.month() == 2 {
            let v = of_income(g, 40, 0.2);
            let text = g.ta("demodados-material", &[("nome", child.trim().to_string())]);
            g.txn(
                card,
                on(month, 3),
                -v,
                "Educação › Material escolar",
                &text,
                who.as_ref(),
            )?;
        }
        if matches!(month.month(), 3 | 9) {
            let v = of_income(g, 35, 0.2);
            let store = g.t("demodados-loja-roupa");
            let d = on(month, g.rng.int(5, 25));
            g.txn(
                card,
                d,
                -v,
                "Filhos › Roupas e calçados infantis",
                &store,
                who.as_ref(),
            )?;
        }
    }
    Ok(())
}

/// Presentes de Natal em dezembro.
fn gifts(g: &mut Gen<'_, '_>, card: &str, month: NaiveDate, per_mille: i64) -> Result<()> {
    if month.month() != 12 {
        return Ok(());
    }
    let total = of_income(g, per_mille, 0.2);
    let days = pick_days(g, month, &WEEKDAYS, 3)
        .into_iter()
        .filter(|d| d.day() <= 23)
        .collect::<Vec<_>>();
    let values = split(&mut g.rng, total, days.len());
    for (d, v) in days.into_iter().zip(values) {
        let store = g.pick_text("demodados-lojas-presente");
        g.txn(card, d, -v, "Presentes e doações › Presentes", &store, None)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Compra parcelada

/// Uma compra grande parcelada em 10 vezes no cartão, no meio do histórico (perfis com cartão,
/// menos a aposentadoria; na baixa renda, o eletrodoméstico do crediário).
pub(super) fn installment_purchase(g: &mut Gen<'_, '_>, a: &Accounts) -> Result<()> {
    let p = g.p;
    let Some(card) = &a.card else {
        return Ok(());
    };
    if !matches!(
        kind(g),
        DemoKind::Comfortable | DemoKind::Single | DemoKind::Struggling | DemoKind::LowIncome
    ) || p.shopping <= 0
    {
        return Ok(());
    }
    let month = add_months(g.start, (p.months / 2) as i32);
    let day = pick_days(g, month, &[Weekday::Sat], 1)
        .first()
        .copied()
        .unwrap_or(on(month, 14));
    if day > g.today {
        return Ok(());
    }
    let total = (p.shopping * 6 / 1000) * 1000 + 990;
    let what = g.pick_text("demodados-compra-parcelada");
    let member = adults(g).into_iter().flatten().next();
    let mut v = json!({
        "account_id": card,
        "description": what,
        "total_amount": total,
        "installments": 10,
        "purchase_date": fmt_date(day),
        "category_id": g.cat("Compras › Eletrônicos")?,
    });
    if let Some(m) = member {
        v["member_id"] = json!(m);
    }
    crate::cards::create_installments(g.tx, &input(v)?, g.today)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// FGTS

/// TR mensal usada no JAM do exemplo (0,15%).
const TR_MONTH: f64 = 0.0015;

/// Conta FGTS de quem tem carteira assinada no Brasil: saldo dos anos de empresa antes do
/// histórico (sem data de admissão, para a conferência não cobrar os depósitos de antes), o depósito de 8% do bruto no dia 7 (competência do mês anterior, igual ao
/// holerite), o JAM no dia 10 pela regra da Caixa (sobre o saldo do crédito anterior) e a
/// distribuição de resultado em agosto.
pub(super) fn fgts(
    g: &mut Gen<'_, '_>,
    member: &Id,
    adult_index: usize,
    source: &Id,
    employer: &str,
    gross: i64,
) -> Result<()> {
    if !g.s.brazil() || gross <= 0 {
        return Ok(());
    }
    let (adult_ages, _) = ages(g.p);
    let age = adult_ages.get(adult_index).copied().unwrap_or(35);
    let max_years = age.saturating_sub(20).clamp(1, 12);
    let years = g.rng.int(1.min(max_years), max_years.min(9));
    let coef = (1.0 + TR_MONTH) * (1.0 + crate::fgts::data::ANNUAL_INTEREST).powf(1.0 / 12.0) - 1.0;
    let coef_text = format!("{coef:.6}");
    let coef: f64 = coef_text.parse().unwrap_or(coef);
    let deposit = gross * 8 / 100;
    // Saldo antes do histórico: depósitos de um salário menor, com o JAM.
    let months_before = (years * 12) as i64;
    let mut opening = 0f64;
    for m in 0..months_before {
        let dep = deposit as f64 * (0.75 + 0.25 * m as f64 / months_before.max(1) as f64);
        opening = opening * (1.0 + coef) + dep;
    }
    let opening = opening.round() as i64;
    let open_date = g.start.pred_opt().unwrap_or(g.start);
    let account = g.account(json!({
        "name": g.ta("demodados-fgts", &[("empresa", employer.to_string())]),
        "kind": "fgts",
        "institution_name": "Caixa",
        "member_ids": [member],
        "opening_balance": opening,
        "opening_date": fmt_date(open_date),
    }))?;
    crate::fgts::update_account(
        g.tx,
        &input(json!({
            "account_id": account,
            "employer_name": employer,
            "income_source_id": source,
        }))?,
    )?;
    // Lançamentos em ordem, com o saldo para o JAM.
    let mut lines: Vec<(NaiveDate, i64)> = vec![];
    let mut last_jam: Option<NaiveDate> = None;
    let balance_until = |lines: &[(NaiveDate, i64)], until: NaiveDate| -> i64 {
        opening
            + lines
                .iter()
                .filter(|(d, _)| *d <= until)
                .map(|(_, v)| v)
                .sum::<i64>()
    };
    for month in g.months() {
        let dep_day = on(month, 7);
        if dep_day <= g.today {
            let comp = add_months(month, -1);
            let competence = format!("{:04}-{:02}", comp.year(), comp.month());
            let text = g.ta(
                "demodados-fgts-deposito",
                &[("mes", format!("{:02}/{:04}", comp.month(), comp.year()))],
            );
            let mut e = crate::fgts::NewEntry::of(crate::fgts::EntryKind::Deposit);
            e.competence = Some(competence);
            crate::fgts::insert_entry(
                g.tx,
                &account,
                &fmt_date(dep_day),
                deposit,
                &text,
                &e,
                None,
                g.today,
            )?;
            lines.push((dep_day, deposit));
        }
        let jam_day = on(month, 10);
        if jam_day <= g.today {
            let base = balance_until(&lines, last_jam.unwrap_or(open_date));
            let v = (base.max(0) as f64 * coef + 1e-6).floor() as i64;
            let mut e = crate::fgts::NewEntry::of(crate::fgts::EntryKind::Jam);
            e.coefficient = Some(coef_text.clone());
            let text = g.ta(
                "demodados-fgts-jam",
                &[("coef", coef_text.replace('.', ","))],
            );
            crate::fgts::insert_entry(
                g.tx,
                &account,
                &fmt_date(jam_day),
                v,
                &text,
                &e,
                None,
                g.today,
            )?;
            lines.push((jam_day, v));
            last_jam = Some(jam_day);
        }
        let dist_day = on(month, 20);
        if month.month() == 8 && dist_day <= g.today {
            let base_year = month.year() - 1;
            let index = crate::fgts::data::distribution(base_year)
                .map(|d| d.index)
                .unwrap_or_else(crate::fgts::data::estimated_distribution);
            let dec31 = NaiveDate::from_ymd_opt(base_year, 12, 31).unwrap_or(open_date);
            let v = (balance_until(&lines, dec31).max(0) as f64 * index).floor() as i64;
            if v > 0 {
                let mut e = crate::fgts::NewEntry::of(crate::fgts::EntryKind::Distribution);
                e.base_year = Some(base_year);
                let text = g.ta(
                    "demodados-fgts-distribuicao",
                    &[("ano", base_year.to_string())],
                );
                crate::fgts::insert_entry(
                    g.tx,
                    &account,
                    &fmt_date(dist_day),
                    v,
                    &text,
                    &e,
                    None,
                    g.today,
                )?;
                lines.push((dist_day, v));
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Orçamento do que é próprio do perfil

/// O que [`extras`] e a compra parcelada gastam por mês, por categoria-mãe, para entrar no
/// orçamento do mês junto com o que a pessoa informou.
pub(super) fn budget_extras(g: &Gen<'_, '_>) -> Vec<(&'static str, i64)> {
    let inc = family_income(g);
    let pm = |per_mille: i64| inc * per_mille / 1000;
    let mut out: Vec<(&'static str, i64)> = match kind(g) {
        DemoKind::Comfortable => {
            let (_, kids) = ages(g.p);
            let allowance: i64 = kids
                .iter()
                .filter(|a| **a >= 6)
                .map(|a| inc * i64::from(*a) / 2500)
                .sum();
            vec![
                ("Moradia", pm(24)),
                ("Cuidados pessoais", pm(8) + pm(3)),
                ("Pets", pm(5) + pm(3)),
                ("Filhos", allowance),
            ]
        }
        DemoKind::LowIncome => vec![
            ("Moradia", pm(22)),
            ("Cuidados pessoais", pm(7)),
            ("Saúde", pm(10)),
        ],
        DemoKind::Struggling => vec![("Moradia", pm(22)), ("Impostos e taxas", pm(6))],
        DemoKind::Single => vec![("Cuidados pessoais", pm(26)), ("Alimentação", pm(30))],
        DemoKind::Retiree => vec![
            ("Saúde", pm(35)),
            ("Presentes e doações", pm(10)),
            ("Cuidados pessoais", pm(10)),
        ],
        DemoKind::Custom => vec![],
    };
    // Parcela da compra parcelada, enquanto durar.
    let plan: i64 =
        g.tx.query_row(
            "SELECT COALESCE(SUM(-amount), 0) FROM txn
              WHERE installment_plan_id IS NOT NULL AND substr(date, 1, 7) = ?1",
            [format!("{:04}-{:02}", g.today.year(), g.today.month())],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if plan > 0 {
        out.push(("Compras", plan));
    }
    out
}
