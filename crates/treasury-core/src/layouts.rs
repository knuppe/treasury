//! Layouts de arquivos importados (docs/layouts.md): cada importador declara os layouts que
//! sabe ler — assunto (extrato de previdência, nota de corretagem, extrato de banco…), emissor
//! e versão — e a importação começa identificando o arquivo. Um arquivo que parece do assunto
//! (pelos marcadores do assunto) mas não bate com nenhum layout conhecido é recusado com um
//! aviso claro ("pode ser uma versão nova do extrato"), em vez de importado errado.
//!
//! Para suportar uma versão nova de um extrato: escreva o leitor, acrescente um `Def` aqui
//! (com `matches` que reconheça a versão nova e não as outras) e um teste com um arquivo de
//! exemplo. As versões antigas continuam valendo para os arquivos antigos.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::error::{Error, Result};
use crate::i18n::{args, t, t_args};
use crate::import::text::fold;

/// Do que o arquivo trata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum LayoutSubject {
    BankStatement,
    CardStatement,
    BenefitStatement,
    BrokerageNote,
    PensionStatement,
    Payslip,
}

impl LayoutSubject {
    /// Nome do assunto no idioma da instalação.
    pub fn label(self) -> String {
        t(match self {
            LayoutSubject::BankStatement => "nucleo-base-layout-extrato-conta",
            LayoutSubject::CardStatement => "nucleo-base-layout-fatura-cartao",
            LayoutSubject::BenefitStatement => "nucleo-base-layout-extrato-beneficio",
            LayoutSubject::BrokerageNote => "nucleo-base-layout-nota-corretagem",
            LayoutSubject::PensionStatement => "nucleo-base-layout-extrato-previdencia",
            LayoutSubject::Payslip => "nucleo-base-layout-holerite",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum LayoutFormat {
    Pdf,
    Csv,
    Ofx,
}

/// Um layout conhecido.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LayoutInfo {
    /// `pension.quanta.v1-colunas`, `csv.flash`, `notes.sinacor`…
    pub id: String,
    pub subject: LayoutSubject,
    pub format: LayoutFormat,
    pub issuer: String,
    pub version: String,
    pub description: String,
}

/// O que foi reconhecido num arquivo.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Identification {
    pub layout: Option<LayoutInfo>,
    /// O assunto pelos marcadores, mesmo sem layout conhecido.
    pub subject: Option<LayoutSubject>,
    /// Por que não foi reconhecido (com a lista do que se sabe ler).
    pub problem: Option<String>,
}

/// O arquivo como os layouts o veem.
pub struct Doc<'a> {
    pub format: LayoutFormat,
    /// Texto do PDF (ou do CSV).
    pub text: &'a str,
    /// Cabeçalho do CSV.
    pub headers: &'a [String],
}

struct Def {
    id: String,
    subject: LayoutSubject,
    format: LayoutFormat,
    issuer: String,
    version: String,
    description: String,
    matches: Box<dyn Fn(&Doc<'_>) -> bool>,
}

impl Def {
    fn info(&self) -> LayoutInfo {
        LayoutInfo {
            id: self.id.clone(),
            subject: self.subject,
            format: self.format,
            issuer: self.issuer.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
        }
    }
}

/// Marcadores de assunto (texto já sem acentos e em minúsculas).
fn subject_hint(doc: &Doc<'_>) -> Option<LayoutSubject> {
    if doc.format != LayoutFormat::Pdf {
        return None;
    }
    let t = fold(doc.text);
    if t.contains("nota de corretagem") || t.contains("nota de negociacao") {
        return Some(LayoutSubject::BrokerageNote);
    }
    if t.contains("extrato financeiro do participante")
        || t.contains("cnpb")
        || (t.contains("previdencia") && t.contains("cotas"))
    {
        return Some(LayoutSubject::PensionStatement);
    }
    if crate::income::payslip::looks_like(doc.text) {
        return Some(LayoutSubject::Payslip);
    }
    if crate::import::card_pdf::looks_like(doc.text) {
        return Some(LayoutSubject::CardStatement);
    }
    None
}

fn defs() -> Vec<Def> {
    let mut v: Vec<Def> = vec![
        Def {
            id: "pension.quanta.v1-colunas".into(),
            subject: LayoutSubject::PensionStatement,
            format: LayoutFormat::Pdf,
            issuer: t("nucleo-base-layout-quanta"),
            version: t("nucleo-base-layout-quanta-colunas"),
            description: t("nucleo-base-layout-quanta-colunas-descricao"),
            matches: Box::new(|d| crate::pension::statement::matches_columns(d.text)),
        },
        Def {
            id: "pension.quanta.v1-colado".into(),
            subject: LayoutSubject::PensionStatement,
            format: LayoutFormat::Pdf,
            issuer: t("nucleo-base-layout-quanta"),
            version: t("nucleo-base-layout-quanta-colado"),
            description: t("nucleo-base-layout-quanta-colado-descricao"),
            matches: Box::new(|d| crate::pension::statement::matches_glued(d.text)),
        },
        Def {
            id: "notes.sinacor".into(),
            subject: LayoutSubject::BrokerageNote,
            format: LayoutFormat::Pdf,
            issuer: t("nucleo-base-layout-sinacor"),
            version: "v1".into(),
            description: t("nucleo-base-layout-sinacor-descricao"),
            matches: Box::new(|d| !crate::import::notes::parse::parse_text(d.text).is_empty()),
        },
        Def {
            id: "payslip.generico.v1".into(),
            subject: LayoutSubject::Payslip,
            format: LayoutFormat::Pdf,
            issuer: t("nucleo-base-layout-generico-empresa"),
            version: "v1".into(),
            description: t("nucleo-base-layout-holerite-descricao"),
            matches: Box::new(|d| crate::income::payslip::matches_generic(d.text)),
        },
        Def {
            id: "card.generico.v1".into(),
            subject: LayoutSubject::CardStatement,
            format: LayoutFormat::Pdf,
            issuer: t("nucleo-base-layout-generico-banco"),
            version: "v1".into(),
            description: t("nucleo-base-layout-fatura-descricao"),
            matches: Box::new(|d| crate::import::card_pdf::matches_generic(d.text)),
        },
        Def {
            id: "bank.caixa.extrato-periodo.v1".into(),
            subject: LayoutSubject::BankStatement,
            format: LayoutFormat::Pdf,
            issuer: t("nucleo-base-layout-caixa"),
            version: t("nucleo-base-layout-caixa-periodo"),
            description: t("nucleo-base-layout-caixa-periodo-descricao"),
            matches: Box::new(|d| crate::import::caixa::looks_like(d.text)),
        },
        Def {
            id: "ofx".into(),
            subject: LayoutSubject::BankStatement,
            format: LayoutFormat::Ofx,
            issuer: t("nucleo-base-layout-ofx"),
            version: "v1".into(),
            description: t("nucleo-base-layout-ofx-descricao"),
            matches: Box::new(|d| d.format == LayoutFormat::Ofx),
        },
    ];
    for p in crate::import::csv_file::presets() {
        if p.id == crate::import::csv_file::GENERIC_PRESET {
            continue;
        }
        let id = p.id.clone();
        let subject = if id == "flash" {
            LayoutSubject::BenefitStatement
        } else if p.mapping.invert_sign || id.contains("cartao") {
            LayoutSubject::CardStatement
        } else {
            LayoutSubject::BankStatement
        };
        v.push(Def {
            id: format!("csv.{id}"),
            subject,
            format: LayoutFormat::Csv,
            issuer: String::new(),
            version: "v1".into(),
            description: p.name.clone(),
            matches: Box::new(move |d| {
                d.format == LayoutFormat::Csv
                    && crate::import::csv_file::suggest_preset(d.headers).as_deref() == Some(&id)
            }),
        });
    }
    v
}

/// Nota para o agente sobre um PDF anexado: o layout reconhecido e as ferramentas que o leem
/// (ou o aviso de layout desconhecido).
pub fn agent_note(text: &str) -> Option<String> {
    let id = identify(&Doc {
        format: LayoutFormat::Pdf,
        text,
        headers: &[],
    });
    let tools = |s: LayoutSubject| {
        let pair = |a: &str, b: &str| {
            t_args(
                "nucleo-base-layout-ferramentas",
                &args(&[
                    ("leitura", a.to_string().into()),
                    ("importacao", b.to_string().into()),
                ]),
            )
        };
        match s {
            LayoutSubject::PensionStatement => {
                pair("preview_pension_statement", "import_pension_statement")
            }
            LayoutSubject::BrokerageNote => {
                pair("preview_brokerage_notes", "import_brokerage_notes")
            }
            LayoutSubject::Payslip => pair("preview_payslip", "import_payslip"),
            LayoutSubject::CardStatement => t("nucleo-base-layout-ferramentas-fatura"),
            _ => "import_statement_file".to_string(),
        }
    };
    match (id.layout, id.problem) {
        (Some(l), _) => Some(t_args(
            "nucleo-base-layout-reconhecido",
            &args(&[
                ("assunto", l.subject.label().into()),
                ("emissor", l.issuer.clone().into()),
                ("versao", l.version.clone().into()),
                ("layout", l.id.clone().into()),
                ("ferramentas", tools(l.subject).into()),
            ]),
        )),
        (None, Some(p)) => Some(t_args(
            "nucleo-base-layout-atencao",
            &args(&[("problema", p.into())]),
        )),
        _ => None,
    }
}

/// Todos os layouts conhecidos (ou só os de um assunto).
pub fn known(subject: Option<LayoutSubject>) -> Vec<LayoutInfo> {
    defs()
        .iter()
        .filter(|d| subject.is_none_or(|s| s == d.subject))
        .map(Def::info)
        .collect()
}

fn known_list(subject: LayoutSubject) -> String {
    known(Some(subject))
        .iter()
        .map(|l| {
            if l.issuer.is_empty() {
                format!("{} {}", l.description, l.version)
            } else {
                format!("{} {}", l.issuer, l.version)
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// Identifica o layout do arquivo.
pub fn identify(doc: &Doc<'_>) -> Identification {
    let defs = defs();
    if let Some(d) = defs
        .iter()
        .find(|d| d.format == doc.format && (d.matches)(doc))
    {
        return Identification {
            layout: Some(d.info()),
            subject: Some(d.subject),
            problem: None,
        };
    }
    let subject = subject_hint(doc);
    Identification {
        layout: None,
        subject,
        problem: subject.map(unknown_message),
    }
}

fn unknown_message(s: LayoutSubject) -> String {
    t_args(
        "nucleo-base-layout-desconhecido",
        &args(&[
            ("assunto", s.label().into()),
            ("conhecidos", known_list(s).into()),
        ]),
    )
}

/// O layout do arquivo para um assunto, ou um erro claro: layout desconhecido do assunto
/// (talvez uma versão nova) ou arquivo de outro assunto.
pub fn require(doc: &Doc<'_>, subject: LayoutSubject) -> Result<LayoutInfo> {
    let id = identify(doc);
    match (id.layout, id.subject) {
        (Some(l), _) if l.subject == subject => Ok(l),
        (Some(l), _) => Err(Error::invalid(t_args(
            "nucleo-base-layout-outro-assunto",
            &args(&[
                ("assunto", l.subject.label().into()),
                ("descricao", l.description.clone().into()),
                ("esperado", subject.label().into()),
            ]),
        ))),
        (None, Some(s)) if s == subject => Err(Error::invalid(unknown_message(s))),
        _ => Err(Error::invalid(t_args(
            "nucleo-base-layout-nao-reconhecido",
            &args(&[
                ("assunto", subject.label().into()),
                ("conhecidos", known_list(subject).into()),
            ]),
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pdf(text: &str) -> Identification {
        identify(&Doc {
            format: LayoutFormat::Pdf,
            text,
            headers: &[],
        })
    }

    #[test]
    fn identifies_known_layouts_and_refuses_unknown_versions() {
        let cols = include_str!("pension/fixtures/quanta_extrato.txt");
        let glued = include_str!("pension/fixtures/quanta_pdf_extract.txt");
        assert_eq!(pdf(cols).layout.unwrap().id, "pension.quanta.v1-colunas");
        assert_eq!(pdf(glued).layout.unwrap().id, "pension.quanta.v1-colado");
        // Mesmo assunto, layout novo: reconhece o assunto e avisa, sem layout.
        let novo = "Extrato Financeiro do Participante\nPlano X\nSaldo de cotas 10,000000";
        let id = pdf(novo);
        assert!(id.layout.is_none());
        assert_eq!(id.subject, Some(LayoutSubject::PensionStatement));
        let err = require(
            &Doc {
                format: LayoutFormat::Pdf,
                text: novo,
                headers: &[],
            },
            LayoutSubject::PensionStatement,
        )
        .unwrap_err();
        assert!(err.to_string().contains("versão nova"), "{err}");
        assert!(err.to_string().contains("Quanta"), "{err}");
        // Outro assunto.
        let err = require(
            &Doc {
                format: LayoutFormat::Pdf,
                text: "boleto qualquer",
                headers: &[],
            },
            LayoutSubject::PensionStatement,
        )
        .unwrap_err();
        assert!(err.to_string().contains("não reconheci"), "{err}");
        // CSV pelo cabeçalho.
        let headers: Vec<String> = [
            "Data",
            "Hora",
            "Movimentação",
            "Valor",
            "Meio de Pagamento",
            "Saldo",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let csv = identify(&Doc {
            format: LayoutFormat::Csv,
            text: "",
            headers: &headers,
        });
        let l = csv.layout.unwrap();
        assert_eq!(
            (l.id.as_str(), l.subject),
            ("csv.flash", LayoutSubject::BenefitStatement)
        );
        assert!(known(None).len() >= 8);
    }

    #[test]
    fn problems_follow_the_locale() {
        use crate::i18n::{Locale, set_current};
        let novo = "Extrato Financeiro do Participante\nPlano X\nSaldo de cotas 10,000000";
        set_current(Locale::En);
        let id = pdf(novo);
        let note = agent_note(novo).unwrap();
        set_current(Locale::PtBr);
        let problem = id.problem.unwrap();
        assert!(
            problem.starts_with("the file looks like a pension statement, but doesn't match"),
            "{problem}"
        );
        assert!(problem.contains("v1 (2023), text in columns"), "{problem}");
        assert!(note.starts_with("[Warning: the file looks like"), "{note}");
        assert!(
            pdf(novo)
                .problem
                .unwrap()
                .starts_with("o arquivo parece um extrato de previdência")
        );
    }
}
