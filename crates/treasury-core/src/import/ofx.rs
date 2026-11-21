//! Leitor de OFX tolerante: 1.x (SGML, folhas sem tag de fechamento) e 2.x (XML).
//!
//! Bancos brasileiros geram OFX "quase válidos": cabeçalho declarando uma codificação e
//! usando outra, `TRNAMT` com vírgula, `STMTTRN` sem fechamento, folhas vazias que engolem
//! as irmãs, tudo numa linha só. O leitor monta uma árvore genérica a partir de tokens e
//! procura os agregados conhecidos (`STMTRS`, `CCSTMTRS`, `STMTTRN`…) onde estiverem.
//!
//! O conteúdo é **dado**: nada do que vem do arquivo é interpretado como instrução.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::error::{Error, Result};

/// Nó da árvore OFX. Folhas têm `text`; agregados têm `children`.
#[derive(Debug, Clone, Default)]
pub struct OfxNode {
    pub name: String,
    pub text: Option<String>,
    pub children: Vec<OfxNode>,
}

impl OfxNode {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Filho direto com o nome.
    pub fn child(&self, name: &str) -> Option<&OfxNode> {
        self.children.iter().find(|c| c.name == name)
    }

    /// Primeiro descendente (busca em profundidade) com o nome.
    pub fn find(&self, name: &str) -> Option<&OfxNode> {
        for c in &self.children {
            if c.name == name {
                return Some(c);
            }
            if let Some(n) = c.find(name) {
                return Some(n);
            }
        }
        None
    }

    /// Todos os descendentes com o nome (inclusive aninhados entre si).
    pub fn find_all<'a>(&'a self, name: &str, out: &mut Vec<&'a OfxNode>) {
        for c in &self.children {
            if c.name == name {
                out.push(c);
            }
            c.find_all(name, out);
        }
    }

    /// Texto de um campo: filho direto; senão descendente, sem entrar em outro `STMTTRN`
    /// (uma folha vazia em SGML pode ter "engolido" as irmãs).
    pub fn field(&self, name: &str) -> Option<&str> {
        if let Some(t) = self
            .children
            .iter()
            .find(|c| c.name == name && c.text.is_some())
            .and_then(|c| c.text.as_deref())
        {
            return Some(t);
        }
        fn walk<'a>(n: &'a OfxNode, name: &str) -> Option<&'a str> {
            for c in &n.children {
                if c.name == "STMTTRN" {
                    continue;
                }
                if c.name == name
                    && let Some(t) = c.text.as_deref()
                {
                    return Some(t);
                }
                if let Some(t) = walk(c, name) {
                    return Some(t);
                }
            }
            None
        }
        walk(self, name)
    }
}

/// Folhas conhecidas: quando aparecem sem texto, são folhas vazias (não agregados).
const KNOWN_LEAVES: &[&str] = &[
    "TRNTYPE",
    "DTPOSTED",
    "DTUSER",
    "DTAVAIL",
    "TRNAMT",
    "FITID",
    "CORRECTFITID",
    "CHECKNUM",
    "REFNUM",
    "SIC",
    "PAYEEID",
    "NAME",
    "MEMO",
    "CURDEF",
    "BANKID",
    "BRANCHID",
    "ACCTID",
    "ACCTTYPE",
    "ACCTKEY",
    "BALAMT",
    "DTASOF",
    "DTSTART",
    "DTEND",
    "CODE",
    "SEVERITY",
    "MESSAGE",
    "DTSERVER",
    "LANGUAGE",
    "ORG",
    "FID",
    "TRNUID",
    "CURRATE",
    "CURSYM",
    "INTU.BID",
    "INTU.USERID",
    "SRVRTID",
    "INV401KSOURCE",
    "EXTDNAME",
    "ADDR1",
    "ADDR2",
    "ADDR3",
    "CITY",
    "STATE",
    "POSTALCODE",
    "COUNTRY",
    "PHONE",
    "DTPROFUP",
    "DTACCTUP",
];

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Open(String),
    Close(String),
    Text(String),
}

fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let after = &rest[i..];
        let end = after.find(';').filter(|&e| e <= 10);
        let Some(end) = end else {
            out.push('&');
            rest = &after[1..];
            continue;
        };
        let ent = &after[1..end];
        let rep = match ent {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some(' '),
            _ if ent.starts_with("#x") || ent.starts_with("#X") => {
                u32::from_str_radix(&ent[2..], 16)
                    .ok()
                    .and_then(char::from_u32)
            }
            _ if ent.starts_with('#') => ent[1..].parse().ok().and_then(char::from_u32),
            _ => None,
        };
        match rep {
            Some(c) => {
                out.push(c);
                rest = &after[end + 1..];
            }
            None => {
                out.push('&');
                rest = &after[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn tokenize(body: &str) -> Vec<Tok> {
    let mut toks = Vec::new();
    let mut rest = body;
    loop {
        let Some(lt) = rest.find('<') else {
            push_text(&mut toks, rest);
            break;
        };
        push_text(&mut toks, &rest[..lt]);
        rest = &rest[lt..];
        if let Some(r) = rest.strip_prefix("<!--") {
            rest = r.find("-->").map(|e| &r[e + 3..]).unwrap_or("");
            continue;
        }
        if let Some(r) = rest.strip_prefix("<![CDATA[") {
            let end = r.find("]]>").unwrap_or(r.len());
            let t = r[..end].trim();
            if !t.is_empty() {
                toks.push(Tok::Text(t.to_string()));
            }
            rest = r.get(end + 3..).unwrap_or("");
            continue;
        }
        if let Some(r) = rest.strip_prefix("<?") {
            rest = r.find("?>").map(|e| &r[e + 2..]).unwrap_or("");
            continue;
        }
        let Some(gt) = rest.find('>') else {
            push_text(&mut toks, &rest[1..]);
            break;
        };
        let inner = rest[1..gt].trim();
        rest = &rest[gt + 1..];
        if inner.starts_with('!') || inner.is_empty() {
            continue;
        }
        let (closing, inner) = match inner.strip_prefix('/') {
            Some(i) => (true, i.trim()),
            None => (false, inner),
        };
        let (self_closing, inner) = match inner.strip_suffix('/') {
            Some(i) => (true, i.trim()),
            None => (false, inner),
        };
        let name: String = inner
            .split(|c: char| c.is_whitespace())
            .next()
            .unwrap_or("")
            .to_ascii_uppercase();
        if name.is_empty() {
            continue;
        }
        if closing {
            toks.push(Tok::Close(name));
        } else if self_closing {
            toks.push(Tok::Open(name.clone()));
            toks.push(Tok::Close(name));
        } else {
            toks.push(Tok::Open(name));
        }
    }
    toks
}

fn push_text(toks: &mut Vec<Tok>, raw: &str) {
    let t = raw.trim();
    if !t.is_empty() {
        toks.push(Tok::Text(decode_entities(t)));
    }
}

/// Monta a árvore a partir dos tokens, tolerando folhas sem fechamento (SGML),
/// fechamentos órfãos e agregados irmãos sem fechamento (`<STMTTRN>…<STMTTRN>`).
fn build_tree(toks: Vec<Tok>) -> OfxNode {
    let mut stack: Vec<OfxNode> = vec![OfxNode::new("#ROOT")];
    let mut it = toks.into_iter().peekable();
    fn pop_into_parent(stack: &mut Vec<OfxNode>) {
        if stack.len() > 1 {
            let n = stack.pop().expect("pilha não vazia");
            stack.last_mut().expect("raiz").children.push(n);
        }
    }
    while let Some(tok) = it.next() {
        match tok {
            Tok::Open(name) => {
                let text = match it.peek() {
                    Some(Tok::Text(_)) => match it.next() {
                        Some(Tok::Text(t)) => Some(t),
                        _ => None,
                    },
                    _ => None,
                };
                let closes_now = matches!(it.peek(), Some(Tok::Close(c)) if *c == name);
                if text.is_some() || closes_now || KNOWN_LEAVES.contains(&name.as_str()) {
                    if closes_now {
                        it.next();
                    }
                    let mut leaf = OfxNode::new(&name);
                    leaf.text = Some(text.unwrap_or_default());
                    stack.last_mut().expect("raiz").children.push(leaf);
                } else {
                    // Agregado irmão sem fechamento do anterior (ex.: dois STMTTRN seguidos).
                    if stack.len() > 1 && stack.last().map(|n| n.name.as_str()) == Some(&name) {
                        pop_into_parent(&mut stack);
                    }
                    stack.push(OfxNode::new(&name));
                }
            }
            Tok::Close(name) => {
                if stack.iter().skip(1).any(|n| n.name == name) {
                    while let Some(top) = stack.last() {
                        let done = top.name == name;
                        pop_into_parent(&mut stack);
                        if done {
                            break;
                        }
                    }
                }
            }
            Tok::Text(t) => {
                // Texto solto dentro de um agregado (ex.: `<PAYEE>Fulano` sem filhos).
                let depth = stack.len();
                if let Some(top) = stack.last_mut()
                    && top.text.is_none()
                    && top.children.is_empty()
                    && depth > 1
                {
                    top.text = Some(t);
                }
            }
        }
    }
    while stack.len() > 1 {
        pop_into_parent(&mut stack);
    }
    stack.pop().expect("raiz")
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct OfxTxn {
    pub trn_type: Option<String>,
    pub date: Option<NaiveDate>,
    pub date_raw: Option<String>,
    pub amount_raw: Option<String>,
    pub fitid: Option<String>,
    pub check_num: Option<String>,
    pub ref_num: Option<String>,
    pub memo: Option<String>,
    pub name: Option<String>,
    pub payee: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct OfxStatement {
    /// `CCSTMTRS` (fatura/extrato de cartão).
    pub credit_card: bool,
    pub currency: Option<String>,
    pub bank_id: Option<String>,
    pub branch_id: Option<String>,
    pub account_id: Option<String>,
    pub account_type: Option<String>,
    pub ledger_balance_raw: Option<String>,
    pub balance_date: Option<NaiveDate>,
    pub date_start: Option<NaiveDate>,
    pub date_end: Option<NaiveDate>,
    pub txns: Vec<OfxTxn>,
}

#[derive(Debug, Clone, Default)]
pub struct OfxDocument {
    /// Cabeçalho (SGML `CHAVE:VALOR` ou atributos do `<?OFX ...?>`), chaves em maiúsculas.
    pub header: BTreeMap<String, String>,
    pub statements: Vec<OfxStatement>,
}

impl OfxDocument {
    pub fn version(&self) -> Option<&str> {
        self.header.get("VERSION").map(String::as_str)
    }
}

fn nonempty(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Atributos `CHAVE="valor"` de uma instrução de processamento XML.
fn pi_attributes(seg: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = seg;
    while let Some(eq) = rest.find('=') {
        let key_start = rest[..eq]
            .rfind(|c: char| c.is_whitespace() || c == '?' || c == '<')
            .map(|i| i + 1)
            .unwrap_or(0);
        let key = rest[key_start..eq].trim().to_ascii_uppercase();
        let after = rest[eq + 1..].trim_start();
        let Some(q) = after.chars().next().filter(|c| *c == '"' || *c == '\'') else {
            rest = &rest[eq + 1..];
            continue;
        };
        let after = &after[1..];
        let Some(end) = after.find(q) else { break };
        if !key.is_empty() {
            out.push((key, after[..end].to_string()));
        }
        rest = &after[end + 1..];
    }
    out
}

/// Lê `CHAVE:VALOR` do cabeçalho SGML e os atributos de `<?OFX ...?>`; a declaração
/// `<?xml ...?>` entra com prefixo `XML.` (ex.: `XML.ENCODING`).
fn parse_header(head: &str) -> BTreeMap<String, String> {
    let mut h = BTreeMap::new();
    for line in head.lines() {
        let line = line.trim();
        if line.starts_with('<') {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            let k = k.trim();
            if !k.is_empty() && k.chars().all(|c| c.is_ascii_alphanumeric()) {
                h.insert(k.to_ascii_uppercase(), v.trim().to_string());
            }
        }
    }
    for (prefix, pi) in [("XML.", "<?xml"), ("", "<?OFX")] {
        if let Some(i) = find_ci(head, pi) {
            let seg = &head[i + pi.len()..];
            let seg = &seg[..seg.find("?>").unwrap_or(seg.len())];
            for (k, v) in pi_attributes(seg) {
                h.insert(format!("{prefix}{k}"), v);
            }
        }
    }
    h
}

/// Codificação declarada no cabeçalho (lida como ASCII, antes de decodificar).
pub fn declared_charset(bytes: &[u8]) -> Option<String> {
    let head = &bytes[..bytes.len().min(2048)];
    let head = String::from_utf8_lossy(head);
    let end = find_ci(&head, "<OFX>").unwrap_or(head.len());
    let h = parse_header(&head[..end]);
    // OFX 2.x: <?xml encoding="..."?>; OFX 1.x: ENCODING:UTF-8 ou CHARSET:1252.
    for key in ["XML.ENCODING", "ENCODING"] {
        if let Some(e) = h.get(key) {
            let e = e.to_ascii_uppercase();
            if e.contains("UTF") {
                return Some("utf-8".into());
            }
            if key == "XML.ENCODING" {
                return Some(e.to_ascii_lowercase());
            }
        }
    }
    let charset = h.get("CHARSET")?.trim().to_ascii_uppercase();
    Some(match charset.as_str() {
        "1252" | "WINDOWS-1252" | "CP1252" => "windows-1252".into(),
        "ISO-8859-1" | "8859-1" | "LATIN1" | "ISO8859-1" => "iso-8859-1".into(),
        "UTF-8" | "UTF8" => "utf-8".into(),
        "NONE" => return None,
        other => other.to_ascii_lowercase(),
    })
}

fn find_ci(hay: &str, needle: &str) -> Option<usize> {
    let h = hay.to_ascii_uppercase();
    let n = needle.to_ascii_uppercase();
    h.find(&n)
}

/// O conteúdo parece OFX?
pub fn looks_like_ofx(bytes: &[u8]) -> bool {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(4096)]).to_ascii_uppercase();
    let t = head.trim_start_matches(['\u{feff}', ' ', '\r', '\n', '\t']);
    t.starts_with("OFXHEADER")
        || head.contains("<OFX>")
        || head.contains("<?OFX")
        || head.contains("<STMTTRN>")
}

/// Lê o documento OFX (texto já decodificado).
pub fn parse(text: &str) -> Result<OfxDocument> {
    let start = find_ci(text, "<OFX>")
        .or_else(|| find_ci(text, "<OFX "))
        .or_else(|| find_ci(text, "<STMTTRN"));
    let Some(start) = start else {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-importacao-ofx-erro-sem-bloco",
        )));
    };
    let header = parse_header(&text[..start]);
    let root = build_tree(tokenize(&text[start..]));

    let mut stmt_nodes: Vec<&OfxNode> = Vec::new();
    collect_statements(&root, &mut stmt_nodes);
    let mut statements: Vec<OfxStatement> = stmt_nodes.iter().map(|n| statement(n)).collect();
    if statements.iter().all(|s| s.txns.is_empty()) {
        // Arquivo sem STMTRS reconhecível: junta todos os STMTTRN que houver.
        let mut trns = Vec::new();
        root.find_all("STMTTRN", &mut trns);
        if !trns.is_empty() {
            let mut s = statements.into_iter().next().unwrap_or_default();
            s.txns = trns.into_iter().map(txn).collect();
            statements = vec![s];
        }
    }
    if statements.is_empty() {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-importacao-ofx-erro-sem-extrato",
        )));
    }
    Ok(OfxDocument { header, statements })
}

fn collect_statements<'a>(n: &'a OfxNode, out: &mut Vec<&'a OfxNode>) {
    for c in &n.children {
        if c.name == "STMTRS" || c.name == "CCSTMTRS" {
            out.push(c);
        } else {
            collect_statements(c, out);
        }
    }
}

fn statement(n: &OfxNode) -> OfxStatement {
    let credit_card = n.name == "CCSTMTRS";
    let acct = n.find("BANKACCTFROM").or_else(|| n.find("CCACCTFROM"));
    let tranlist = n.find("BANKTRANLIST");
    let ledger = n.find("LEDGERBAL");
    let mut trns = Vec::new();
    n.find_all("STMTTRN", &mut trns);
    OfxStatement {
        credit_card,
        currency: nonempty(n.field("CURDEF")).map(|c| c.to_ascii_uppercase()),
        bank_id: acct.and_then(|a| nonempty(a.field("BANKID"))),
        branch_id: acct.and_then(|a| nonempty(a.field("BRANCHID"))),
        account_id: acct.and_then(|a| nonempty(a.field("ACCTID"))),
        account_type: acct.and_then(|a| nonempty(a.field("ACCTTYPE"))),
        ledger_balance_raw: ledger.and_then(|l| nonempty(l.field("BALAMT"))),
        balance_date: ledger.and_then(|l| l.field("DTASOF")).and_then(parse_date),
        date_start: tranlist
            .and_then(|t| t.field("DTSTART"))
            .and_then(parse_date),
        date_end: tranlist.and_then(|t| t.field("DTEND")).and_then(parse_date),
        txns: trns.into_iter().map(txn).collect(),
    }
}

fn txn(n: &OfxNode) -> OfxTxn {
    let date_raw = nonempty(n.field("DTPOSTED")).or_else(|| nonempty(n.field("DTUSER")));
    // PAYEE pode ser folha (quirk) ou agregado com NAME.
    let payee = n.child("PAYEE").and_then(|p| {
        nonempty(p.text.as_deref())
            .or_else(|| nonempty(p.child("NAME").and_then(|c| c.text.as_deref())))
    });
    let name = n
        .children
        .iter()
        .find(|c| c.name == "NAME")
        .and_then(|c| nonempty(c.text.as_deref()))
        .or_else(|| nonempty(n.field("NAME")).filter(|v| Some(v) != payee.as_ref()));
    OfxTxn {
        trn_type: nonempty(n.field("TRNTYPE")).map(|t| t.to_ascii_uppercase()),
        date: date_raw.as_deref().and_then(parse_date),
        date_raw,
        amount_raw: nonempty(n.field("TRNAMT")),
        fitid: nonempty(n.field("FITID")),
        check_num: nonempty(n.field("CHECKNUM")),
        ref_num: nonempty(n.field("REFNUM")),
        memo: nonempty(n.field("MEMO")),
        name,
        payee,
    }
}

/// Data OFX: `AAAAMMDD[HHMMSS[.XXX]][[±H[:TZ]]]`. A data é a do próprio extrato (no fuso
/// declarado), sem conversão para UTC: `20261014230000[-3:BRT]` é 14/10/2026.
/// Tolerante a `AAAA-MM-DD` e `DD/MM/AAAA`.
pub fn parse_date(raw: &str) -> Option<NaiveDate> {
    let s = raw.trim();
    let s = s.split('[').next().unwrap_or(s).trim();
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 8 {
        let y: i32 = digits[0..4].parse().ok()?;
        let m: u32 = digits[4..6].parse().ok()?;
        let d: u32 = digits[6..8].parse().ok()?;
        return NaiveDate::from_ymd_opt(y, m, d);
    }
    let head: String = s.chars().take(10).collect();
    NaiveDate::parse_from_str(&head, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(&head, "%d/%m/%Y"))
        .ok()
}

/// Valor OFX: ponto ou vírgula decimal, sinal opcional, sem separador de milhar pela
/// especificação (mas aceita `1.234,56`). Zeros além das casas da moeda são descartados;
/// casas significativas a mais são erro (nunca arredondamos dinheiro em silêncio).
pub fn parse_amount(raw: &str, currency: &str) -> Result<i64> {
    let units = crate::money::minor_units(currency) as usize;
    let s: String = raw
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{a0}')
        .collect();
    let seps: Vec<(usize, char)> = s
        .char_indices()
        .filter(|(_, c)| *c == '.' || *c == ',')
        .collect();
    let canonical = match seps.as_slice() {
        [] => s.clone(),
        many if many.len() > 1 && many.iter().all(|(_, c)| *c == many[0].1) => {
            // Só separadores de milhar (ex.: 1.234.567).
            s.replace(many[0].1, "")
        }
        _ => {
            let (idx, _) = *seps.last().expect("não vazio");
            let int: String = s[..idx]
                .chars()
                .filter(|c| *c != '.' && *c != ',')
                .collect();
            let mut frac = s[idx + 1..].to_string();
            let suffix_neg = frac.ends_with('-');
            if suffix_neg {
                frac.pop();
            }
            while frac.len() > units && frac.ends_with('0') {
                frac.pop();
            }
            let mut c = if frac.is_empty() {
                int
            } else {
                format!("{int},{frac}")
            };
            if suffix_neg {
                c.push('-');
            }
            c
        }
    };
    crate::money::parse_amount(&canonical, currency)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_with_timezone_and_variants() {
        let d = |s| parse_date(s).map(|d| d.to_string());
        assert_eq!(d("20261014120000[-3:BRT]").as_deref(), Some("2026-10-14"));
        assert_eq!(d("20261014230000[-03:EST]").as_deref(), Some("2026-10-14"));
        assert_eq!(d("20261014000000.000[-3]").as_deref(), Some("2026-10-14"));
        assert_eq!(d("20261014").as_deref(), Some("2026-10-14"));
        assert_eq!(d(" 20261014100000 ").as_deref(), Some("2026-10-14"));
        assert_eq!(d("2026-10-14").as_deref(), Some("2026-10-14"));
        assert_eq!(d("14/10/2026").as_deref(), Some("2026-10-14"));
        assert_eq!(d("20261341"), None);
        assert_eq!(d("lixo"), None);
    }

    #[test]
    fn amounts_with_dot_or_comma() {
        assert_eq!(parse_amount("-123.45", "BRL").unwrap(), -12345);
        assert_eq!(parse_amount("-123,45", "BRL").unwrap(), -12345);
        assert_eq!(parse_amount("+50.5", "BRL").unwrap(), 5050);
        assert_eq!(parse_amount("1500", "BRL").unwrap(), 150000);
        assert_eq!(parse_amount("-12.3400", "BRL").unwrap(), -1234);
        assert_eq!(parse_amount("-1.234,56", "BRL").unwrap(), -123456);
        assert_eq!(parse_amount("1,234.56", "BRL").unwrap(), 123456);
        assert_eq!(parse_amount("1.234.567", "BRL").unwrap(), 123456700);
        assert_eq!(parse_amount("50,00-", "BRL").unwrap(), -5000);
        assert!(
            parse_amount("-1.234", "BRL").is_err(),
            "3 casas significativas"
        );
        assert!(parse_amount("abc", "BRL").is_err());
    }

    #[test]
    fn tree_tolerates_sgml_quirks() {
        let body = "<OFX><BANKTRANLIST><STMTTRN><TRNTYPE>DEBIT<MEMO><TRNAMT>-1.00<FITID>a\
                    <STMTTRN><TRNTYPE>CREDIT<TRNAMT>2,00<FITID>b<PAYEE>Fulano &amp; Cia</BANKTRANLIST></OFX>";
        let doc = parse(body).unwrap();
        let t = &doc.statements[0].txns;
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].fitid.as_deref(), Some("a"));
        assert_eq!(t[0].memo, None, "MEMO vazio não engole as irmãs");
        assert_eq!(t[1].amount_raw.as_deref(), Some("2,00"));
        assert_eq!(t[1].payee.as_deref(), Some("Fulano & Cia"));
    }

    #[test]
    fn header_and_charset() {
        let raw = b"OFXHEADER:100\r\nDATA:OFXSGML\r\nVERSION:102\r\nENCODING:USASCII\r\nCHARSET:1252\r\n\r\n<OFX></OFX>";
        assert_eq!(declared_charset(raw).as_deref(), Some("windows-1252"));
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?><?OFX OFXHEADER="200" VERSION="211"?><OFX></OFX>"#;
        assert_eq!(declared_charset(xml).as_deref(), Some("utf-8"));
        let doc_head = parse_header(
            r#"<?xml version="1.0" encoding="UTF-8"?><?OFX OFXHEADER="200" VERSION="211"?>"#,
        );
        assert_eq!(doc_head.get("VERSION").map(String::as_str), Some("211"));
        assert!(looks_like_ofx(raw));
        assert!(looks_like_ofx(xml));
        assert!(!looks_like_ofx(b"Data,Valor\n01/10/2026,1.00"));
    }
}
