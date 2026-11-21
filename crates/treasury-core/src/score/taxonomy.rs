//! Taxonomia (§3): ativo → classe da metodologia → segmento → perfil de pesos. Ações BR
//! pelo setor do FCA (holdings herdam o setor da investida principal informado no nome do
//! setor), com o leiaute das demonstrações (banco, seguradora) acima do setor; FIIs pela
//! composição do ativo. O usuário pode trocar o segmento (override por ticker).

use std::collections::BTreeMap;

use rusqlite::Connection;

use super::config::Methodology;
use crate::assets::AssetClass;
use crate::error::{Error, Result};
use crate::research::Layout;

/// Ajuste com os segmentos escolhidos pelo usuário (ticker → segmento).
pub const OVERRIDES_SETTING: &str = "score.segment_overrides";

/// Classe da metodologia para uma classe de ativo do cadastro.
pub fn class_of(c: AssetClass) -> &'static str {
    match c {
        AssetClass::StockBr => "equity_br",
        AssetClass::Fii => "reit",
        AssetClass::Reit => "reit",
        AssetClass::StockUs | AssetClass::StockIntl | AssetClass::Bdr => "equity_intl",
        AssetClass::TreasuryBr => "fi_sovereign",
        AssetClass::FixedIncome => "fi_bank",
        AssetClass::EtfBr | AssetClass::EtfUs | AssetClass::EtfIntl => "etf",
        AssetClass::Fund | AssetClass::Pension => "fund",
        AssetClass::Crypto => "crypto",
        AssetClass::Other => "other",
    }
}

/// Setor normalizado (sem acento e pontuação; holdings sem o prefixo "Emp. Adm. Part.").
pub fn normalize_sector(s: &str) -> String {
    let n = crate::research::norm(s);
    let n = n
        .strip_prefix("emp adm part ")
        .map(str::to_string)
        .unwrap_or(n);
    super::config::methodology()
        .equity_br
        .sector_aliases
        .get(&n)
        .cloned()
        .unwrap_or(n)
}

/// Segmento de uma ação BR.
pub fn equity_segment(m: &Methodology, sector: Option<&str>, layout: Option<Layout>) -> String {
    let e = &m.equity_br;
    if let Some(l) = layout
        && let Some(s) = e.layout_segments.get(l.as_str())
    {
        return s.clone();
    }
    if let Some(sec) = sector {
        let n = normalize_sector(sec);
        for r in &e.sector_map {
            if !r.sectors.contains(&n) {
                continue;
            }
            // Setor de banco com demonstrações de empresa comum: sem os indicadores do
            // IF.data (o CNPJ não é de banco), o perfil CAMELS nunca fecha.
            if let (Some(need), Some(l)) = (e.layout_required.get(&r.segment), layout)
                && need != l.as_str()
            {
                let holding = crate::research::norm(sec).starts_with("emp adm part ");
                return match (&e.holding_segment, holding) {
                    (Some(h), true) => h.clone(),
                    _ => e.default_segment.clone(),
                };
            }
            return r.segment.clone();
        }
    }
    e.default_segment.clone()
}

/// Composição do ativo de um FII (frações do ativo total).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ReitComposition {
    pub cri: Option<f64>,
    pub fii: Option<f64>,
    pub real_estate: Option<f64>,
}

/// Segmento de um FII pela composição do ativo (papel, fundo de fundos, tijolo, híbrido).
pub fn reit_segment(m: &Methodology, c: ReitComposition) -> String {
    let r = &m.reit;
    if c.cri
        .is_some_and(|x| x >= r.composition.papel_min_cri_share)
    {
        "papel".into()
    } else if c.fii.is_some_and(|x| x >= r.composition.fof_min_fii_share) {
        "fof".into()
    } else if c
        .real_estate
        .is_some_and(|x| x >= r.composition.tijolo_min_real_estate_share)
    {
        "tijolo".into()
    } else {
        r.default_segment.clone()
    }
}

/// Subsegmento do FII usado como "setor" dos pares (sem os rótulos genéricos).
pub fn reit_subsegment(m: &Methodology, segmento_atuacao: Option<&str>) -> Option<String> {
    let s = segmento_atuacao?.trim();
    let n = crate::research::norm(s);
    (!n.is_empty() && !m.reit.noisy_subsegments.contains(&n)).then(|| s.to_string())
}

/// Segmento escolhido pelo usuário para um ticker (se houver).
pub fn overrides(conn: &Connection) -> Result<BTreeMap<String, String>> {
    Ok(crate::settings::get(conn, OVERRIDES_SETTING)?.unwrap_or_default())
}

/// Grava (ou apaga, com `None`) o segmento de um ticker; confere se existe na classe.
pub fn set_override(
    conn: &Connection,
    m: &Methodology,
    class: &str,
    ticker: &str,
    segment: Option<&str>,
) -> Result<BTreeMap<String, String>> {
    let t = ticker.trim().to_uppercase();
    if t.is_empty() {
        return Err(Error::invalid(crate::i18n::t("nucleo-score-erro-ticker")));
    }
    let mut all = overrides(conn)?;
    match segment {
        None => {
            all.remove(&t);
        }
        Some(s) => {
            let ok = m.segments_of(class).iter().any(|(k, _)| k == s);
            if !ok {
                return Err(Error::invalid(crate::i18n::t_args(
                    "nucleo-score-erro-segmento",
                    &crate::i18n::args(&[("segmento", s.to_string().into())]),
                )));
            }
            all.insert(t, s.to_string());
        }
    }
    crate::settings::set(conn, OVERRIDES_SETTING, &all)?;
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::config::methodology;

    #[test]
    fn equity_segments_from_fca_sector_and_layout() {
        let m = methodology();
        let seg = |s: &str| equity_segment(m, Some(s), Some(Layout::Industrial));
        assert_eq!(seg("Energia Elétrica"), "utilities");
        assert_eq!(seg("Emp. Adm. Part. - Energia Elétrica"), "utilities");
        assert_eq!(seg("Saneamento, Serv. Água e Gás"), "utilities");
        assert_eq!(seg("Extração Mineral"), "cyclicals");
        assert_eq!(seg("Petróleo e Gás"), "cyclicals");
        assert_eq!(seg("Comércio (Atacado e Varejo)"), "retail");
        assert_eq!(
            seg("Construção Civil, Mat. Constr. e Decoração"),
            "construction"
        );
        assert_eq!(
            seg("Emp. Adm. Part. - Const. Civil, Mat. Const. e Decoração"),
            "construction"
        );
        assert_eq!(seg("Serviços médicos"), "health");
        assert_eq!(seg("Comunicação e Informática"), "tech");
        assert_eq!(
            seg("Máquinas, Equipamentos, Veículos e Peças"),
            "industrials"
        );
        assert_eq!(seg("Emp. Adm. Part. - Sem Setor Principal"), "default");
        assert_eq!(
            equity_segment(m, Some("Bancos"), Some(Layout::Bank)),
            "banks"
        );
        // O leiaute vence o setor (holding de banco com setor genérico).
        assert_eq!(
            equity_segment(
                m,
                Some("Emp. Adm. Part. - Sem Setor Principal"),
                Some(Layout::Bank)
            ),
            "banks"
        );
        assert_eq!(
            equity_segment(
                m,
                Some("Seguradoras e Corretoras"),
                Some(Layout::Industrial)
            ),
            "insurance"
        );
        assert_eq!(equity_segment(m, None, None), "default");
        // Holding de banco com demonstrações de empresa comum (Itaúsa): fora do CAMELS.
        assert_eq!(
            seg("Emp. Adm. Part. - Intermediação Financeira"),
            "holdings"
        );
        assert_eq!(seg("Bancos"), "default");
        // Sem leiaute conhecido, vale o setor.
        assert_eq!(
            equity_segment(m, Some("Emp. Adm. Part. - Intermediação Financeira"), None),
            "banks"
        );
    }

    #[test]
    fn reit_segments_from_composition() {
        let m = methodology();
        let c = |cri, fii, re| ReitComposition {
            cri: Some(cri),
            fii: Some(fii),
            real_estate: Some(re),
        };
        assert_eq!(reit_segment(m, c(0.8, 0.0, 0.0)), "papel");
        assert_eq!(reit_segment(m, c(0.1, 0.7, 0.0)), "fof");
        assert_eq!(reit_segment(m, c(0.0, 0.0, 0.9)), "tijolo");
        assert_eq!(reit_segment(m, c(0.3, 0.2, 0.4)), "hibrido");
        assert_eq!(reit_segment(m, ReitComposition::default()), "hibrido");
        assert_eq!(
            reit_subsegment(m, Some("Logística")).as_deref(),
            Some("Logística")
        );
        assert_eq!(reit_subsegment(m, Some("Multicategoria")), None);
    }

    #[test]
    fn classes_map_to_methodology() {
        assert_eq!(class_of(AssetClass::StockBr), "equity_br");
        assert_eq!(class_of(AssetClass::Fii), "reit");
        assert_eq!(class_of(AssetClass::FixedIncome), "fi_bank");
    }
}
