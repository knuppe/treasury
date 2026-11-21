//! Pix "copia e cola" (BR Code, padrão EMV QRCPS-MPM do Banco Central): o texto de um QR Pix
//! ou colado. Conferido pelo CRC16 do fim do código. O app **não** consulta a URL do Pix
//! dinâmico (a consulta avisa o recebedor e sai do computador): fica com o nome, a cidade e o
//! valor que o próprio código traz (docs/ocr-other-documents.md › 3.3).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PixCode {
    /// O código inteiro (para copiar e pagar no banco).
    pub payload: String,
    /// Chave Pix do recebedor (Pix estático).
    #[ts(optional)]
    pub key: Option<String>,
    /// Endereço do PSP (Pix dinâmico; não é consultado).
    #[ts(optional)]
    pub url: Option<String>,
    /// Valor em centavos, quando fixado no código.
    #[ts(optional, type = "number")]
    pub amount: Option<i64>,
    #[ts(optional)]
    pub name: Option<String>,
    #[ts(optional)]
    pub city: Option<String>,
    /// Identificador da cobrança (`***` quando não há).
    #[ts(optional)]
    pub txid: Option<String>,
    /// Mensagem ao pagador, quando houver.
    #[ts(optional)]
    pub description: Option<String>,
    /// Código de uso único (cobrança).
    pub dynamic: bool,
}

/// Campos TLV (id de 2 dígitos, tamanho de 2 dígitos, valor).
fn fields(s: &str) -> Option<Vec<(&str, &str)>> {
    let mut out = vec![];
    let mut i = 0;
    while i < s.len() {
        let id = s.get(i..i + 2)?;
        let len: usize = s.get(i + 2..i + 4)?.parse().ok()?;
        let v = s.get(i + 4..i + 4 + len)?;
        out.push((id, v));
        i += 4 + len;
    }
    Some(out)
}

fn cents(v: &str) -> Option<i64> {
    let (int, frac) = v.split_once('.').unwrap_or((v, ""));
    if int.is_empty() && frac.is_empty() || frac.len() > 2 {
        return None;
    }
    let int: i64 = if int.is_empty() { 0 } else { int.parse().ok()? };
    let frac: i64 = format!("{frac:0<2}").parse().ok()?;
    Some(int * 100 + frac)
}

/// Lê um BR Code Pix (começa com `000201`, termina no CRC16 de `6304`).
pub fn parse(text: &str) -> Option<PixCode> {
    let p: String = text
        .trim()
        .chars()
        .filter(|c| *c != '\n' && *c != '\r')
        .collect();
    if !p.starts_with("000201") || p.len() < 30 || !p.is_ascii() {
        return None;
    }
    let crc_at = p.len().checked_sub(4)?;
    if p.get(crc_at - 4..crc_at)? != "6304" {
        return None;
    }
    let want = u16::from_str_radix(&p[crc_at..], 16).ok()?;
    if super::br_code::crc16(&p.as_bytes()[..crc_at]) != want {
        return None;
    }
    let top = fields(&p)?;
    let mut code = PixCode {
        payload: p.clone(),
        key: None,
        url: None,
        amount: None,
        name: None,
        city: None,
        txid: None,
        description: None,
        dynamic: false,
    };
    let mut is_pix = false;
    for (id, v) in top {
        match id {
            "01" => code.dynamic = v == "12",
            "26" => {
                let sub = fields(v)?;
                if !sub
                    .iter()
                    .any(|(i, v)| *i == "00" && v.eq_ignore_ascii_case("br.gov.bcb.pix"))
                {
                    continue;
                }
                is_pix = true;
                for (i, v) in sub {
                    let v = Some(v.trim().to_string()).filter(|s| !s.is_empty());
                    match i {
                        "01" => code.key = v,
                        "02" => code.description = v,
                        "25" => code.url = v,
                        _ => {}
                    }
                }
            }
            "54" => code.amount = cents(v).filter(|c| *c > 0),
            "59" => code.name = Some(v.trim().to_string()).filter(|s| !s.is_empty()),
            "60" => code.city = Some(v.trim().to_string()).filter(|s| !s.is_empty()),
            "62" => {
                if let Some(sub) = fields(v) {
                    code.txid = sub
                        .iter()
                        .find(|(i, _)| *i == "05")
                        .map(|(_, v)| v.to_string())
                        .filter(|s| !s.is_empty());
                }
            }
            _ => {}
        }
    }
    if code.url.is_some() {
        code.dynamic = true;
    }
    is_pix.then_some(code)
}

/// Procura o código "copia e cola" impresso no texto do OCR (às vezes em várias linhas).
pub fn find_in_text(rows: &[String]) -> Option<PixCode> {
    let trimmed: Vec<&str> = rows.iter().map(|r| r.trim()).collect();
    // A quebra de linha pode ter caído num espaço do nome ou no meio de um campo.
    [trimmed.concat(), trimmed.join(" ")]
        .iter()
        .find_map(|joined| {
            let start = joined.find("000201")?;
            let tail = &joined[start..];
            // O CRC fecha o código: tenta cada "6304" seguido de 4 dígitos hexadecimais.
            tail.match_indices("6304")
                .filter_map(|(i, _)| tail.get(..i + 8))
                .find_map(parse)
        })
}

/// Chave estável do código para não lançar duas vezes a mesma cobrança.
pub fn dedup_key(c: &PixCode) -> String {
    use sha2::Digest;
    let h = sha2::Sha256::digest(c.payload.as_bytes());
    format!("pix:{}", &hex::encode(h)[..24])
}
