//! Notas fiscais de fora do Brasil pelo QR (docs/receipts.md › Notas de outros países).
//! Portugal: o código QR das faturas (Portaria 195/2020, especificações da AT) é um texto com
//! campos `X:valor` separados por `*` (A: NIF de quem emitiu, D: tipo, F: data AAAAMMDD, G:
//! número, H: ATCUD, N: total de IVA, O: total com impostos). Espanha: o QR do VeriFactu (Orden
//! HAC/1177/2024) é um link da AEAT com `nif`, `numserie`, `fecha` (DD-MM-AAAA) e `importe`; o
//! app só lê os campos do link, nunca abre o endereço. Nos dois, como na Argentina, o QR não
//! traz os itens: a data, a loja e o total servem de gabarito para a foto e evitam a mesma
//! nota entrar duas vezes. A
//! Argentina (AFIP, RG 4892/2020) põe no QR um link `https://www.afip.gob.ar/fe/qr/?p=` com um
//! JSON em base64: data, CUIT de quem emitiu, ponto de venda, tipo e número do comprovante,
//! valor, moeda e cotação. O QR não traz os itens nem o nome da loja: a compra abre preenchida
//! com o que há, e os itens vêm da foto do cupom ou à mão. Vale em qualquer país da família
//! (compra feita numa viagem).

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};

/// Nota de fora lida pelo QR.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ForeignReceipt {
    /// País (ISO), ex.: `AR`.
    pub country: String,
    /// Órgão que valida a nota, ex.: `AFIP`.
    pub authority: String,
    /// `YYYY-MM-DD`.
    pub date: String,
    /// Identificação fiscal de quem emitiu (CUIT formatado, `30-12345678-9`).
    pub issuer_tax_id: String,
    /// Tipo do comprovante como sai na nota, ex.: `Factura B`.
    pub document: String,
    /// Ponto de venda e número, ex.: `0003-00012345`.
    pub number: String,
    /// Nota de crédito (devolve dinheiro).
    pub credit: bool,
    /// Valor na moeda da nota, em centavos.
    #[ts(type = "number")]
    pub total: i64,
    /// Moeda da nota (ISO), ex.: `ARS`.
    pub currency: String,
    /// O valor na moeda da família pela cotação guardada no app, quando há.
    pub converted: Option<crate::fx::Converted>,
    /// Código de autorização (CAE ou CAEA; em Portugal, o ATCUD).
    pub authorization: Option<String>,
    /// Imposto total da nota (o IVA do QR de Portugal), em centavos, quando o QR traz.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub tax: Option<i64>,
    /// Como se chama a identificação fiscal de quem emitiu (`CUIT`, `NIF`).
    #[serde(default)]
    pub tax_id_label: String,
    /// O link do QR, para conferir no site do órgão.
    pub url: String,
}

/// Um link de QR de nota fiscal de outro país que o app sabe ler?
pub fn is_foreign(url: &str) -> bool {
    is_at_code(url) || host_of(url).is_some_and(|h| afip_host(&h) || aeat_host(&h))
}

/// O texto do QR de uma fatura de Portugal ("A:123456789*B:…*…").
fn is_at_code(text: &str) -> bool {
    let t = text.trim();
    t.starts_with("A:") && t.contains("*F:") && t.contains("*O:")
}

fn aeat_host(h: &str) -> bool {
    h == "agenciatributaria.gob.es"
        || h.ends_with(".agenciatributaria.gob.es")
        || h == "aeat.es"
        || h.ends_with(".aeat.es")
}

fn host_of(url: &str) -> Option<String> {
    let u = url.trim();
    let rest = u
        .strip_prefix("https://")
        .or_else(|| u.strip_prefix("http://"))?;
    Some(
        rest.split(['/', '?', '#'])
            .next()
            .unwrap_or("")
            .to_ascii_lowercase(),
    )
}

fn afip_host(h: &str) -> bool {
    h == "afip.gob.ar"
        || h.ends_with(".afip.gob.ar")
        || h == "arca.gob.ar"
        || h.ends_with(".arca.gob.ar")
}

/// Valor do parâmetro `p` do link, sem o `%XX`.
fn param_p(url: &str) -> Option<String> {
    param(url, "p")
}

/// Valor de um parâmetro do link, sem o `%XX`.
fn param(url: &str, key: &str) -> Option<String> {
    let query = url.split_once('?')?.1.split('#').next().unwrap_or("");
    let raw = query
        .split('&')
        .find_map(|kv| kv.strip_prefix(key)?.strip_prefix('='))?
        .to_string();
    let b = raw.as_bytes();
    let hex = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match (
            b[i],
            b.get(i + 1).copied().and_then(hex),
            b.get(i + 2).copied().and_then(hex),
        ) {
            (b'%', Some(h), Some(l)) => {
                out.push(h << 4 | l);
                i += 3;
            }
            (c, _, _) => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

/// Número que vem como número ou como texto.
fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().replace(',', ".").parse().ok(),
        _ => None,
    }
}

fn int(v: &Value) -> Option<u64> {
    num(v).filter(|n| *n >= 0.0).map(|n| n as u64)
}

/// Tipo do comprovante (tabela de comprovantes da AFIP).
fn document_name(code: u64) -> (String, bool) {
    let (name, credit) = match code {
        1 => ("Factura A", false),
        2 => ("Nota de Débito A", false),
        3 => ("Nota de Crédito A", true),
        6 => ("Factura B", false),
        7 => ("Nota de Débito B", false),
        8 => ("Nota de Crédito B", true),
        11 => ("Factura C", false),
        12 => ("Nota de Débito C", false),
        13 => ("Nota de Crédito C", true),
        19 => ("Factura E", false),
        51 => ("Factura M", false),
        81 => ("Tique Factura A", false),
        82 => ("Tique Factura B", false),
        83 => ("Tique", false),
        111 => ("Tique Factura C", false),
        201 => ("Factura de Crédito Electrónica MiPyMEs A", false),
        206 => ("Factura de Crédito Electrónica MiPyMEs B", false),
        211 => ("Factura de Crédito Electrónica MiPyMEs C", false),
        _ => return (format!("Comprobante {code}"), false),
    };
    (name.to_string(), credit)
}

/// Moeda da AFIP → ISO.
fn currency(code: &str) -> String {
    match code.trim().to_ascii_uppercase().as_str() {
        "PES" | "" => "ARS".into(),
        "DOL" => "USD".into(),
        "060" => "EUR".into(),
        "012" => "BRL".into(),
        "021" => "GBP".into(),
        other => other.to_string(),
    }
}

fn cuit(n: u64) -> String {
    let s = format!("{n:011}");
    format!("{}-{}-{}", &s[..2], &s[2..10], &s[10..])
}

fn bad() -> Error {
    Error::invalid(t("nucleo-importacao-cupom-erro-afip"))
}

fn bad_qr() -> Error {
    Error::invalid(t("nucleo-importacao-cupom-erro-qr"))
}

/// O valor na moeda da família pela cotação guardada no app, quando a moeda é outra.
fn in_base(
    conn: &Connection,
    total: i64,
    cur: &str,
    date: &str,
) -> Result<Option<crate::fx::Converted>> {
    let base = crate::household::base_currency(conn).unwrap_or_else(|_| "BRL".into());
    if cur == base {
        Ok(None)
    } else {
        crate::fx::convert(conn, total, cur, &base, date)
    }
}

/// "12.34" → 1234 (o QR de Portugal e o da Espanha usam o ponto decimal).
fn cents(v: &str) -> Option<i64> {
    let n: f64 = v.trim().replace(',', ".").parse().ok()?;
    n.is_finite().then(|| (n * 100.0).round() as i64)
}

/// Tipo do documento do QR de Portugal (tabela do SAF-T).
fn at_document(code: &str) -> (String, bool) {
    let (name, credit) = match code {
        "FT" => ("Fatura", false),
        "FS" => ("Fatura simplificada", false),
        "FR" => ("Fatura-recibo", false),
        "ND" => ("Nota de débito", false),
        "NC" => ("Nota de crédito", true),
        other => return (other.to_string(), false),
    };
    (name.to_string(), credit)
}

/// Lê o texto do QR de uma fatura de Portugal.
fn read_at(conn: &Connection, text: &str) -> Result<ForeignReceipt> {
    let field = |code: &str| {
        text.trim()
            .split('*')
            .find_map(|f| f.strip_prefix(code)?.strip_prefix(':'))
            .map(str::trim)
            .filter(|v| !v.is_empty())
    };
    let nif = field("A")
        .filter(|n| n.len() == 9 && n.bytes().all(|b| b.is_ascii_digit()))
        .ok_or_else(bad_qr)?;
    let f = field("F").filter(|d| d.len() == 8).ok_or_else(bad_qr)?;
    let date = crate::time::parse_date(&format!("{}-{}-{}", &f[..4], &f[4..6], &f[6..]))
        .map_err(|_| bad_qr())?;
    let total = field("O").and_then(cents).ok_or_else(bad_qr)?;
    let (document, credit) = at_document(field("D").unwrap_or(""));
    let date_s = crate::time::fmt_date(date);
    Ok(ForeignReceipt {
        country: "PT".into(),
        authority: "AT".into(),
        converted: in_base(conn, total, "EUR", &date_s)?,
        date: date_s,
        issuer_tax_id: nif.to_string(),
        document,
        number: field("G").unwrap_or("").to_string(),
        credit,
        total,
        currency: "EUR".into(),
        // ATCUD "0": série ainda sem código de validação.
        authorization: field("H").filter(|h| *h != "0").map(String::from),
        tax: field("N").and_then(cents),
        tax_id_label: "NIF".into(),
        url: text.trim().to_string(),
    })
}

/// Lê os campos do link do QR do VeriFactu (Espanha). O endereço não é aberto.
fn read_aeat(conn: &Connection, url: &str) -> Result<ForeignReceipt> {
    let nif = param(url, "nif")
        .filter(|n| n.len() == 9)
        .ok_or_else(bad_qr)?;
    let fecha = param(url, "fecha").ok_or_else(bad_qr)?;
    let parts: Vec<&str> = fecha.split('-').collect();
    let date = match parts.as_slice() {
        [d, m, y] if y.len() == 4 => crate::time::parse_date(&format!("{y}-{m}-{d}")).ok(),
        _ => None,
    }
    .ok_or_else(bad_qr)?;
    let total = param(url, "importe")
        .as_deref()
        .and_then(cents)
        .ok_or_else(bad_qr)?;
    let date_s = crate::time::fmt_date(date);
    Ok(ForeignReceipt {
        country: "ES".into(),
        authority: "AEAT".into(),
        converted: in_base(conn, total.abs(), "EUR", &date_s)?,
        date: date_s,
        issuer_tax_id: nif.to_ascii_uppercase(),
        document: "Factura".into(),
        number: param(url, "numserie").unwrap_or_default(),
        // Fatura retificativa com valor negativo devolve dinheiro.
        credit: total < 0,
        total: total.abs(),
        currency: "EUR".into(),
        authorization: None,
        tax: None,
        tax_id_label: "NIF".into(),
        url: url.trim().to_string(),
    })
}

/// Lê o QR de uma nota de fora do Brasil: Portugal (texto), Espanha e Argentina (link).
pub fn read(conn: &Connection, url: &str) -> Result<ForeignReceipt> {
    if is_at_code(url) {
        return read_at(conn, url);
    }
    let host = host_of(url).unwrap_or_default();
    if aeat_host(&host) {
        return read_aeat(conn, url);
    }
    read_afip(conn, url, &host)
}

/// Lê o QR da AFIP. O valor na moeda da família sai pela cotação guardada no app (a da nota,
/// `ctz`, é para pesos), quando há.
fn read_afip(conn: &Connection, url: &str, host: &str) -> Result<ForeignReceipt> {
    if !afip_host(host) {
        return Err(not_supported(host));
    }
    let p = param_p(url).ok_or_else(bad)?;
    let json = crate::transactions::b64::decode(&p).map_err(|_| bad())?;
    let v: Value = serde_json::from_slice(&json).map_err(|_| bad())?;
    let date = v
        .get("fecha")
        .and_then(Value::as_str)
        .and_then(|d| crate::time::parse_date(d.get(..10).unwrap_or(d)).ok())
        .ok_or_else(bad)?;
    let importe = v.get("importe").and_then(num).ok_or_else(bad)?;
    let issuer = v.get("cuit").and_then(int).ok_or_else(bad)?;
    let (document, credit) = document_name(v.get("tipoCmp").and_then(int).unwrap_or(0));
    let pto = v.get("ptoVta").and_then(int).unwrap_or(0);
    let nro = v.get("nroCmp").and_then(int).unwrap_or(0);
    let cur = currency(v.get("moneda").and_then(Value::as_str).unwrap_or("PES"));
    let total = (importe * 100.0).round() as i64;
    let date_s = crate::time::fmt_date(date);
    let base = crate::household::base_currency(conn).unwrap_or_else(|_| "BRL".into());
    let converted = if cur == base {
        None
    } else {
        crate::fx::convert(conn, total, &cur, &base, &date_s)?
    };
    let authorization = v.get("codAut").and_then(|c| match c {
        Value::Number(n) => Some(n.to_string()),
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        _ => None,
    });
    Ok(ForeignReceipt {
        country: "AR".into(),
        authority: "AFIP".into(),
        date: date_s,
        issuer_tax_id: cuit(issuer),
        document,
        number: format!("{pto:04}-{nro:08}"),
        credit,
        total,
        currency: cur,
        converted,
        authorization,
        tax: None,
        tax_id_label: "CUIT".into(),
        url: url.trim().to_string(),
    })
}

/// Link de QR que o app não lê: diz de onde ele é e o que dá para fazer.
pub fn not_supported(host: &str) -> Error {
    Error::invalid(t_args(
        "nucleo-importacao-cupom-erro-sefaz",
        &args(&[("host", host.to_string().into())]),
    ))
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::api::Api;
    use crate::paths::TreasuryPaths;

    fn link(json: &str) -> String {
        format!(
            "https://www.afip.gob.ar/fe/qr/?p={}",
            crate::transactions::b64::encode(json.as_bytes())
        )
    }

    fn api() -> (tempfile::TempDir, Api) {
        let dir = tempfile::tempdir().unwrap();
        let api = Api::new(TreasuryPaths::with_root(dir.path().join(".t")).unwrap()).unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "Casa"}),
        )
        .unwrap();
        (dir, api)
    }

    fn read_qr(api: &Api, url: &str) -> crate::error::Result<Value> {
        api.call("receipts.foreign_qr", json!({ "url": url }))
    }

    #[test]
    fn reads_the_afip_qr_in_a_brazilian_vault() {
        let (_d, api) = api();
        let url = link(
            r#"{"ver":1,"fecha":"2026-10-20","cuit":30712345678,"ptoVta":3,"tipoCmp":6,"nroCmp":12345,"importe":15234.5,"moneda":"PES","ctz":1,"tipoDocRec":99,"nroDocRec":0,"tipoCodAut":"E","codAut":76123456789012}"#,
        );
        assert!(is_foreign(&url));
        let r = read_qr(&api, &url).unwrap();
        assert_eq!(r["country"], "AR");
        assert_eq!(r["date"], "2026-10-20");
        assert_eq!(r["issuer_tax_id"], "30-71234567-8");
        assert_eq!(r["document"], "Factura B");
        assert_eq!(r["number"], "0003-00012345");
        assert_eq!(r["total"], 1_523_450);
        assert_eq!(r["currency"], "ARS");
        assert_eq!(r["credit"], false);
        assert_eq!(r["authorization"], "76123456789012");
        // Without a stored peso exchange rate, the value in reais is left to the person.
        assert!(r["converted"].is_null());
        // With the exchange rate, the value in reais comes out.
        api.call(
            "fx.set",
            json!({"base": "ARS", "quote": "BRL", "date": "2026-10-19", "rate": "0.0055"}),
        )
        .unwrap();
        let r = read_qr(&api, &url).unwrap();
        assert_eq!(r["converted"]["amount"], 8_379, "{r}");
        assert_eq!(r["converted"]["currency"], "BRL");
    }

    #[test]
    fn tolerates_url_encoding_strings_and_dollars() {
        let (_d, api) = api();
        let json = r#"{"ver":1,"fecha":"2026-09-01","cuit":"20123456789","ptoVta":"1","tipoCmp":"8","nroCmp":"7","importe":"99.90","moneda":"DOL","ctz":"980.5"}"#;
        let p = crate::transactions::b64::encode(json.as_bytes()).replace('=', "%3D");
        let r = read_qr(&api, &format!("https://www.afip.gob.ar/fe/qr/?p={p}")).unwrap();
        assert_eq!(r["issuer_tax_id"], "20-12345678-9");
        assert_eq!(r["currency"], "USD");
        assert_eq!(r["total"], 9_990);
        assert_eq!(r["credit"], true, "credit note");
    }

    #[test]
    fn other_links_say_where_they_are_from() {
        let (_d, api) = api();
        assert!(!is_foreign(
            "https://www.sefaz.rs.gov.br/NFCE/NFCE-COM.aspx?p=1"
        ));
        let e = read_qr(&api, "https://www.sii.cl/boleta?x=1").unwrap_err();
        assert!(e.message().contains("www.sii.cl"), "{}", e.message());
        let e = read_qr(&api, "https://www.afip.gob.ar/fe/qr/?p=naoebase64").unwrap_err();
        assert_eq!(e.code(), "invalid");
        // The usual reading, with a link from elsewhere, explains the same.
        let e = api
            .call(
                "receipts.preview",
                json!({"url": "https://www.sii.cl/boleta?x=1"}),
            )
            .unwrap_err();
        assert!(e.message().contains("www.sii.cl"), "{}", e.message());
    }

    /// The QR of Portuguese invoices (text with fields separated by `*`) and the Spanish VeriFactu
    /// one (link with the fields): date, NIF, number and total, without opening any address.
    #[test]
    fn reads_the_portuguese_and_spanish_qr() {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::paths::TreasuryPaths::with_root(dir.path().join(".t")).unwrap();
        let api = Api::new(paths).unwrap();
        api.call(
            "vault.create",
            json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"], "country": "PT"}),
        )
        .unwrap();
        let pt = "A:500000000*B:999999990*C:PT*D:FS*E:N*F:20261114*G:FS 0012/3456*H:JFXYZ123-3456*I1:PT*I3:10.00*I4:0.60*I7:8.13*I8:1.87*N:2.47*O:20.60*Q:abcd*R:9999";
        assert!(is_foreign(pt));
        let r = api.call("receipts.foreign_qr", json!({"url": pt})).unwrap();
        assert_eq!(r["country"], "PT");
        assert_eq!(r["authority"], "AT");
        assert_eq!(r["date"], "2026-11-14");
        assert_eq!(r["issuer_tax_id"], "500000000");
        assert_eq!(r["tax_id_label"], "NIF");
        assert_eq!(r["document"], "Fatura simplificada");
        assert_eq!(r["number"], "FS 0012/3456");
        assert_eq!(r["total"], 2_060);
        assert_eq!(r["tax"], 247);
        assert_eq!(r["currency"], "EUR");
        assert_eq!(r["authorization"], "JFXYZ123-3456");
        assert!(r["converted"].is_null(), "the family already uses the euro");
        // Credit note and truncated QR.
        let nc = pt.replace("D:FS", "D:NC");
        assert_eq!(
            api.call("receipts.foreign_qr", json!({"url": nc})).unwrap()["credit"],
            true
        );
        assert!(
            api.call(
                "receipts.foreign_qr",
                json!({"url": "A:500000000*B:999999990*F:2026*O:1.00"})
            )
            .is_err()
        );

        let es = "https://www2.agenciatributaria.gob.es/wlpl/TIKE-CONT/ValidarQR?nif=B12345678&numserie=A%2F2026%2F0001&fecha=14-11-2026&importe=121.00";
        assert!(is_foreign(es));
        let r = api.call("receipts.foreign_qr", json!({"url": es})).unwrap();
        assert_eq!(r["country"], "ES");
        assert_eq!(r["authority"], "AEAT");
        assert_eq!(r["date"], "2026-11-14");
        assert_eq!(r["issuer_tax_id"], "B12345678");
        assert_eq!(r["number"], "A/2026/0001");
        assert_eq!(r["total"], 12_100);
        assert!(
            api.call(
                "receipts.foreign_qr",
                json!({"url": "https://www2.agenciatributaria.gob.es/wlpl/TIKE-CONT/ValidarQR?nif=B12345678"})
            )
            .is_err()
        );
    }
}
