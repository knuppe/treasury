//! B3 COTAHIST diário (arquivo público e gratuito, `COTAHIST_DDDMMAAAA.ZIP`): fechamento e
//! volume financeiro de todos os papéis do mercado à vista num pregão. Base das cotações e
//! da liquidez do universo de rankings e screener. Layout fixo de 245 posições.

use chrono::NaiveDate;
use rust_decimal::Decimal;

pub fn daily_url(d: NaiveDate) -> String {
    format!(
        "https://bvmf.bmfbovespa.com.br/InstDados/SerHist/COTAHIST_D{}.ZIP",
        d.format("%d%m%Y")
    )
}

/// Arquivo mensal do COTAHIST (`COTAHIST_MMMAAAA.ZIP`, ~10 MB): usado uma vez por mês
/// fechado para o histórico de 13 meses (volatilidade, momentum e liquidez do universo).
pub fn monthly_url(year: i32, month: u32) -> String {
    format!("https://bvmf.bmfbovespa.com.br/InstDados/SerHist/COTAHIST_M{month:02}{year}.ZIP")
}

#[derive(Debug, Clone, PartialEq)]
pub struct Quote {
    pub ticker: String,
    pub date: NaiveDate,
    pub close: Decimal,
    /// Volume financeiro do dia (R$).
    pub volume: Decimal,
    pub trades: i64,
    pub isin: String,
    /// Nome de pregão (NOMRES, ex.: "FII GGRCOVEP") e especificação (ESPECI, ex.: "CI"):
    /// o que as notas SINACOR trazem na coluna "Especificação do título".
    pub short_name: String,
    pub spec: String,
    /// Código BDI (02 lote padrão, 12 FII, 14 ETF, 34–36 BDR…).
    pub bdi: String,
}

/// Códigos BDI do mercado à vista que interessam: lote padrão (02), recuperação (07, 08),
/// FIIs (12), FIAGRO (13), ETFs e certificados (14) e BDRs (34, 35, 36).
const BDI: &[&str] = &["02", "07", "08", "12", "13", "14", "34", "35", "36"];

fn slice(line: &[u8], from: usize, to: usize) -> &str {
    line.get(from..to)
        .and_then(|b| std::str::from_utf8(b).ok())
        .unwrap_or("")
        .trim()
}

fn num(s: &str, scale: u32) -> Option<Decimal> {
    let n: i128 = s.parse().ok()?;
    Some(
        Decimal::try_from_i128_with_scale(n, scale)
            .ok()?
            .normalize(),
    )
}

/// Linhas de cotação (registro 01) do mercado à vista.
pub fn parse(text: &[u8]) -> Vec<Quote> {
    let mut out = vec![];
    for line in text.split(|b| *b == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if line.len() < 245 || &line[0..2] != b"01" || slice(line, 24, 27) != "010" {
            continue;
        }
        if !BDI.contains(&slice(line, 10, 12)) {
            continue;
        }
        let Ok(date) = NaiveDate::parse_from_str(slice(line, 2, 10), "%Y%m%d") else {
            continue;
        };
        let (Some(last), Some(volume)) =
            (num(slice(line, 108, 121), 2), num(slice(line, 170, 188), 2))
        else {
            continue;
        };
        let factor = num(slice(line, 210, 217), 0)
            .filter(|f| !f.is_zero())
            .unwrap_or(Decimal::ONE);
        if last.is_zero() {
            continue;
        }
        out.push(Quote {
            ticker: slice(line, 12, 24).to_uppercase(),
            date,
            close: (last / factor).normalize(),
            volume,
            trades: slice(line, 147, 152).parse().unwrap_or(0),
            isin: slice(line, 230, 242).to_string(),
            short_name: slice(line, 27, 39).to_string(),
            spec: slice(line, 39, 49).to_string(),
            bdi: slice(line, 10, 12).to_string(),
        });
    }
    out
}

/// Lê o zip diário (uma entrada `.TXT`).
pub fn parse_zip(bytes: &[u8]) -> Result<Vec<Quote>, String> {
    let mut zip = super::cvm::Zip::open(bytes)?;
    let Some(name) = zip.find(|n| n.ends_with(".txt")) else {
        return Err(crate::i18n::t("nucleo-pesquisa-erro-cotahist-sem-txt"));
    };
    let mut buf = vec![];
    zip.read_all(&name, &mut buf)?;
    Ok(parse(&buf))
}

#[cfg(test)]
pub(crate) mod fixtures {
    /// Uma linha no layout oficial (245 posições).
    pub fn line(
        date: &str,
        bdi: &str,
        ticker: &str,
        close_cents: i64,
        volume_cents: i64,
        isin: &str,
    ) -> String {
        let mut s = String::new();
        s.push_str("01");
        s.push_str(date);
        s.push_str(bdi);
        s.push_str(&format!("{ticker:<12}"));
        s.push_str("010");
        s.push_str(&format!("{:<12}", "NOME"));
        s.push_str(&format!("{:<10}", "ON"));
        s.push_str("   ");
        s.push_str("R$  ");
        for _ in 0..4 {
            s.push_str(&format!("{close_cents:013}"));
        }
        s.push_str(&format!("{close_cents:013}"));
        s.push_str(&format!("{:013}", 0));
        s.push_str(&format!("{:013}", 0));
        s.push_str(&format!("{:05}", 1234));
        s.push_str(&format!("{:018}", 1000));
        s.push_str(&format!("{volume_cents:018}"));
        s.push_str(&format!("{:013}", 0));
        s.push('0');
        s.push_str("99991231");
        s.push_str("0000001");
        s.push_str(&format!("{:013}", 0));
        s.push_str(&format!("{isin:<12}"));
        s.push_str("100");
        assert_eq!(s.len(), 245);
        s
    }

    pub fn zip(date: &str, rows: &[(&str, &str, i64, i64)]) -> Vec<u8> {
        let mut txt = format!("00COTAHIST.2026BOVESPA {date}{}\r\n", " ".repeat(214));
        for (bdi, t, c, v) in rows {
            txt.push_str(&line(date, bdi, t, *c, *v, "BRXXXXACNOR0"));
            txt.push_str("\r\n");
        }
        super::super::cvm::fixtures::zip_latin1(&[(&format!("COTAHIST_D{date}.TXT"), txt)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn parses_cotahist_lines() {
        let z = fixtures::zip(
            "20261013",
            &[
                ("02", "PETR4", 3855, 216_425_115_200),
                ("12", "HGLG11", 15634, 290_610_229),
                ("96", "PETR4F", 3850, 1_000),
                ("78", "PETRJ400", 120, 5_000),
            ],
        );
        let q = parse_zip(&z).unwrap();
        assert_eq!(q.len(), 2, "fracionário e opções ficam de fora");
        assert_eq!(q[0].ticker, "PETR4");
        assert_eq!(q[0].close, dec!(38.55));
        assert_eq!(q[0].volume, dec!(2164251152));
        assert_eq!(q[0].date.to_string(), "2026-10-13");
        assert_eq!(q[1].close, dec!(156.34));
        assert_eq!(
            daily_url(NaiveDate::from_ymd_opt(2026, 10, 9).unwrap()),
            "https://bvmf.bmfbovespa.com.br/InstDados/SerHist/COTAHIST_D09102026.ZIP"
        );
    }
}
