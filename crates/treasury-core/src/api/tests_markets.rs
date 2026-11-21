//! Bolsas do mundo, ativos de outras bolsas, cotações intradiárias e busca de tickers pela API.

use serde_json::{Value, json};

use super::*;

fn api() -> (tempfile::TempDir, Api) {
    let dir = tempfile::tempdir().unwrap();
    let paths = TreasuryPaths::with_root(dir.path().join(".treasury")).unwrap();
    let api = Api::new(paths).unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "Família", "members": ["Ana"]}),
    )
    .unwrap();
    (dir, api)
}

#[test]
fn exchanges_catalog_and_user_exchanges_with_undo() {
    let (_d, api) = api();
    let all = api.call("market.exchanges", Value::Null).unwrap();
    let all = all.as_array().unwrap();
    assert!(all.len() >= 40);
    assert_eq!(all[0]["code"], "B3");
    let lse = all.iter().find(|e| e["code"] == "LSE").unwrap();
    assert_eq!(lse["yahoo_suffix"], ".L");
    assert_eq!(lse["timezone"], "Europe/London");
    assert_eq!(lse["fx_automatic"], true);

    let saved = api
        .call(
            "market.exchange_save",
            json!({"code": "pse", "name": "Philippine Stock Exchange", "country": "PH", "currency": "PHP",
                   "yahoo_suffix": ".PS", "timezone": "Asia/Manila", "open_time": "09:30", "close_time": "15:00"}),
        )
        .unwrap();
    assert_eq!(saved["code"], "PSE");
    assert_eq!(saved["builtin"], false);
    let last = api.call("changes.list", json!({"limit": 1})).unwrap();
    assert_eq!(last[0]["reason"], "Bolsa salva: PSE");
    api.call("changes.undo", json!({"id": last[0]["id"]}))
        .unwrap();
    let n = api.call("market.exchanges", Value::Null).unwrap();
    assert!(n.as_array().unwrap().iter().all(|e| e["code"] != "PSE"));
    let err = api
        .call("market.exchange_delete", json!({"code": "B3"}))
        .unwrap_err();
    assert_eq!(err.code(), "conflict");
}

#[test]
fn assets_get_exchange_currency_country_and_yahoo_symbol() {
    let (_d, api) = api();
    let vod = api
        .call(
            "assets.create",
            json!({"ticker": "vod", "name": "Vodafone", "class": "stock_intl", "market": "lse"}),
        )
        .unwrap();
    assert_eq!(vod["market"], "LSE");
    assert_eq!(vod["currency"], "GBP");
    assert_eq!(vod["country"], "GB");
    assert_eq!(vod["quote_symbol"], "VOD.L");
    assert_eq!(vod["exchange"]["timezone"], "Europe/London");
    assert_eq!(vod["class_label"], "Ações em outras bolsas");
    let toyota = api
        .call(
            "assets.create",
            json!({"ticker": "7203", "name": "Toyota", "class": "stock_intl", "market": "JPX"}),
        )
        .unwrap();
    assert_eq!(
        (toyota["market"].as_str(), toyota["currency"].as_str()),
        (Some("TSE"), Some("JPY"))
    );
    assert_eq!(toyota["quote_symbol"], "7203.T");
    let etf = api
        .call(
            "assets.create",
            json!({"ticker": "CSPX", "name": "iShares Core S&P 500", "class": "etf_intl", "market": "LSE", "currency": "USD"}),
        )
        .unwrap();
    assert_eq!(etf["currency"], "USD", "moeda informada vence a da bolsa");
    // Sem bolsa (ou com bolsa fora do catálogo), ações de outras bolsas são recusadas.
    let e = api
        .call(
            "assets.create",
            json!({"ticker": "X", "name": "X", "class": "stock_intl"}),
        )
        .unwrap_err();
    assert_eq!(e.code(), "invalid");
    let e = api
        .call(
            "assets.create",
            json!({"ticker": "X", "name": "X", "class": "etf_intl", "market": "MARTE"}),
        )
        .unwrap_err();
    assert!(e.to_string().contains("catálogo"), "{e}");
    // Classes do Brasil e dos EUA como sempre; nome da bolsa vira o código.
    let brk = api
        .call(
            "assets.create",
            json!({"ticker": "BRK.B", "name": "Berkshire", "class": "stock_us", "market": "NYSE"}),
        )
        .unwrap();
    assert_eq!(
        (brk["quote_symbol"].as_str(), brk["currency"].as_str()),
        (Some("BRK-B"), Some("USD"))
    );
    let voo = api
        .call(
            "assets.create",
            json!({"ticker": "VOO", "name": "Vanguard S&P 500", "class": "etf_us", "market": "NYSE Arca"}),
        )
        .unwrap();
    assert_eq!(voo["market"], "ARCA");
    let petr = api
        .call(
            "assets.create",
            json!({"ticker": "PETR4", "name": "Petrobras", "class": "stock_br"}),
        )
        .unwrap();
    assert_eq!(
        (petr["market"].as_str(), petr["quote_symbol"].as_str()),
        (Some("B3"), Some("PETR4.SA"))
    );
    assert_eq!(petr["currency"], "BRL");
    // Trocar a classe para "outras bolsas" sem bolsa do catálogo é recusado.
    let e = api
        .call(
            "assets.update",
            json!({"id": petr["id"], "class": "stock_intl", "market": null}),
        )
        .unwrap_err();
    assert_eq!(e.code(), "invalid");
}

#[test]
fn quotes_intraday_and_search_commands() {
    let (_d, api) = api();
    let a = api
        .call(
            "assets.create",
            json!({"ticker": "PETR4", "name": "Petrobras PN", "class": "stock_br"}),
        )
        .unwrap();
    let id = a["id"].as_str().unwrap();
    assert!(
        api.call("market.quote", json!({"asset_id": id}))
            .unwrap()
            .is_null()
    );
    let s = api
        .call("market.intraday", json!({"asset_id": id, "range": "5d"}))
        .unwrap();
    assert_eq!(s["points"], json!([]));
    assert_eq!(s["exchange"], "B3");
    assert_eq!(s["timezone"], "America/Sao_Paulo");
    assert_eq!(
        api.call("market.intraday", json!({"asset_id": id, "range": "1y"}))
            .unwrap_err()
            .code(),
        "invalid"
    );
    assert_eq!(api.call("market.quotes", json!({})).unwrap(), json!([]));
    // Atualização sem ativos acompanhados não usa a rede.
    let r = api.call("market.refresh_quotes", json!({})).unwrap();
    assert_eq!(r["refreshed"], json!([]));
    // Busca só local (sem rede): ativo cadastrado com o símbolo do Yahoo.
    let r = api
        .call("market.search", json!({"query": "petr", "online": false}))
        .unwrap();
    assert_eq!(r["online"], false);
    assert_eq!(r["results"][0]["asset_id"], id);
    assert_eq!(r["results"][0]["symbol"], "PETR4.SA");
    assert_eq!(r["results"][0]["type_label"], "Ação");
    let empty = api.call("market.search", json!({"query": " "})).unwrap();
    assert_eq!(empty["results"], json!([]));

    // MCP somente leitura: cotações sim; rede (busca, atualização) e bolsas, não.
    let mcp = Actor::Mcp {
        client: "cli-teste".into(),
        read_only: true,
    };
    assert!(api.call_as(&mcp, "market.quotes", json!({})).is_ok());
    assert!(api.call_as(&mcp, "market.exchanges", Value::Null).is_ok());
    for cmd in [
        "market.search",
        "market.refresh_quotes",
        "market.exchange_save",
    ] {
        assert!(
            matches!(api.call_as(&mcp, cmd, json!({})), Err(Error::Forbidden(_))),
            "{cmd}"
        );
    }
}

/// Mercado fora do catálogo na edição: erro com a lista, e um cliente MCP não deixa proposta
/// na fila (feedback 01a184bb-5c3c).
#[test]
fn update_rejects_unknown_market_without_a_proposal() {
    let (_d, api) = api();
    let a = api
        .call(
            "assets.create",
            json!({"ticker": "BBAS3", "name": "Banco do Brasil ON", "class": "stock_br"}),
        )
        .unwrap();
    let e = api
        .call("assets.update", json!({"id": a["id"], "market": "XYZW"}))
        .unwrap_err();
    assert_eq!(e.code(), "invalid");
    assert!(e.to_string().contains("B3"), "{e}");
    // Apelido do catálogo e mercados do próprio app continuam valendo.
    let u = api
        .call("assets.update", json!({"id": a["id"], "market": "bovespa"}))
        .unwrap();
    assert_eq!(u["market"], "B3");
    let mcp = Actor::Mcp {
        client: "sonda".into(),
        read_only: false,
    };
    let e = api
        .call_as(
            &mcp,
            "assets.update",
            json!({"id": a["id"], "market": "SONDA"}),
        )
        .unwrap_err();
    assert_eq!(e.code(), "invalid");
    assert_eq!(api.call("ai.pending_count", json!({})).unwrap(), 0);
}
