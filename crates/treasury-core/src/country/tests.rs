use serde_json::{Value, json};

use crate::api::Api;
use crate::paths::TreasuryPaths;

fn api() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new(paths).unwrap();
    (dir, api)
}

fn create(api: &Api, country: Option<&str>) {
    let mut args = json!({"password": "senha-mestra-forte", "household_name": "Família", "members": ["Ana", "Bruno"]});
    if let Some(c) = country {
        args["country"] = json!(c);
    }
    api.call("vault.create", args).unwrap();
}

fn enabled(api: &Api) -> Vec<String> {
    api.call("sources.list", Value::Null)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["enabled"] == true)
        .map(|s| s["id"].as_str().unwrap().to_string())
        .collect()
}

fn names(api: &Api, what: &str) -> Vec<String> {
    api.call(what, json!({}))
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn a_new_vault_in_the_us_speaks_american() {
    let (_d, api) = api();
    create(&api, Some("us"));
    let h = api.call("household.get", Value::Null).unwrap();
    assert_eq!(h["country"], "US");
    assert_eq!(h["base_currency"], "USD");

    let st = api.call("country.get", Value::Null).unwrap();
    assert_eq!(st["info"]["support"], "adapted");
    assert_eq!(st["info"]["features"]["br_market"], false);
    assert_eq!(st["info"]["features"]["payroll"], true);
    assert_eq!(st["info"]["inflation_label"], "CPI-U");
    assert_eq!(st["info"]["payroll"]["kinds"]["clt"], "Empregado (W-2)");
    assert_eq!(st["choices"][0]["code"], "BR");
    assert_eq!(st["choices"][1]["code"], "US");
    assert_eq!(st["choices"][2]["code"], "PT");
    assert!(st["choices"].as_array().unwrap().len() > 50);

    // Fontes do Brasil desligadas; as globais e as americanas ligadas.
    let on = enabled(&api);
    for id in ["yahoo", "sec", "fred", "ecb", "coingecko"] {
        assert!(on.contains(&id.to_string()), "{id} desligada: {on:?}");
    }
    for id in [
        "b3",
        "bcb",
        "fundamentus",
        "tesouro",
        "cvm",
        "fipe",
        "ibge",
        "eurostat",
    ] {
        assert!(!on.contains(&id.to_string()), "{id} ligada");
    }

    // Categorias e regras prontas de lá.
    let cats = names(&api, "categories.list");
    assert!(cats.contains(&"Imposto predial (property tax)".to_string()));
    assert!(cats.contains(&"401(k) e IRA".to_string()));
    assert!(
        !cats
            .iter()
            .any(|c| c == "IOF" || c == "JCP" || c == "13º salário")
    );
    let rules = names(&api, "rules.list");
    assert!(rules.iter().any(|r| r.starts_with("Zelle")), "{rules:?}");
    assert!(rules.iter().any(|r| r.starts_with("Mercado (Walmart")));
    assert!(!rules.iter().any(|r| r.starts_with("PIX")));

    // Índices e impostos seguem o país.
    let cat = api.call("indices.catalog", Value::Null).unwrap();
    assert_eq!(cat["country"], "US");
    let tax = api.call("tax_burden.settings", Value::Null).unwrap();
    assert_eq!(tax["country"], "US");
}

#[test]
fn switching_country_moves_sources_and_keeps_the_users_choices() {
    let (_d, api) = api();
    create(&api, None);
    let h = api.call("household.get", Value::Null).unwrap();
    assert_eq!(h["country"], "BR");
    let all = enabled(&api);
    // Todas ligadas, menos as que só a pessoa liga (Open Food Facts).
    assert_eq!(
        all.len(),
        crate::sources::SourceId::ALL.len() - crate::sources::OPT_IN.len()
    );
    // Desligada pelo usuário: fica desligada em qualquer país.
    api.call("sources.set", json!({"id": "brapi", "enabled": false}))
        .unwrap();

    let ch = api.call("country.set", json!({"country": "PT"})).unwrap();
    assert_eq!(ch["currency_changed"], true);
    assert!(ch["disabled"].as_array().unwrap().iter().any(|s| s == "b3"));
    assert_eq!(ch["state"]["info"]["tax_dataset"], "EU");
    assert_eq!(ch["state"]["info"]["tax_region"], "PT");
    let on = enabled(&api);
    assert!(on.contains(&"eurostat".to_string()) && on.contains(&"ecb".to_string()));
    assert!(!on.contains(&"fred".to_string()) && !on.contains(&"bcb".to_string()));
    let h = api.call("household.get", Value::Null).unwrap();
    assert_eq!(
        (h["country"].as_str(), h["base_currency"].as_str()),
        (Some("PT"), Some("EUR"))
    );
    let tax = api.call("tax_burden.settings", Value::Null).unwrap();
    assert_eq!(
        (tax["country"].as_str(), tax["region"].as_str()),
        (Some("EU"), Some("PT"))
    );

    // De volta ao Brasil: as fontes que o app desligou voltam; a do usuário, não.
    api.call("country.set", json!({"country": "BR"})).unwrap();
    let on = enabled(&api);
    assert_eq!(
        on.len(),
        crate::sources::SourceId::ALL.len() - 1 - crate::sources::OPT_IN.len()
    );
    assert!(!on.contains(&"brapi".to_string()));
    assert_eq!(
        api.call("household.get", Value::Null).unwrap()["base_currency"],
        "BRL"
    );

    assert!(api.call("country.set", json!({"country": "XX"})).is_err());
}

#[test]
fn with_data_the_base_currency_only_changes_on_request() {
    let (_d, api) = api();
    create(&api, None);
    api.call(
        "accounts.create",
        json!({"name": "Conta", "kind": "checking"}),
    )
    .unwrap();
    let st = api.call("country.get", Value::Null).unwrap();
    assert_eq!(st["has_data"], true);
    let ch = api.call("country.set", json!({"country": "US"})).unwrap();
    assert_eq!(ch["currency_changed"], false);
    assert_eq!(
        api.call("household.get", Value::Null).unwrap()["base_currency"],
        "BRL"
    );
    let ch = api
        .call(
            "country.set",
            json!({"country": "US", "change_currency": true}),
        )
        .unwrap();
    assert_eq!(ch["currency_changed"], true);
    assert_eq!(
        api.call("household.get", Value::Null).unwrap()["base_currency"],
        "USD"
    );
}

/// Família nos EUA: o que é cadastrado sem moeda fica em dólar (o que é brasileiro por natureza,
/// como ações da B3, continua em reais) e os textos do núcleo mostram dólar, nunca real.
#[test]
fn a_us_family_counts_in_dollars() {
    let (_d, api) = api();
    create(&api, Some("US"));
    let h = api.call("household.get", Value::Null).unwrap();
    assert_eq!(h["base_currency"], "USD");

    let acc = api
        .call(
            "accounts.create",
            json!({"name": "Checking", "kind": "checking", "opening_balance": 500_000,
                   "opening_date": "2026-01-01"}),
        )
        .unwrap();
    assert_eq!(acc["currency"], "USD");
    let acc = acc["id"].as_str().unwrap().to_string();
    let eur = api
        .call(
            "accounts.create",
            json!({"name": "Euro account", "kind": "foreign", "currency": "EUR",
                   "opening_date": "2026-01-01"}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let goal = api
        .call(
            "goals.create",
            json!({"name": "Trip", "kind": "travel", "target_amount": 300_000}),
        )
        .unwrap();
    assert_eq!(goal["currency"], "USD");
    let other = api
        .call(
            "assets.create",
            json!({"name": "Art collection", "class": "other"}),
        )
        .unwrap();
    assert_eq!(other["currency"], "USD");
    let petr = api
        .call(
            "assets.create",
            json!({"ticker": "PETR4", "name": "Petrobras", "class": "stock_br"}),
        )
        .unwrap();
    assert_eq!(petr["currency"], "BRL");

    // Contrato: a moeda da conta de débito; sem ela, a da família.
    let loan = |debit: Option<&str>| {
        let mut a = json!({"name": "Car loan", "kind": "vehicle", "principal": 2_000_000,
            "annual_rate": "7", "system": "price", "contract_date": "2026-01-10",
            "first_due_date": "2026-02-10", "term_months": 36});
        if let Some(d) = debit {
            a["debit_account_id"] = json!(d);
        }
        api.call("debt.create", a).unwrap()["currency"].clone()
    };
    assert_eq!(loan(Some(&acc)), "USD");
    assert_eq!(loan(Some(&eur)), "EUR");
    assert_eq!(loan(None), "USD");

    let nw = api.call("reports.net_worth", json!({})).unwrap();
    assert_eq!(nw["base_currency"], "USD");

    // Câmbio: a "moeda da casa" é o dólar (1 EUR = 1,10 USD), com o IOF sobre os dólares.
    let q = api
        .call(
            "fx.exchange_quote",
            json!({"from_account_id": acc, "to_account_id": eur, "date": "2026-10-10",
                   "amount_from": 110_000, "rate": "1.10", "iof_pct": "1"}),
        )
        .unwrap();
    assert_eq!(q["amount_to"], 100_000, "{q}");
    assert_eq!(q["iof_amount"], 1_100, "{q}");
    assert_eq!(q["total_from"], 111_100, "{q}");
    api.call(
        "fx.set",
        json!({"base": "EUR", "quote": "USD", "date": "2026-10-10", "rate": "1.08"}),
    )
    .unwrap();
    let r = api
        .call(
            "fx.reference_rate",
            json!({"currency": "EUR", "date": "2026-10-10"}),
        )
        .unwrap();
    assert_eq!(r["sell"], "1.08", "{r}");
    assert!(
        api.call("fx.reference_rate", json!({"currency": "USD"}))
            .is_err()
    );

    // Textos do núcleo em dólar.
    let plan = api
        .call(
            "debt.payoff_plan",
            json!({"extra_monthly": 10_000, "loan_ids": [],
                   "custom": [{"name": "Card", "balance": 300_000, "annual_rate": "24",
                               "min_payment": 20_000}]}),
        )
        .unwrap();
    let text = plan["explanation"].as_str().unwrap();
    assert!(text.contains("USD") && !text.contains("R$"), "{text}");
    let s = api
        .call(
            "scenarios.simulate",
            json!({"events": [], "years": 2, "monthly_income": 500_000,
                   "monthly_expense": 400_000}),
        )
        .unwrap();
    let text = format!("{} {}", s["explanation"], s["notes"]);
    assert!(text.contains("USD") && !text.contains("R$"), "{text}");
    let e = api
        .call(
            "transactions.create",
            json!({"account_id": acc, "date": "2026-10-01", "amount": -1000,
                   "description": "Store", "splits": [{"amount": -600}, {"amount": -300}]}),
        )
        .unwrap_err()
        .to_string();
    assert!(e.contains("USD") && !e.contains("R$"), "{e}");
}

#[test]
fn portugal_and_other_countries_get_their_names() {
    let pt = crate::categories::seeds_for("PT");
    let all: Vec<&str> = pt
        .iter()
        .flat_map(|(_, g)| {
            g.iter()
                .flat_map(|(n, _, kids)| std::iter::once(*n).chain(kids.iter().copied()))
        })
        .collect();
    for n in [
        "Subsídio de Natal",
        "Subsídio de férias",
        "IMI",
        "IUC e inspeção",
        "Telemóvel",
        "PPR",
    ] {
        assert!(all.contains(&n), "{n}");
    }
    for n in ["JCP", "13º salário", "IPTU", "Bolsa Família"] {
        assert!(!all.contains(&n), "{n}");
    }
    let mx = crate::categories::seeds_for("MX");
    let all: Vec<&str> = mx
        .iter()
        .flat_map(|(_, g)| {
            g.iter()
                .flat_map(|(n, _, kids)| std::iter::once(*n).chain(kids.iter().copied()))
        })
        .collect();
    assert!(all.contains(&"Imposto sobre imóveis") && !all.contains(&"IOF"));
}

#[test]
fn a_generic_country_uses_the_catalog() {
    let (_d, api) = api();
    create(&api, Some("MX"));
    let st = api.call("country.get", Value::Null).unwrap();
    assert_eq!(st["info"]["support"], "generic");
    assert_eq!(st["info"]["currency"], "MXN");
    assert_eq!(st["info"]["features"]["payroll"], false);
    assert_eq!(st["info"]["features"]["tax_burden"], false);
    let on = enabled(&api);
    assert!(!on.contains(&"b3".to_string()));
    assert_eq!(
        api.call("household.get", Value::Null).unwrap()["base_currency"],
        "MXN"
    );
    let rules = names(&api, "rules.list");
    assert!(rules.iter().any(|r| r.starts_with("Streaming")));
    assert!(!rules.iter().any(|r| r.starts_with("Combustível")));
    // O agente sabe onde a família mora.
    let line = api
        .with_session(|s, _| Ok(super::prompt_line(s.db.conn())))
        .unwrap();
    assert!(line.contains("MXN"), "{line}");
}

#[test]
fn canada_is_adapted_with_the_catalog_names() {
    let (_d, api) = api();
    create(&api, Some("CA"));
    let st = api.call("country.get", Value::Null).unwrap();
    assert_eq!(st["info"]["support"], "adapted");
    assert_eq!(st["info"]["currency"], "CAD");
    assert_eq!(st["info"]["features"]["payroll"], true);
    assert_eq!(st["info"]["features"]["tax_burden"], false);
    assert!(enabled(&api).contains(&"boc".to_string()));
    assert_eq!(st["info"]["payroll"]["contribution"], "CPP e EI");
}

#[test]
fn ready_rules_of_each_country_compile_and_match() {
    for c in [
        "BR", "US", "PT", "ES", "GB", "CA", "FR", "DE", "AR", "CL", "XX",
    ] {
        crate::rules::builtin::check_list(c);
    }
}

#[test]
fn the_static_list_is_the_catalog() {
    let dir = tempfile::tempdir().unwrap();
    let db =
        crate::storage::Db::create(&dir.path().join("t.db"), "s", crate::vault::KDF_TEST).unwrap();
    let mut stmt = db
        .conn()
        .prepare("SELECT country, name, currency FROM country_profile WHERE country <> 'EA' ORDER BY country")
        .unwrap();
    let mut table: Vec<(String, String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    table.sort();
    let mut list: Vec<(String, String, String)> = super::catalog::COUNTRIES
        .iter()
        .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()))
        .collect();
    list.sort();
    assert_eq!(list, table);
    // Sem cofre: a lista para a criação.
    let (_d, api) = api();
    let c = api.call("app.countries", Value::Null).unwrap();
    assert_eq!(c[0]["code"], "BR");
    assert_eq!(c.as_array().unwrap().len(), list.len());
}

#[test]
fn texts_lose_what_is_only_brazilian() {
    let t = "Guarde a reserva em aplicação de liquidez diária e baixo risco (Tesouro Selic ou CDB com liquidez diária).";
    assert_eq!(super::localize_text("BR", t), t);
    assert!(super::localize_text("PT", t).contains("Certificados de Aforro"));
    assert!(super::localize_text("US", t).contains("T-bills"));
    let f = super::localize_text(
        "PT",
        "Se o financiamento pesa, simule amortizar com o FGTS ou portar para uma taxa menor em 07 · Dívidas.",
    );
    assert!(!f.contains("FGTS"), "{f}");
    assert_eq!(
        super::localize_text("PT", "Rotativo e cheque especial"),
        "Rotativo e descoberto bancário"
    );
}

/// Os trechos só do Brasil (`nucleo-pais-*-br`) estão, em cada idioma, nos textos de onde
/// [`super::localize_text`] os tira: se alguém mudar a redação de um lado só, a troca para os
/// produtos do país da família para de funcionar.
#[test]
fn brazilian_phrases_match_the_source_texts_in_every_locale() {
    use crate::i18n::{Locale, args, tr};
    let rate = args(&[("taxa", "8,1".into())]);
    for l in [Locale::PtBr, Locale::En, Locale::Es] {
        let k = |key: &str| tr(l, key, None);
        let pairs = [
            ("nucleo-saude-reserva-dica-2", k("nucleo-pais-reserva-br")),
            ("nucleo-saude-moradia-dica-3", k("nucleo-pais-fgts-br")),
            (
                "nucleo-saude-credito-dica-2",
                format!(" {}", k("nucleo-pais-consignado-br")),
            ),
            (
                "insights-vermelho-acao",
                format!(" {}", k("nucleo-pais-juros-mercado-br")),
            ),
            ("insights-tarifas-acao-pacote", k("nucleo-pais-tarifas-br")),
            ("insights-tarifas-acao-ambos", k("nucleo-pais-tarifas-br")),
            (
                "nucleo-saude-credito-dica-2",
                k("nucleo-pais-cheque-especial-br"),
            ),
            (
                "insights-tarifas-acao-credito",
                k("nucleo-pais-cheque-especial-br"),
            ),
        ];
        for (source, phrase) in pairs {
            let text = tr(l, source, Some(&rate));
            assert!(
                text.contains(&phrase),
                "{}: {source} sem o trecho {phrase:?}: {text}",
                l.code()
            );
        }
    }
    // Em inglês, para uma família em Portugal: sem os produtos e as referências do Brasil.
    crate::i18n::set_current(Locale::En);
    let pt = super::localize_text("PT", &tr(Locale::En, "nucleo-saude-reserva-dica-2", None));
    let us = super::localize_text("US", &tr(Locale::En, "insights-tarifas-acao-ambos", None));
    let mx = super::localize_text("MX", &tr(Locale::En, "nucleo-saude-moradia-dica-3", None));
    crate::i18n::set_current(Locale::PtBr);
    assert!(
        pt.contains("Certificados de Aforro") && !pt.contains("Tesouro"),
        "{pt}"
    );
    assert!(!us.contains("CMN") && us.contains("no monthly fee"), "{us}");
    assert!(!mx.contains("FGTS"), "{mx}");
}

/// Nomes dos países, termos da folha e dicas no idioma da instalação; os países do catálogo
/// em ordem alfabética no idioma (os com adaptação continuam primeiro).
#[test]
fn country_texts_follow_the_installation_locale() {
    use crate::i18n::{Locale, set_current};
    let (_d, api) = api();
    create(&api, Some("pt"));
    set_current(Locale::En);
    let st = api.call("country.get", Value::Null).unwrap();
    let choices = api.call("app.countries", Value::Null).unwrap();
    set_current(Locale::PtBr);
    assert_eq!(st["info"]["name"], "Portugal");
    assert_eq!(st["info"]["inflation_label"], "HICP");
    assert_eq!(st["info"]["payroll"]["payslip"], "payslip");
    assert_eq!(st["info"]["payroll"]["payslips"], "payslips");
    assert_eq!(st["info"]["payroll"]["kinds"]["retirement"], "Retirement");
    assert!(
        st["info"]["tips"][0]
            .as_str()
            .unwrap()
            .starts_with("An emergency fund")
    );
    let names: Vec<&str> = choices
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        &names[..4],
        ["Brazil", "United States", "Portugal", "Spain"]
    );
    let rest = &names[super::FEATURED.len()..];
    let mut sorted = rest.to_vec();
    sorted.sort_by_key(|n| crate::import::text::fold(n));
    assert_eq!(rest, sorted.as_slice());
    assert!(names.contains(&"Germany") && names.contains(&"South Africa"));
    // pt-BR continua igual.
    let st = api.call("country.get", Value::Null).unwrap();
    assert_eq!(st["info"]["inflation_label"], "IHPC");
    assert_eq!(st["info"]["payroll"]["payslips"], "recibos de vencimento");
}

#[test]
fn spain_is_adapted() {
    let es = crate::categories::seeds_for("ES");
    let all: Vec<&str> = es
        .iter()
        .flat_map(|(_, g)| {
            g.iter()
                .flat_map(|(n, _, kids)| std::iter::once(*n).chain(kids.iter().copied()))
        })
        .collect();
    for n in [
        "Pagas extra",
        "IBI",
        "IVTM e ITV",
        "IRPF",
        "Plan de pensiones",
    ] {
        assert!(all.contains(&n), "{n}");
    }
    for n in [
        "Saque do FGTS",
        "13º salário",
        "Férias",
        "JCP",
        "IOF",
        "IPTU",
    ] {
        assert!(!all.contains(&n), "{n}");
    }
    let (_d, api) = api();
    create(&api, Some("ES"));
    let st = api.call("country.get", Value::Null).unwrap();
    assert_eq!(st["info"]["support"], "adapted");
    assert_eq!(st["info"]["currency"], "EUR");
    assert_eq!(st["info"]["features"]["payroll"], true);
    assert_eq!(st["info"]["features"]["fgts"], false);
    assert_eq!(st["info"]["payroll"]["contribution"], "Seguridad Social");
    assert_eq!(st["info"]["tax_region"], "ES");
    let rules = names(&api, "rules.list");
    assert!(
        rules
            .iter()
            .any(|r| r.starts_with("Supermercado (Mercadona"))
    );
    assert!(rules.iter().any(|r| r.starts_with("Bizum")));
}
