//! Textos do histórico de demonstração no idioma do app (catálogo `demodados.ftl`) e o que
//! muda com o país da família: o 13º e o INSS no Brasil, os subsídios de Natal e de férias em
//! Portugal, os ativos da carteira e os nomes de lojas e empresas (estes seguem o país, não o
//! idioma).

use fluent_bundle::FluentValue;

use crate::i18n::{self, Locale};

/// Nomes de lojas, corretoras e apps (seguem o país da família).
const BRANDS: &[&str] = &[
    "demodados-corretora",
    "demodados-corretora-instituicao",
    "demodados-mercados",
    "demodados-postos",
    "demodados-apps",
    "demodados-farmacias",
    "demodados-delivery",
    "demodados-cinema",
    "demodados-loja-roupa",
    "demodados-loja-casa",
    "demodados-loja-eletronicos",
    "demodados-atacarejo",
    "demodados-lojas-presente",
];

pub(super) struct Texts {
    pub l: Locale,
    /// País da família (ISO, maiúsculo).
    pub country: String,
}

impl Texts {
    pub fn new(country: &str) -> Self {
        Self {
            l: i18n::current(),
            country: country.to_ascii_uppercase(),
        }
    }

    pub fn t(&self, key: &str) -> String {
        self.brand(key)
            .unwrap_or_else(|| i18n::tr(self.l, key, None))
    }

    /// Lojas, corretoras e apps são os do país, não os do idioma: uma família de Portugal com
    /// o app em pt-BR vê Continente e Galp; a da Espanha, Mercadona e Repsol
    /// (docs/international.md). A variante do país (`--es`) vem antes; depois, o catálogo do
    /// idioma do país.
    fn brand(&self, key: &str) -> Option<String> {
        if !BRANDS.contains(&key) {
            return None;
        }
        let variant = format!("{key}--{}", self.country.to_ascii_lowercase());
        if i18n::has_key(Locale::PtBr, &variant) {
            return Some(i18n::tr(self.l, &variant, None));
        }
        let l = match self.country.as_str() {
            "BR" => Locale::PtBr,
            "US" => Locale::En,
            "PT" => Locale::PtPt,
            _ => return None,
        };
        Some(i18n::tr(l, key, None))
    }

    pub fn ta(&self, key: &str, a: &[(&'static str, String)]) -> String {
        let list: Vec<(&'static str, FluentValue<'static>)> = a
            .iter()
            .map(|(k, v)| (*k, FluentValue::from(v.clone())))
            .collect();
        i18n::tr(self.l, key, Some(&i18n::args(&list)))
    }

    /// Lista separada por vírgulas no catálogo (lojas, empresas, clientes).
    pub fn list(&self, key: &str) -> Vec<String> {
        self.t(key)
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn brazil(&self) -> bool {
        self.country == "BR"
    }

    pub fn portugal(&self) -> bool {
        self.country == "PT"
    }

    pub fn usa(&self) -> bool {
        self.country == "US"
    }
}
