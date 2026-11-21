//! A cesta com cupons inventados: só o café sobe, o resto fica igual.

use chrono::NaiveDate;
use serde_json::json;

use super::{BasketArgs, basket};
use crate::food::tests::{api, danfe};

#[test]
fn same_products_same_store_month_by_month() {
    let (_d, api, _acc, _cats) = api();
    for (date, coffee, paid) in [
        ("10/01/2026", "20,00", "70,00"),
        ("10/02/2026", "22,00", "72,00"),
        ("10/03/2026", "24,00", "74,00"),
    ] {
        api.call(
            "receipts.import",
            json!({"text": danfe(date, coffee, paid)}),
        )
        .unwrap();
    }
    let today = NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();
    let b = api
        .with_session(|s, _| basket(s.db.conn(), &BasketArgs { months: Some(3) }, today))
        .unwrap();
    assert_eq!(b.months.len(), 3);
    assert!(b.enough);
    assert_eq!(b.months[0].index, 100.0);
    // Fevereiro: o café (+10%) numa cesta em que o resto não mudou.
    let feb = b.months[1].change.unwrap();
    assert!(feb > 0.0 && feb < 0.10, "{feb}");
    assert!(b.months[2].index > b.months[1].index);
    assert_eq!(b.risers.len(), 1);
    assert!(
        b.risers[0].name.to_lowercase().contains("cafe"),
        "{:?}",
        b.risers[0].name
    );
    assert_eq!(b.risers[0].before, 4000, "R$ 20 por 500 g = R$ 40/kg");
    assert_eq!(b.risers[0].now, 4800);
    assert!(b.fallers.is_empty());
    // Sem um ano de compras, não há "as mesmas compras um ano antes".
    assert_eq!(b.same_basket_year_ago, None);
    // Sem o IPCA de alimentos baixado, sem comparação.
    assert_eq!(b.official_change, None);
}

/// A série oficial de alimentos de cada país: taxa mensal no Brasil, nível (índice) nos
/// outros, virando a variação de um mês para o outro; sem a principal, vale a reserva.
#[test]
fn official_food_series_by_country() {
    use crate::storage::Db;
    use crate::vault::KDF_TEST;
    let dir = tempfile::tempdir().unwrap();
    let db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
    let c = db.conn();
    let d = |y, m| NaiveDate::from_ymd_opt(y, m, 1).unwrap();
    let put = |series: &str, date: NaiveDate, value: &str| {
        c.execute(
            "INSERT INTO index_value (series_id, date, value, estimated, source, fetched_at)
             VALUES (?1, ?2, ?3, 0, 'teste', '2026-11-17T00:00:00Z')",
            rusqlite::params![series, crate::time::fmt_date(date), value],
        )
        .unwrap();
    };
    // Toda série do perfil existe no catálogo (V138 e V164).
    for country in ["BR", "US", "PT", "ES", "FR", "DE", "GB", "CA", "CL"] {
        for s in crate::food::profile::of(country).official_series {
            let n: i64 = c
                .query_row(
                    "SELECT count(*) FROM index_series WHERE id = ?1",
                    [&s],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(n, 1, "{country}: {s}");
        }
    }
    // EUA sem a chave do FRED: vale a reserva da OCDE, de nível para variação.
    put("cpi_food_us_oecd", d(2026, 8), "100");
    put("cpi_food_us_oecd", d(2026, 9), "101");
    put("cpi_food_us_oecd", d(2026, 10), "103.02");
    let today = NaiveDate::from_ymd_opt(2026, 11, 17).unwrap();
    let (series, rates) = super::official_rates(c, "US", d(2026, 9), today)
        .unwrap()
        .unwrap();
    assert_eq!(series, "cpi_food_us_oecd");
    assert!((rates["2026-09"] - 0.01).abs() < 1e-9);
    assert!((rates["2026-10"] - 0.02).abs() < 1e-9);
    // Com o "Food at home" do FRED, ele vem primeiro.
    put("us_cpi_food_home", d(2026, 9), "300");
    put("us_cpi_food_home", d(2026, 10), "303");
    let (series, rates) = super::official_rates(c, "US", d(2026, 10), today)
        .unwrap()
        .unwrap();
    assert_eq!(series, "us_cpi_food_home");
    assert!((rates["2026-10"] - 0.01).abs() < 1e-9);
    // País sem série: nada (a tela compara com o índice cheio e diz isso).
    assert!(
        super::official_rates(c, "JP", d(2026, 9), today)
            .unwrap()
            .is_none()
    );
}
