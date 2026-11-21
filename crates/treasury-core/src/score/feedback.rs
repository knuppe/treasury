//! Dado ausente vira feedback para o desenvolvedor (tela Desenvolvedor): um por ativo para
//! métricas críticas. Não repete enquanto houver um aberto com o mesmo título, não junta ativos
//! diferentes num feedback só e fecha sozinho o do ativo quando o score volta a ser calculado.
//! Métrica sem fonte (`unavailable` na metodologia) é lacuna conhecida, sem aviso.

use rusqlite::{Connection, OptionalExtension};

use super::AssetScore;
use super::config::Methodology;
use crate::devfeedback::{DevFeedbackArea, DevFeedbackInput, DevFeedbackKind};

const CLIENT: &str = "score de ativos";

/// Itens que dizem respeito ao ativo, não à coleta.
/// Faltas que não são lacuna da coleta (não viram aviso ao desenvolvedor): histórico curto da
/// empresa recém-listada e o extrato do fundo de pensão, que só o usuário tem.
const INTRINSIC: &[&str] = &[super::intl::SHORT_HISTORY, super::funds::PLAN_STATEMENT];

fn open_with_title(conn: &Connection, title: &str) -> bool {
    conn.query_row(
        "SELECT 1 FROM dev_feedback WHERE title = ?1 AND status IN ('aberto', 'em_andamento')",
        [title],
        |_| Ok(()),
    )
    .optional()
    .ok()
    .flatten()
    .is_some()
}

/// Título do aviso de um ativo no idioma `l`.
fn title_in(l: crate::i18n::Locale, who: &str) -> String {
    crate::i18n::tr(
        l,
        "nucleo-score-feedback-titulo",
        Some(&crate::i18n::args(&[("ativo", who.to_string().into())])),
    )
}

/// O título do aviso em todos os idiomas (o aviso pode ter sido aberto em outro idioma).
fn titles(who: &str) -> Vec<String> {
    let mut v: Vec<String> = crate::i18n::AVAILABLE
        .iter()
        .map(|l| title_in(*l, who))
        .collect();
    v.sort();
    v.dedup();
    v
}

fn add(conn: &Connection, who: &str, body: String) {
    if titles(who).iter().any(|t| open_with_title(conn, t)) {
        return;
    }
    let title = title_in(crate::i18n::current(), who);
    let _ = crate::devfeedback::add_distinct(
        conn,
        &DevFeedbackInput {
            title,
            body,
            area: DevFeedbackArea::Dados,
            kind: DevFeedbackKind::Dados,
            ..Default::default()
        },
        CLIENT,
    );
}

/// Posição encerrada: o ativo teve operações e hoje não tem quantidade (vendido, resgatado,
/// incorporado). Dado faltando nele não é lacuna a investigar.
fn closed_position(conn: &Connection, asset_id: &str, as_of: &str) -> bool {
    let traded = conn
        .query_row(
            "SELECT EXISTS (SELECT 1 FROM trade WHERE asset_id = ?1)",
            [asset_id],
            |r| r.get::<_, bool>(0),
        )
        .unwrap_or(false);
    traded
        && crate::portfolio::engine::quantity_held(conn, asset_id, None, as_of)
            .is_ok_and(|q| q.is_zero())
}

pub fn record_missing(conn: &Connection, m: &Methodology, s: &AssetScore) {
    let who = s.ticker.clone().unwrap_or_else(|| s.name.clone());
    // Fora da carteira: sem aviso novo, e o aberto fecha sozinho com o motivo.
    if let Some(id) = s.asset_id.as_deref()
        && closed_position(conn, id, &s.as_of)
    {
        let why = crate::i18n::t_args(
            "nucleo-score-feedback-resolvido-encerrado",
            &crate::i18n::args(&[("data", s.as_of.clone().into())]),
        );
        for title in titles(&who) {
            let _ = crate::devfeedback::resolve_auto(conn, &title, CLIENT, &why);
        }
        return;
    }
    // Falta do próprio ativo (empresa recém-listada) não é lacuna da coleta: sem aviso.
    let intrinsic = s
        .missing
        .iter()
        .find(|x| INTRINSIC.contains(&x.key.as_str()));
    let critical: Vec<&super::MissingItem> = s
        .missing
        .iter()
        .filter(|x| x.critical && !INTRINSIC.contains(&x.key.as_str()))
        .collect();
    if critical.is_empty() {
        let (key, detail) = if let Some(x) = intrinsic {
            (
                "nucleo-score-feedback-resolvido-intrinseco",
                x.reason.clone(),
            )
        } else if s.score.is_some() {
            (
                "nucleo-score-feedback-resolvido-calculado",
                s.band_label.clone(),
            )
        } else {
            ("nucleo-score-feedback-resolvido", s.band_meaning.clone())
        };
        let why = crate::i18n::t_args(
            key,
            &crate::i18n::args(&[("data", s.as_of.clone().into()), ("detalhe", detail.into())]),
        );
        for title in titles(&who) {
            let _ = crate::devfeedback::resolve_auto(conn, &title, CLIENT, &why);
        }
    }
    if !critical.is_empty() {
        let body = crate::i18n::t_args(
            "nucleo-score-feedback-corpo",
            &crate::i18n::args(&[
                ("ativo", who.clone().into()),
                ("classe", s.class_label.clone().into()),
                ("perfil", s.profile_label.clone().into()),
                ("data", s.as_of.clone().into()),
                (
                    "itens",
                    critical
                        .iter()
                        .map(|x| format!("- {} ({}): {}", x.label, x.dimension, x.reason))
                        .collect::<Vec<_>>()
                        .join("\n")
                        .into(),
                ),
                ("versao", m.version.clone().into()),
            ]),
        );
        add(conn, &who, body);
    }
    // Métricas sem fonte estruturada são lacunas conhecidas da metodologia (`unavailable`,
    // listadas em docs/asset-score.md): não viram aviso, que nunca fecharia. Fecha os que
    // versões anteriores abriram.
    let _ = close_known_gaps(conn);
}

/// Começo do título dos avisos de lacuna que versões antigas abriam (texto gravado em pt-BR:
/// casa com o que está no banco, não se traduz).
const GAP_PREFIX: &str = "Score de ativos: sem fonte para ";

fn close_known_gaps(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE dev_feedback SET status = 'resolvido', updated_at = ?2,
                body = body || char(10) || char(10) || ?3
          WHERE created_by = ?1 AND status IN ('aberto', 'em_andamento')
            AND substr(title, 1, length(?4)) = ?4",
        rusqlite::params![
            CLIENT,
            crate::time::now_utc(),
            crate::i18n::t("nucleo-score-feedback-lacuna-conhecida"),
            GAP_PREFIX
        ],
    )
}
