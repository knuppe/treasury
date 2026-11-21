//! Linha da vida: a matemática numa tábua montada à mão, o leitor do zip, as tábuas embutidas,
//! as fases, os ajustes, o questionário, o planejador de aposentadoria, os marcos e o MCP.

use std::io::Write as _;

use chrono::NaiveDate;
use serde_json::{Value, json};

use super::data::{TableSex, Tables};
use super::math::{self, Mortality};
use super::*;
use crate::api::{Actor, Api};
use crate::paths::TreasuryPaths;

// ---------------------------------------------------------------------------
// Tábua à mão

/// Uma tábua de um país (`BR`) e três sexos, com `q(sexo, ano, idade)`.
fn hand_table(years: (i32, i32), last_age: u32, q: impl Fn(TableSex, i32, u32) -> f64) -> Tables {
    let sexes = [TableSex::Both, TableSex::Female, TableSex::Male];
    let mut qx = vec![];
    for s in sexes {
        for y in years.0..=years.1 {
            for a in 0..=last_age {
                qx.push((q(s, y, a).clamp(0.0, 1.0) * 65535.0).round() as u16);
            }
        }
    }
    let cohorts = (years.0, years.1);
    let n = (cohorts.1 - cohorts.0 + 1) as usize * 3;
    Tables::from_parts(
        false,
        vec!["BR".into()],
        &["both", "female", "male"],
        years,
        last_age,
        cohorts,
        qx,
        vec![0; n],
        None,
    )
    .unwrap()
}

/// Gompertz simples que melhora 2% ao ano; homens com mortalidade maior.
fn improving() -> Tables {
    hand_table((2024, 2030), 100, |s, y, a| {
        let sex = match s {
            TableSex::Male => 1.4,
            TableSex::Female => 0.8,
            TableSex::Both => 1.1,
        };
        let improve = (-0.02 * f64::from(y - 2024)).exp();
        let q = 0.0005 + sex * 3e-5 * improve * (0.095 * f64::from(a)).exp();
        if a == 100 { 0.45 } else { q.min(0.9) }
    })
}

struct Constant(f64);
impl Mortality for Constant {
    fn q(&self, _: i32, _: u32) -> f64 {
        self.0
    }
}

#[test]
fn survival_falls_and_the_numbers_come_from_the_curve() {
    let t = improving();
    let v = t.view(0, TableSex::Both);
    let c = math::curve(&v, 41, 2026);
    assert_eq!(c.at(41), 1.0);
    assert!(
        c.s.windows(2).all(|w| w[1] <= w[0]),
        "sobrevivência nunca sobe"
    );
    let (m, p25, p10) = (c.quantile(0.5), c.quantile(0.25), c.quantile(0.10));
    assert!(m > 41.0 && m < p25 && p25 < p10, "{m} {p25} {p10}");
    // A média fica perto da mediana, e a mediana divide a curva ao meio.
    let mean = 41.0 + c.remaining_mean();
    assert!((mean - m).abs() < 6.0, "{mean} {m}");
    let below = c.at(m.floor() as u32);
    let above = c.at(m.ceil() as u32);
    assert!(below >= 0.5 && above <= 0.5);
    // Taxa constante: e = 0,5 + (1 − q)/q e a mediana em ln 0,5 / ln (1 − q).
    let k = math::curve(&Constant(0.1), 30, 2026);
    assert!((k.remaining_mean() - 9.5).abs() < 1e-6);
    let median = 30.0 + 0.5f64.ln() / 0.9f64.ln();
    assert!(
        (k.quantile(0.5) - median).abs() < 0.05,
        "{}",
        k.quantile(0.5)
    );
    // Mais velho hoje: mediana mais alta (já passou pelos riscos), restante menor.
    let old = math::curve(&v, 70, 2026);
    assert!(old.quantile(0.5) > m);
    assert!(old.remaining_mean() < c.remaining_mean());
}

#[test]
fn cohort_beats_period_when_mortality_improves() {
    let t = improving();
    for s in [TableSex::Both, TableSex::Female, TableSex::Male] {
        let v = t.view(0, s);
        let cohort = math::cohort_e0(&v, 2024);
        let period = math::period_e0(&v, 2024);
        assert!(cohort > period, "{s:?}: coorte {cohort} ≤ período {period}");
        // Sem melhora, as duas são iguais.
        let flat = hand_table((2024, 2030), 100, |_, _, a| 0.001 + 1e-4 * f64::from(a));
        let f = flat.view(0, s);
        assert!((math::cohort_e0(&f, 2024) - math::period_e0(&f, 2024)).abs() < 1e-9);
    }
    // Mulheres vivem mais que homens; ambos fica entre os dois.
    let e = |s| math::cohort_e0(&t.view(0, s), 2026);
    assert!(e(TableSex::Female) > e(TableSex::Both) && e(TableSex::Both) > e(TableSex::Male));
}

#[test]
fn after_the_last_year_and_the_open_group_the_last_rates_repeat() {
    // q depende do ano e da idade; última idade 10 (grupo aberto com q = 0,5).
    let t = hand_table((2024, 2026), 10, |_, y, a| {
        if a == 10 {
            0.5
        } else {
            0.01 * f64::from(y - 2023) + 0.001 * f64::from(a)
        }
    });
    assert_eq!(
        t.q(0, TableSex::Both, 2100, 3),
        t.q(0, TableSex::Both, 2026, 3)
    );
    assert_eq!(
        t.q(0, TableSex::Both, 2010, 3),
        t.q(0, TableSex::Both, 2024, 3)
    );
    assert_eq!(
        t.q(0, TableSex::Both, 2025, 50),
        t.q(0, TableSex::Both, 2025, 10)
    );
    let v = t.view(0, TableSex::Both);
    let c = math::curve(&v, 8, 2025);
    // Depois dos 10, metade morre a cada ano.
    for age in 11..20 {
        let ratio = c.at(age + 1) / c.at(age);
        assert!((ratio - 0.5).abs() < 1e-4, "{age}: {ratio}");
    }
    // A geração segue a diagonal: aos 9 anos (2026) já usa a taxa de 2026; depois de 2026,
    // repete 2026.
    let q8 = 1.0 - c.at(9);
    assert!((q8 - t.q(0, TableSex::Both, 2025, 8)).abs() < 1e-9);
    let q9 = 1.0 - c.at(10) / c.at(9);
    assert!((q9 - t.q(0, TableSex::Both, 2026, 9)).abs() < 1e-9);
    // A curva acaba (zero na prática) e o fim não quebra a mediana.
    assert!(c.s.last().unwrap() < &1e-11);
    assert!(c.quantile(0.5) > 10.0);
}

#[test]
fn a_couple_lasts_at_least_as_long_as_each() {
    let t = improving();
    let a = math::curve(&t.view(0, TableSex::Male), 45, 2026);
    let b = math::curve(&t.view(0, TableSex::Female), 40, 2026);
    for k in 0..80 {
        let both = math::either_alive(&a, &b, k);
        assert!(
            both >= a.after(k) - 1e-12 && both >= b.after(k) - 1e-12,
            "{k}"
        );
        assert!(both <= 1.0);
    }
    let k10 = math::either_alive_until(&a, &b, 0.10);
    assert!(45.0 + k10 >= a.quantile(0.10) - 1e-9);
    assert!(40.0 + k10 >= b.quantile(0.10) - 1e-9);
}

/// O leitor do zip no formato do script (`scripts/longevity-data.py`).
#[test]
fn the_zip_format_round_trips() {
    let years = (2024, 2025);
    let ages = 3u32;
    let mut qx = vec![];
    for (si, _) in ["both", "female", "male"].iter().enumerate() {
        for y in 0..2u16 {
            for a in 0..=ages as u16 {
                qx.push(1000 * (si as u16 + 1) + 100 * y + a);
            }
        }
    }
    let e0: Vec<u16> = vec![7660, 0, 7990, 7987, 7331, 7300];
    let index = json!({"source": "wpp2024", "countries": ["BR"],
        "sexes": ["both", "female", "male"], "years": [years.0, years.1], "ages": [0, ages],
        "cohort_years": [2025, 2026]});
    let ibge = json!({
        "both": {"qx": [0.012, 1.0], "ex": [76.6, 1.5]},
        "female": {"qx": [0.011, 1.0], "ex": [79.9, 1.6]},
        "male": {"qx": [0.013, 1.0], "ex": [73.3, 1.4]},
    });
    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut z = zip::ZipWriter::new(&mut buf);
        let o = zip::write::SimpleFileOptions::default();
        let bytes = |v: &[u16]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
        for (name, data) in [
            ("index.json", index.to_string().into_bytes()),
            ("qx.bin", bytes(&qx)),
            ("e0_cohort.bin", bytes(&e0)),
            ("ibge2024.json", ibge.to_string().into_bytes()),
        ] {
            z.start_file(name, o).unwrap();
            z.write_all(&data).unwrap();
        }
        z.finish().unwrap();
    }
    let t = Tables::from_zip(buf.get_ref()).unwrap();
    assert!(!t.synthetic);
    assert_eq!(t.country("BR"), Some(0));
    assert_eq!(t.country("US"), None);
    let raw = |v: u16| f64::from(v) / 65535.0;
    assert_eq!(t.q(0, TableSex::Female, 2025, 2), raw(2102));
    assert_eq!(t.q(0, TableSex::Male, 2024, 9), raw(3003), "grupo aberto");
    assert_eq!(t.cohort_e0(0, TableSex::Both, 2025), Some(76.6));
    assert_eq!(t.cohort_e0(0, TableSex::Both, 2026), None, "0 = sem dado");
    assert_eq!(t.cohort_e0(0, TableSex::Male, 1949), None);
    assert_eq!(t.ibge.as_ref().unwrap().of(TableSex::Male).ex[0], 73.3);
    // Tamanho errado é recusado.
    assert!(
        Tables::from_parts(
            false,
            vec!["BR".into()],
            &["both", "female", "male"],
            years,
            ages,
            (2025, 2026),
            vec![0; 3],
            e0.clone(),
            None
        )
        .is_err()
    );
}

/// As tábuas embutidas carregam e cobrem todos os países do catálogo.
#[test]
fn embedded_tables_cover_the_catalog() {
    let t = data::embedded().unwrap();
    assert_eq!(t.years, (2024, 2100));
    assert_eq!(t.last_age, 100);
    assert_eq!(t.cohorts, (1950, 2026));
    for (code, ..) in crate::country::catalog::COUNTRIES {
        assert!(t.country(code).is_some(), "{code} sem tábua");
    }
    assert!(t.ibge.is_some(), "referência do IBGE");
    // A expectativa por geração guardada bate com a calculada aqui (nascidos em 2024..=2026
    // só usam anos guardados).
    let br = t.country("BR").unwrap();
    for s in [TableSex::Both, TableSex::Female, TableSex::Male] {
        for b in 2024..=2026 {
            let stored = t.cohort_e0(br, s, b).unwrap();
            let here = math::cohort_e0(&t.view(br, s), b);
            assert!(
                (stored - here).abs() < 0.05,
                "{s:?} {b}: {stored} vs {here}"
            );
        }
        // Nascidos depois de 1950 vivem mais que a tábua de período de 2024 (coorte ≥
        // período), e as gerações mais novas mais que as antigas.
        let period = math::period_e0(&t.view(br, s), 2024);
        assert!(t.cohort_e0(br, s, 2026).unwrap() > period);
        assert!(t.cohort_e0(br, s, 2026).unwrap() > t.cohort_e0(br, s, 1960).unwrap());
    }
}

#[test]
fn stage_boundaries() {
    let cases = [
        (0, LifeStage::Childhood),
        (11, LifeStage::Childhood),
        (12, LifeStage::Adolescence),
        (17, LifeStage::Adolescence),
        (18, LifeStage::Youth),
        (29, LifeStage::Youth),
        (30, LifeStage::Adulthood),
        (59, LifeStage::Adulthood),
        (60, LifeStage::LaterLife),
        (85, LifeStage::LaterLife),
        (130, LifeStage::LaterLife),
    ];
    for (age, stage) in cases {
        assert_eq!(stage_of(age), stage, "{age}");
    }
    let s = stages();
    assert_eq!(s.len(), 5);
    assert_eq!(s[0].from, 0);
    assert!(
        s.windows(2).all(|w| w[0].to == Some(w[1].from)),
        "contíguas"
    );
    assert_eq!(s.last().unwrap().to, None);
    for st in &s {
        assert!(!st.stage.label().starts_with("nucleo-"));
    }
}

#[test]
fn age_counts_completed_years() {
    let d = |s: &str| crate::time::parse_date(s).unwrap();
    assert_eq!(age_on(d("1985-10-21"), d("2026-10-20")), Some(40));
    assert_eq!(age_on(d("1985-10-20"), d("2026-10-20")), Some(41));
    assert_eq!(age_on(d("2026-10-21"), d("2026-10-20")), None);
}

// ---------------------------------------------------------------------------
// Pela API

struct Vault {
    _dir: tempfile::TempDir,
    api: Api,
    ana: String,
    bruno: String,
}

fn vault() -> Vault {
    let dir = tempfile::tempdir().unwrap();
    let api = Api::new_unlocked_process(TreasuryPaths::with_root(dir.path().join(".t")).unwrap())
        .unwrap();
    api.call(
        "vault.create",
        json!({"password": "senha-mestra-forte", "household_name": "F", "members": ["Ana", "Bruno"]}),
    )
    .unwrap();
    let members = api.call("household.list_members", json!({})).unwrap();
    let id = |n: &str| {
        members
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["name"] == n)
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let (ana, bruno) = (id("Ana"), id("Bruno"));
    Vault {
        _dir: dir,
        api,
        ana,
        bruno,
    }
}

fn set_profile(api: &Api, id: &str, profile: Value) {
    api.call(
        "household.update_member",
        json!({"id": id, "profile": profile}),
    )
    .unwrap();
}

/// Ana titular (1985, mulher) e Bruno cônjuge (1982, homem).
fn couple_vault() -> Vault {
    let v = vault();
    set_profile(
        &v.api,
        &v.ana,
        json!({"birth_date": "1985-03-10", "relationship": "titular", "sex": "female"}),
    );
    set_profile(
        &v.api,
        &v.bruno,
        json!({"birth_date": "1982-07-01", "relationship": "conjuge", "sex": "male"}),
    );
    v
}

fn mcp() -> Actor {
    Actor::Mcp {
        client: "agente".into(),
        read_only: false,
    }
}

#[test]
fn member_sex_is_saved_and_undone_with_the_profile() {
    let v = vault();
    set_profile(
        &v.api,
        &v.ana,
        json!({"birth_date": "1990-01-01", "sex": "female"}),
    );
    let m = v.api.call("household.list_members", json!({})).unwrap();
    let ana = m
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == v.ana.as_str())
        .unwrap();
    assert_eq!(ana["profile"]["sex"], "female");
    assert!(m[1]["profile"]["sex"].is_null() || m[0]["profile"]["sex"].is_null());
    set_profile(
        &v.api,
        &v.ana,
        json!({"birth_date": "1990-01-01", "sex": "unspecified"}),
    );
    let last = v.api.call("changes.list", json!({"limit": 1})).unwrap();
    v.api
        .call("changes.undo", json!({"id": last[0]["id"]}))
        .unwrap();
    let m = v.api.call("household.list_members", json!({})).unwrap();
    let ana = m
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == v.ana.as_str())
        .unwrap();
    assert_eq!(ana["profile"]["sex"], "female", "desfazer volta o sexo");
    assert!(
        v.api
            .call(
                "household.update_member",
                json!({"id": v.ana, "profile": {"sex": "x"}})
            )
            .is_err()
    );
}

#[test]
fn settings_default_off_and_save() {
    let v = vault();
    let s = v.api.call("longevity.settings", json!({})).unwrap();
    assert_eq!(
        s,
        json!({"enabled": false, "hidden_members": [], "show_children_end": false,
               "show_advanced_later_life": false})
    );
    let s = v
        .api
        .call(
            "longevity.set_settings",
            json!({"enabled": true, "hidden_members": [v.bruno, v.bruno], "show_children_end": true,
                   "show_advanced_later_life": false}),
        )
        .unwrap();
    assert_eq!(s["enabled"], true);
    assert_eq!(s["hidden_members"], json!([v.bruno]), "sem repetição");
    assert_eq!(v.api.call("longevity.settings", json!({})).unwrap(), s);
    // Membro desconhecido é recusado; o agente não muda os ajustes.
    assert!(
        v.api
            .call(
                "longevity.set_settings",
                json!({"enabled": true, "hidden_members": ["x"]})
            )
            .is_err()
    );
    let e = v
        .api
        .call_as(&mcp(), "longevity.set_settings", json!({"enabled": false}))
        .unwrap_err();
    assert!(matches!(e, crate::Error::Forbidden(_)), "{e}");
    assert_eq!(
        v.api.call("longevity.stages", json!({})).unwrap()[4]["stage"],
        "later_life"
    );
}

#[test]
fn questionnaire_yes_turns_it_on_only_for_the_family() {
    let v = vault();
    let q = v.api.call("profile.questions", json!({})).unwrap();
    assert_eq!(q.as_array().unwrap().len(), 8);
    let last = &q[7];
    assert_eq!(last["id"], "longevity");
    let opts: Vec<&str> = last["options"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["id"].as_str().unwrap())
        .collect();
    assert_eq!(opts, ["yes", "no", "later"]);
    let q = v
        .api
        .call("profile.questions", json!({"member_id": v.bruno}))
        .unwrap();
    assert_eq!(
        q.as_array().unwrap().len(),
        7,
        "o questionário do membro não tem a pergunta"
    );
    // "Depois" e "Não" deixam desligada; sem pontos (não muda o nível).
    let p = v
        .api
        .call(
            "profile.answer",
            json!({"answers": {"longevity": ["later"], "confianca": ["muito"]}}),
        )
        .unwrap();
    assert_eq!(p["level"], "avancado");
    assert_eq!(
        v.api.call("longevity.settings", json!({})).unwrap()["enabled"],
        false
    );
    // O membro respondendo "Sim" não liga (a chave é da família).
    v.api
        .call(
            "profile.answer",
            json!({"member_id": v.bruno, "answers": {"longevity": ["yes"]}}),
        )
        .unwrap();
    assert_eq!(
        v.api.call("longevity.settings", json!({})).unwrap()["enabled"],
        false
    );
    v.api
        .call("profile.answer", json!({"answers": {"longevity": ["yes"]}}))
        .unwrap();
    assert_eq!(
        v.api.call("longevity.settings", json!({})).unwrap()["enabled"],
        true
    );
    assert!(
        v.api
            .call(
                "profile.answer",
                json!({"answers": {"longevity": ["talvez"]}})
            )
            .is_err()
    );
}

#[test]
fn family_outlook_off_is_empty_and_on_has_members_couple_and_missing() {
    let v = couple_vault();
    let off = v.api.call("longevity.family", json!({})).unwrap();
    assert_eq!(off["enabled"], false);
    assert_eq!(off["members"], json!([]));
    assert!(off["any_adult_alive"].is_null());
    assert_eq!(off["country"], "BR");
    assert!(
        off["source"]["attribution"]
            .as_str()
            .unwrap()
            .contains("World Population Prospects 2024")
    );
    // Uma filha sem sexo e um membro sem data de nascimento.
    let caio = v
        .api
        .call(
            "household.create_member",
            json!({"name": "Clara", "profile": {"birth_date": "2018-05-05", "relationship": "filho"}}),
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    v.api
        .call("household.create_member", json!({"name": "Dora"}))
        .unwrap();
    v.api
        .call("longevity.set_settings", json!({"enabled": true}))
        .unwrap();
    let f = v.api.call("longevity.family", json!({})).unwrap();
    assert_eq!(f["enabled"], true);
    let names: Vec<&str> = f["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Ana", "Bruno", "Clara"]);
    let missing = f["missing"].as_array().unwrap();
    assert_eq!(missing.len(), 2, "{missing:?}");
    assert_eq!(missing[0]["missing"], json!(["sex"]));
    assert_eq!(missing[1]["missing"], json!(["birth_date", "sex"]));
    let clara = &f["members"][2];
    assert_eq!(
        (clara["child"].clone(), clara["show_end"].clone()),
        (json!(true), json!(false))
    );
    assert_eq!(clara["stage"], "childhood");
    let ana = &f["members"][0];
    assert_eq!(ana["sex"], "female");
    assert_eq!(ana["stage"], "adulthood");
    let num = |v: &Value, k: &str| v[k].as_f64().unwrap();
    assert!(num(ana, "median_age") > num(ana, "age"));
    assert!(
        num(ana, "median_age") < num(ana, "p25_age") && num(ana, "p25_age") < num(ana, "p10_age")
    );
    assert_eq!(ana["curve"][0]["survival"], 1.0);
    assert_eq!(ana["curve"].as_array().unwrap().last().unwrap()["age"], 110);
    assert!(ana["cohort_e0"].as_f64().unwrap() > 60.0);
    assert_eq!(ana["period_ref"]["source"], "IBGE 2024");
    assert!((ana["period_ref"]["e0"].as_f64().unwrap() - 79.9).abs() < 0.05);
    // Mulher de 1985 vive mais, pela média, que homem de 1982.
    assert!(num(ana, "median_age") > num(&f["members"][1], "median_age"));
    // Casal: a chance de pelo menos um vivo começa em 1, cai, e nunca fica abaixo de cada um.
    let pts = f["any_adult_alive"].as_array().unwrap();
    assert_eq!(pts[0]["probability"], 1.0);
    assert!(
        pts.windows(2)
            .all(|w| w[1]["probability"].as_f64() <= w[0]["probability"].as_f64())
    );
    let y25 = f["any_alive_p25_year"].as_i64().unwrap();
    let y10 = f["any_alive_p10_year"].as_i64().unwrap();
    assert!(y25 < y10);
    let ana_born = 1985;
    assert!((y10 - ana_born) as f64 >= num(ana, "p10_age") - 1.0);
    // Com o ajuste, a criança mostra o final; escondido, o membro sai da lista.
    v.api
        .call(
            "longevity.set_settings",
            json!({"enabled": true, "show_children_end": true, "hidden_members": [v.bruno]}),
        )
        .unwrap();
    let f = v.api.call("longevity.family", json!({})).unwrap();
    let names: Vec<&str> = f["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Ana", "Clara"]);
    assert_eq!(f["members"][1]["show_end"], true);
    // O membro sozinho: o mesmo resultado; sem data de nascimento, erro claro.
    let o = v
        .api
        .call("longevity.outlook", json!({"member_id": caio}))
        .unwrap();
    assert_eq!(o["name"], "Clara");
    let dora = f["missing"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["name"] == "Dora")
        .unwrap()["member_id"]
        .clone();
    let e = v
        .api
        .call("longevity.outlook", json!({"member_id": dora}))
        .unwrap_err();
    assert_eq!(e.code(), "invalid");
    assert!(e.message().contains("Dora"), "{}", e.message());
}

#[test]
fn retirement_defaults_and_the_simulation_block() {
    let v = vault();
    set_profile(
        &v.api,
        &v.ana,
        json!({"birth_date": "1980-01-15", "relationship": "titular", "sex": "female"}),
    );
    let today = crate::time::today_local();
    let age = age_on(crate::time::parse_date("1980-01-15").unwrap(), today).unwrap();
    // Desligada: 95, com a idade de hoje.
    let d = v.api.call("goals.retirement_defaults", json!({})).unwrap();
    assert_eq!(
        d,
        json!({"current_age": age, "life_expectancy": 95, "from_longevity": false})
    );
    let sim = v
        .api
        .call(
            "goals.retirement",
            json!({"current_age": age, "retirement_age": 65, "life_expectancy": 95,
                   "monthly_spending": 500_000, "simulations": 100}),
        )
        .unwrap();
    assert!(sim["longevity"].is_null());
    // Sem a idade no pedido (como o agente pede, ou o cofre de demonstração): a do membro.
    let sim = v
        .api
        .call(
            "goals.retirement",
            json!({"monthly_spending": 500_000, "simulations": 100}),
        )
        .unwrap();
    assert_eq!(sim["plan"]["current_age"], age, "{sim}");
    let inss = v
        .api
        .call(
            "goals.inss_estimate",
            json!({"input": {"sex": "female", "contribution_years": 20, "average_salary": 500_000}}),
        )
        .unwrap();
    assert!(inss.is_object(), "{inss}");
    // Ligada: o "1 em 10" da Ana, arredondado para cima.
    v.api
        .call("longevity.set_settings", json!({"enabled": true}))
        .unwrap();
    let o = v
        .api
        .call("longevity.outlook", json!({"member_id": v.ana}))
        .unwrap();
    let p10 = o["p10_age"].as_f64().unwrap();
    let single = v.api.call("goals.retirement_defaults", json!({})).unwrap();
    assert_eq!(single["from_longevity"], true);
    let s = single["life_expectancy"].as_f64().unwrap();
    assert!(s >= p10 - 0.05 && s <= p10.ceil() + 1.0, "{s} vs {p10}");
    assert_eq!(s, s.trunc());
    // Com cônjuge, pela chance de pelo menos um vivo: nunca menos.
    set_profile(
        &v.api,
        &v.bruno,
        json!({"birth_date": "1978-06-01", "relationship": "conjuge", "sex": "male"}),
    );
    let both = v.api.call("goals.retirement_defaults", json!({})).unwrap();
    assert!(both["life_expectancy"].as_u64() >= single["life_expectancy"].as_u64());
    // Do ponto de vista do cônjuge (idades dele).
    let his = v
        .api
        .call("goals.retirement_defaults", json!({"member_id": v.bruno}))
        .unwrap();
    assert_eq!(his["from_longevity"], true);
    assert!(his["current_age"].as_u64().unwrap() > age as u64);
    // O resultado da simulação traz o risco de longevidade do plano.
    let sim = v
        .api
        .call(
            "goals.retirement",
            json!({"current_age": age, "retirement_age": 65, "life_expectancy": 95,
                   "monthly_spending": 500_000, "simulations": 100}),
        )
        .unwrap();
    let l = &sim["longevity"];
    assert_eq!(l["plan_age"], 95);
    let survival = l["survival"].as_f64().unwrap();
    assert!(survival > 0.0 && survival < 1.0, "{l}");
    assert_eq!(l["suggested_age"], both["life_expectancy"]);
    // Sem data de nascimento: 95 e nada no resultado.
    set_profile(&v.api, &v.ana, json!({"relationship": "titular"}));
    let d = v.api.call("goals.retirement_defaults", json!({})).unwrap();
    assert_eq!(
        d,
        json!({"current_age": null, "life_expectancy": 95, "from_longevity": false})
    );
}

#[test]
fn milestones_come_from_the_plan_goals_and_debts() {
    let v = vault();
    set_profile(
        &v.api,
        &v.ana,
        json!({"birth_date": "1980-01-15", "relationship": "titular", "sex": "female"}),
    );
    set_profile(
        &v.api,
        &v.bruno,
        json!({"birth_date": "1979-02-01", "relationship": "conjuge"}),
    );
    let today = crate::time::today_local();
    let age = age_on(crate::time::parse_date("1980-01-15").unwrap(), today).unwrap();
    v.api
        .call(
            "goals.retirement_save",
            json!({"current_age": age, "retirement_age": 62, "life_expectancy": 92,
                   "monthly_spending": 500_000, "inss_monthly": 300_000, "inss_age": 65,
                   "simulations": 100,
                   "events": [{"age": 70, "amount": -2_000_000, "label": "Reforma da casa"}]}),
        )
        .unwrap();
    let in_years = |n: i32| crate::time::fmt_date(crate::time::add_months(today, 12 * n));
    v.api
        .call(
            "goals.create",
            json!({"name": "Viagem", "kind": "travel", "target_amount": 1_500_000,
                   "target_date": in_years(3)}),
        )
        .unwrap();
    let acc = v
        .api
        .call(
            "accounts.create",
            json!({"name": "Conta", "kind": "checking", "member_ids": [v.bruno]}),
        )
        .unwrap()["id"]
        .clone();
    let first = crate::time::add_months(today, 1);
    v.api
        .call(
            "debt.create",
            json!({"name": "Carro", "kind": "vehicle", "principal": 6_000_000, "annual_rate": "12",
                   "system": "price", "contract_date": crate::time::fmt_date(today),
                   "first_due_date": crate::time::fmt_date(first), "term_months": 48,
                   "debit_account_id": acc, "member_id": v.bruno}),
        )
        .unwrap();
    v.api
        .call("longevity.set_settings", json!({"enabled": true}))
        .unwrap();
    let o = v
        .api
        .call("longevity.outlook", json!({"member_id": v.ana}))
        .unwrap();
    let kinds: Vec<(&str, i64)> = o["milestones"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| (m["kind"].as_str().unwrap(), m["age"].as_i64().unwrap()))
        .collect();
    assert!(kinds.contains(&("retirement", 62)), "{kinds:?}");
    assert!(kinds.contains(&("inss", 65)), "{kinds:?}");
    assert!(kinds.contains(&("plan_end", 92)), "{kinds:?}");
    assert!(kinds.contains(&("life_event", 70)), "{kinds:?}");
    assert!(kinds.iter().any(|(k, _)| *k == "goal"), "{kinds:?}");
    assert!(
        !kinds.iter().any(|(k, _)| *k == "debt_end"),
        "a dívida é do Bruno"
    );
    let ret = o["milestones"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["kind"] == "retirement")
        .unwrap();
    assert_eq!(ret["year"], 1980 + 62);
    assert_eq!(ret["label"], "Aposentadoria");
    let inss = o["milestones"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["kind"] == "inss")
        .unwrap();
    assert_eq!(inss["amount"], 300_000);
    let b = v
        .api
        .call("longevity.outlook", json!({"member_id": v.bruno}))
        .unwrap();
    let debt = b["milestones"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["kind"] == "debt_end")
        .expect("fim do financiamento do Bruno");
    assert_eq!(debt["label"], "Fim de Carro");
    assert!(debt["amount"].as_i64().unwrap() > 0);
    assert!(
        !b["milestones"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["kind"] == "retirement")
    );
}

#[test]
fn mcp_tools_exist_only_with_the_feature_on() {
    use crate::mcp::inproc;
    let v = couple_vault();
    let names = |api: &Api| -> Vec<String> {
        inproc::tool_defs(inproc::agent_access(api))
            .into_iter()
            .map(|t| t.name)
            .collect()
    };
    let off = names(&v.api);
    assert!(
        !off.iter()
            .any(|n| n == "get_family_outlook" || n == "get_member_outlook")
    );
    let o = inproc::call_tool(
        &v.api,
        &mcp(),
        "get_family_outlook",
        json!({}),
        inproc::agent_access(&v.api),
    );
    assert!(o.is_error, "{}", o.content);
    // O comando também recusa o agente com a função desligada.
    let e = v
        .api
        .call_as(&mcp(), "longevity.family", json!({}))
        .unwrap_err();
    assert!(matches!(e, crate::Error::Forbidden(_)), "{e}");

    v.api
        .call(
            "longevity.set_settings",
            json!({"enabled": true, "hidden_members": [v.bruno]}),
        )
        .unwrap();
    let on = names(&v.api);
    assert!(on.iter().any(|n| n == "get_family_outlook"));
    assert!(on.iter().any(|n| n == "get_member_outlook"));
    let o = inproc::call_tool(
        &v.api,
        &mcp(),
        "get_member_outlook",
        json!({"member_id": v.ana}),
        inproc::agent_access(&v.api),
    );
    assert!(!o.is_error, "{}", o.content);
    assert!(
        o.content.contains("Ana") && o.content.contains("1 em 10"),
        "{}",
        o.content
    );
    let o = inproc::call_tool(
        &v.api,
        &mcp(),
        "get_family_outlook",
        json!({}),
        inproc::agent_access(&v.api),
    );
    assert!(!o.is_error, "{}", o.content);
    assert!(o.content.contains("1 membro"), "{}", o.content);
    // Quem escondeu a própria barra fica fora também para o agente.
    let o = inproc::call_tool(
        &v.api,
        &mcp(),
        "get_member_outlook",
        json!({"member_id": v.bruno}),
        inproc::agent_access(&v.api),
    );
    assert!(o.is_error, "{}", o.content);
}

#[test]
fn summaries_and_texts_follow_the_locale() {
    use crate::i18n::{Locale, set_current};
    let t = data::embedded().unwrap();
    for l in [Locale::PtBr, Locale::En, Locale::Es] {
        set_current(l);
        let s = source_of(t, "BR");
        assert!(s.attribution.starts_with(
            "United Nations, Department of Economic and Social Affairs, Population Division (2024). \
             World Population Prospects 2024, Online Edition."
        ));
        assert!(s.attribution.contains("IBGE"));
        assert!(!source_of(t, "PT").attribution.contains("IBGE"));
        assert!(!s.name.starts_with("nucleo-"));
    }
    set_current(Locale::En);
    assert_eq!(dec1(Locale::En, 84.25), "84.2");
    assert_eq!(LifeStage::LaterLife.label(), "Later life");
    set_current(Locale::PtBr);
    assert_eq!(dec1(Locale::PtBr, 84.0), "84,0");
}

#[test]
fn outlook_on_a_fixed_day_is_deterministic() {
    let v = couple_vault();
    let conn_out = |api: &Api| {
        api.with_session(|s, _| {
            outlook_on(
                s.db.conn(),
                data::embedded()?,
                &v.ana,
                NaiveDate::from_ymd_opt(2026, 10, 20).unwrap(),
            )
        })
        .unwrap()
    };
    let a = conn_out(&v.api);
    let b = conn_out(&v.api);
    assert_eq!(a, b);
    assert_eq!(a.age, 41);
    assert_eq!(a.birth_year, 1985);
    assert_eq!(a.curve.len(), (110 - 41 + 1) as usize);
}
