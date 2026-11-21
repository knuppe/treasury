use std::collections::BTreeMap;

use serde_json::json;

use super::*;
use crate::api::{Actor, Api};
use crate::paths::TreasuryPaths;

fn answers(pairs: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
    pairs
        .iter()
        .map(|(q, a)| (q.to_string(), a.iter().map(|s| s.to_string()).collect()))
        .collect()
}

#[test]
fn levels_follow_the_answers() {
    let beginner = answers(&[
        ("confianca", &["nada"]),
        ("controle", &["nunca"]),
        ("objetivo", &["onde_vai", "reserva"]),
        ("reserva", &["nao_sei"]),
        ("renda", &["beneficios"]),
        ("termos", &["perdido"]),
        ("tom", &["simples"]),
    ]);
    assert_eq!(score(&beginner), (Level::Iniciante, Tone::Simples));
    let middle = answers(&[
        ("confianca", &["mais_ou_menos"]),
        ("controle", &["planilha"]),
        ("objetivo", &["dividas"]),
        ("reserva", &["um_pouco"]),
        ("termos", &["mais_ou_menos"]),
        ("tom", &["equilibrado"]),
    ]);
    assert_eq!(score(&middle), (Level::Intermediario, Tone::Equilibrado));
    let advanced = answers(&[
        ("confianca", &["muito"]),
        ("controle", &["detalhado"]),
        ("objetivo", &["investir"]),
        ("reserva", &["suficiente"]),
        ("termos", &["entendo"]),
        ("tom", &["tecnico"]),
    ]);
    assert_eq!(score(&advanced), (Level::Avancado, Tone::Tecnico));
    // The chosen tone counts (and also weighs on the level); without a tone, the level's one.
    let mixed = answers(&[
        ("confianca", &["muito"]),
        ("controle", &["detalhado"]),
        ("tom", &["simples"]),
    ]);
    assert_eq!(score(&mixed), (Level::Intermediario, Tone::Simples));
    assert_eq!(
        score(&answers(&[("confianca", &["muito"])])),
        (Level::Avancado, Tone::Tecnico)
    );
    assert_eq!(
        score(&answers(&[("renda", &["fixa"])])),
        (Level::Intermediario, Tone::Equilibrado)
    );
    // Invalid answers.
    assert!(validate(&answers(&[("confianca", &["nada", "muito"])])).is_err());
    assert!(validate(&answers(&[("xyz", &["a"])])).is_err());
    assert!(validate(&answers(&[("objetivo", &["investir", "reserva"])])).is_ok());
    let p = from_answers(beginner).unwrap();
    assert_eq!(p.income.as_deref(), Some("beneficios"));
    assert!(
        prompt_line(&p).contains("government benefits"),
        "{}",
        prompt_line(&p)
    );
}

#[test]
fn new_vault_asks_existing_vault_stays_advanced() {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana"]}),
    )
    .unwrap();
    let p = api.call("profile.get", json!({})).unwrap();
    assert_eq!(p["onboarding_pending"], true);
    // Seven questions and, only in the family questionnaire, the life-line one (no points).
    assert_eq!(
        api.call("profile.questions", json!({}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        8
    );
    // Whoever already has data does not see the questionnaire and stays on advanced.
    api.call(
        "accounts.create",
        json!({"name": "Conta", "kind": "checking"}),
    )
    .unwrap();
    let p = api.call("profile.get", json!({})).unwrap();
    assert_eq!(
        (p["onboarding_pending"].clone(), p["level"].clone()),
        (json!(false), json!("avancado"))
    );
    assert_eq!(p["source"], "padrao");

    let p = api
        .call(
            "profile.answer",
            json!({"answers": {"confianca": ["nada"], "controle": ["nunca"], "termos": ["perdido"]}}),
        )
        .unwrap();
    assert_eq!(p["level"], "iniciante");
    assert_eq!(
        api.call("profile.get", json!({})).unwrap()["source"],
        "questionario"
    );
    let p = api
        .call("profile.set", json!({"level": "intermediario"}))
        .unwrap();
    assert_eq!(
        (p["level"].clone(), p["source"].clone()),
        (json!("intermediario"), json!("escolha"))
    );
    assert_eq!(p["answers"]["confianca"][0], "nada", "the answers are kept");
    let p = api.call("profile.skip", json!({})).unwrap();
    assert_eq!(p["level"], "avancado");
    assert!(
        api.call("profile.answer", json!({"answers": {"tom": ["alto"]}}))
            .is_err()
    );
    // The agent reads, but does not change.
    let mcp = Actor::Mcp {
        client: "x".into(),
        read_only: false,
    };
    assert!(api.call_as(&mcp, "profile.get", json!({})).is_ok());
    assert!(api.call_as(&mcp, "profile.skip", json!({})).is_err());
}

/// The questionnaire comes out in the installation language: every question and option has text
/// in the catalog (none shows up as a key) and changes with the language.
#[test]
fn questions_follow_the_installation_locale() {
    use crate::i18n::{Locale, set_current};
    for l in [Locale::PtBr, Locale::En, Locale::Es] {
        set_current(l);
        for q in questions() {
            assert!(!q.title.starts_with("nucleo-"), "{}: {}", l.code(), q.title);
            assert!(q.options.iter().all(|o| !o.label.starts_with("nucleo-")));
        }
        assert!(!Level::Avancado.label().starts_with("nucleo-"));
    }
    set_current(Locale::En);
    let q = questions();
    let e = validate(&answers(&[("tom", &["simples", "tecnico"])]))
        .unwrap_err()
        .message();
    set_current(Locale::PtBr);
    assert_eq!(q[0].title, "How do you feel about managing your money?");
    assert_eq!(q[2].hint.as_deref(), Some("You can choose more than one."));
    assert_eq!(e, "question tom accepts only one answer");
    assert_eq!(
        questions()[0].options[0].label,
        "Nada seguro: quero aprender o básico"
    );
    assert_eq!(Level::Intermediario.label(), "Intermediário");
}
