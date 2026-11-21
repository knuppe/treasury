//! Categorias padrão pelo idioma (docs/i18n-design.md › Dados semeados): chaves do catálogo,
//! nomes por idioma e país, chave de origem nos cofres antigos, tradução e sugestões.

use super::*;
use crate::changes::{Author, record};
use crate::i18n::{Locale, set_current};
use crate::storage::Db;
use crate::vault::KDF_TEST;

/// Cofre com o conjunto inicial do país no idioma atual.
fn vault(country: &str) -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
    record(db.conn_mut(), &Author::system("t"), "seed", |tx| {
        crate::household::seed_in(tx, "Casa", "BRL", country, &Vec::<String>::new())?;
        seed_defaults_for(tx, country)
    })
    .unwrap();
    (dir, db)
}

fn names(db: &Db) -> Vec<String> {
    let mut stmt = db.conn().prepare("SELECT name FROM category").unwrap();
    stmt.query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

fn count(db: &Db, sql: &str) -> i64 {
    db.conn().query_row(sql, [], |r| r.get(0)).unwrap()
}

fn id_of(db: &Db, name: &str) -> String {
    db.conn()
        .query_row("SELECT id FROM category WHERE name = ?1", [name], |r| {
            r.get(0)
        })
        .unwrap()
}

#[test]
fn every_seed_has_a_unique_key_in_every_catalog() {
    let keys: Vec<String> = all_seed_keys().collect();
    let unique: HashSet<&String> = keys.iter().collect();
    assert_eq!(keys.len(), unique.len(), "chaves repetidas");
    for l in [Locale::PtBr, Locale::En, Locale::Es] {
        for k in &keys {
            assert!(i18n::has_key(l, k), "{} sem {k}", l.code());
        }
    }
    // O pt-BR do nome-base é o nome da tabela (o que o app sempre criou no Brasil).
    for (_, seeds) in TABLES {
        for (name, _, children) in seeds {
            assert_eq!(i18n::tr(Locale::PtBr, &seed_key(None, name), None), *name);
            for c in *children {
                assert_eq!(i18n::tr(Locale::PtBr, &seed_key(Some(name), c), None), *c);
            }
        }
    }
    // Nenhuma chave do catálogo sobrando: cada uma é uma semente ou a variante de uma.
    let known: HashSet<String> = keys
        .iter()
        .flat_map(|k| ALL_VARIANTS.iter().map(move |v| format!("{k}{v}")))
        .collect();
    for l in i18n::AVAILABLE {
        for k in i18n::catalog_keys(l)
            .iter()
            .filter(|k| k.starts_with(KEY_PREFIX))
        {
            assert!(
                known.contains(k),
                "{}: {k} não é de nenhuma semente",
                l.code()
            );
        }
    }
    assert_eq!(seed_key(None, "13º salário"), "categorias-13o-salario");
    assert_eq!(
        seed_key(Some("Filhos"), "Mesada"),
        "categorias-filhos-mesada"
    );
    assert_eq!(seed_key(None, "BPC/LOAS"), "categorias-bpc-loas");
}

#[test]
fn names_in_pt_br_are_the_ones_of_each_country() {
    let iptu = seed_key(Some("Moradia"), "IPTU");
    for (country, want) in [
        ("BR", "IPTU"),
        ("US", "Imposto predial (property tax)"),
        ("PT", "IMI"),
        ("MX", "Imposto sobre imóveis"),
    ] {
        assert_eq!(seed_name(Locale::PtBr, country, &iptu), want);
    }
    let mei = seed_key(Some("Vendas e negócio próprio"), "Faturamento do MEI");
    // Os EUA não têm a sua: caem na dos outros países.
    assert_eq!(
        seed_name(Locale::PtBr, "US", &mei),
        "Faturamento do negócio"
    );
    assert_eq!(seed_name(Locale::PtBr, "PT", &mei), "Faturação do negócio");
    // pt-PT: as palavras de Portugal, e o resto pela reserva em pt-BR.
    let rent = seed_key(Some("Moradia"), "Aluguel");
    assert_eq!(seed_name(Locale::PtPt, "BR", &rent), "Renda da casa");
    assert_eq!(seed_name(Locale::PtPt, "PT", &iptu), "IMI");
    assert_eq!(seed_name(Locale::PtPt, "BR", &iptu), "IPTU");
    // Inglês e espanhol, com a variante do país.
    assert_eq!(seed_name(Locale::En, "US", &iptu), "Property tax");
    assert_eq!(seed_name(Locale::En, "BR", &iptu), "Property tax (IPTU)");
    assert_eq!(
        seed_name(Locale::Es, "PT", &rent),
        "Alquiler de la vivienda"
    );
}

#[test]
fn seeds_follow_the_language_and_store_the_key() {
    set_current(Locale::En);
    let (_d, us) = vault("US");
    let n = names(&us);
    for want in [
        "Housing",
        "HOA fees",
        "Property tax",
        "401(k) and IRA",
        "Groceries",
    ] {
        assert!(n.iter().any(|x| x == want), "{want}: {n:?}");
    }
    assert!(!n.iter().any(|x| x == "Mercado" || x == "IOF"));
    assert_eq!(
        count(&us, "SELECT count(*) FROM category WHERE seed_key IS NULL"),
        0
    );
    let key: String = us
        .conn()
        .query_row(
            "SELECT seed_key FROM category WHERE name = 'Groceries'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(key, "categorias-mercado");
    set_current(Locale::Es);
    let (_d, br) = vault("BR");
    let n = names(&br);
    for want in [
        "Vivienda",
        "Supermercado",
        "IPTU (impuesto predial)",
        "13º salario",
    ] {
        assert!(n.iter().any(|x| x == want), "{want}: {n:?}");
    }
    // Mesmo conjunto do pt-BR (só os nomes mudam).
    set_current(Locale::PtBr);
    let (_d, pt_br) = vault("BR");
    assert_eq!(names(&pt_br).len(), n.len());
    assert!(missing_defaults(pt_br.conn()).unwrap().is_empty());
}

#[test]
fn old_vaults_get_the_key_once_and_renamed_ones_too_when_in_place() {
    let (_d, mut db) = vault("BR");
    let total = count(&db, "SELECT count(*) FROM category");
    // Cofre de antes da V127: sem chave, um nome com outra grafia, um renomeado e um da pessoa.
    record(db.conn_mut(), &Author::system("t"), "x", |tx| {
        tx.execute("UPDATE category SET seed_key = NULL", [])?;
        tx.execute(
            "UPDATE category SET name = 'mercado' WHERE name = 'Mercado'",
            [],
        )?;
        tx.execute(
            "UPDATE category SET name = 'Padoca' WHERE name = 'Padaria'",
            [],
        )?;
        let alim: String = tx.query_row(
            "SELECT id FROM category WHERE name = 'Alimentação'",
            [],
            |r| r.get(0),
        )?;
        tx.execute(
            "INSERT INTO category (id, parent_id, name, kind, created_at, updated_at)
             VALUES ('feira', ?1, 'Feira', 'expense', 'a', 'a')",
            [alim],
        )?;
        Ok(())
    })
    .unwrap();
    let n = backfill_seed_keys(db.conn()).unwrap();
    assert_eq!(
        n as i64,
        total - 1,
        "todas menos a renomeada (a da pessoa não conta)"
    );
    assert_eq!(
        count(&db, "SELECT count(*) FROM category WHERE seed_key IS NULL"),
        2
    );
    let key: Option<String> = db
        .conn()
        .query_row(
            "SELECT seed_key FROM category WHERE name = 'mercado'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(key.as_deref(), Some("categorias-mercado"));
    // Uma vez só (e, se rodasse de novo, só olharia as sem chave).
    assert_eq!(backfill_seed_keys(db.conn()).unwrap(), 0);
    db.conn()
        .execute("DELETE FROM setting WHERE key = ?1", [BACKFILL_SETTING])
        .unwrap();
    assert_eq!(backfill_seed_keys(db.conn()).unwrap(), 0);
}

#[test]
fn undo_still_works_after_the_key_is_filled_outside_the_history() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = Db::create(&dir.path().join("t.db"), "s", KDF_TEST).unwrap();
    // Como um cofre antigo: as categorias entraram sem chave.
    let cs = record(db.conn_mut(), &Author::user(), "seed", |tx| {
        seed_defaults(tx)?;
        tx.execute("UPDATE category SET seed_key = NULL", [])?;
        Ok(())
    })
    .unwrap()
    .change_set_id
    .unwrap();
    assert!(backfill_seed_keys(db.conn()).unwrap() > 60);
    crate::changes::undo(db.conn_mut(), &cs).unwrap();
    assert_eq!(count(&db, "SELECT count(*) FROM category"), 0);
}

#[test]
fn translating_renames_only_default_names() {
    let (_d, mut db) = vault("BR");
    assert_eq!(untranslated_defaults(db.conn()).unwrap(), 0);
    record(db.conn_mut(), &Author::user(), "x", |tx| {
        tx.execute(
            "UPDATE category SET name = 'Padoca' WHERE name = 'Padaria'",
            [],
        )?;
        // Uma da pessoa com o nome que "Delivery" teria em inglês: essa fica.
        let alim: String = tx.query_row(
            "SELECT id FROM category WHERE name = 'Alimentação'",
            [],
            |r| r.get(0),
        )?;
        tx.execute(
            "INSERT INTO category (id, parent_id, name, kind, created_at, updated_at)
             VALUES ('fd', ?1, 'Food delivery', 'expense', 'a', 'a')",
            [alim],
        )?;
        Ok(())
    })
    .unwrap();
    set_current(Locale::En);
    let before = untranslated_defaults(db.conn()).unwrap();
    assert!(before > 100, "{before}");
    let n = record(db.conn_mut(), &Author::user(), "t", translate_defaults)
        .unwrap()
        .value;
    assert_eq!(n, before);
    let all = names(&db);
    for want in [
        "Groceries",
        "Padoca",
        "Delivery",
        "Food delivery",
        "Food",
        "Internet",
    ] {
        assert!(all.iter().any(|x| x == want), "{want}: {all:?}");
    }
    assert_eq!(all.iter().filter(|x| *x == "Food delivery").count(), 1);
    assert_eq!(untranslated_defaults(db.conn()).unwrap(), 0);
    // De volta ao pt-BR: só as que estão com o nome padrão (em inglês) voltam.
    set_current(Locale::PtBr);
    assert_eq!(untranslated_defaults(db.conn()).unwrap(), n);
    record(db.conn_mut(), &Author::user(), "t", translate_defaults).unwrap();
    let all = names(&db);
    assert!(all.iter().any(|x| x == "Mercado") && !all.iter().any(|x| x == "Groceries"));
    assert!(all.iter().any(|x| x == "Padoca"));
    // Trocar de país não renomeia: os nomes do Brasil já são do idioma atual.
    db.conn()
        .execute("UPDATE household SET country = 'US'", [])
        .unwrap();
    assert_eq!(untranslated_defaults(db.conn()).unwrap(), 0);
    set_current(Locale::En);
    record(db.conn_mut(), &Author::user(), "t", translate_defaults).unwrap();
    let all = names(&db);
    assert!(all.iter().any(|x| x == "Property tax") && all.iter().any(|x| x == "HOA fees"));
    set_current(Locale::PtBr);
    // A renomeada continua padrão para as buscas do núcleo (pela chave).
    let padoca = id_of(&db, "Padoca");
    assert_eq!(
        find_br_child(db.conn(), CategoryKind::Expense, "Alimentação", "Padaria").unwrap(),
        Some(padoca)
    );
}

#[test]
fn suggestions_compare_by_key_in_any_language() {
    let (_d, mut db) = vault("BR");
    set_current(Locale::En);
    // Nomes em pt-BR e o app em inglês: nada falta.
    assert!(missing_defaults(db.conn()).unwrap().is_empty());
    record(db.conn_mut(), &Author::user(), "x", |tx| {
        tx.execute("DELETE FROM category WHERE name = 'Fraldas e higiene'", [])?;
        // Renomeada: continua sendo a semente (não volta como sugestão).
        tx.execute(
            "UPDATE category SET name = 'Feirinha' WHERE name = 'Mercado'",
            [],
        )?;
        Ok(())
    })
    .unwrap();
    let miss: Vec<String> = missing_defaults(db.conn())
        .unwrap()
        .into_iter()
        .map(|m| m.path)
        .collect();
    assert_eq!(miss, ["Filhos › Diapers and hygiene"]);
    let n = record(db.conn_mut(), &Author::user(), "add", |tx| {
        add_missing_defaults(tx, None)
    })
    .unwrap()
    .value;
    assert_eq!(n, 1);
    let key: String = db
        .conn()
        .query_row(
            "SELECT seed_key FROM category WHERE name = 'Diapers and hygiene'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(key, "categorias-fraldas-e-higiene");
    assert!(missing_defaults(db.conn()).unwrap().is_empty());
    // Cofre antigo (sem chave): pelo nome, em qualquer idioma.
    db.conn()
        .execute("UPDATE category SET seed_key = NULL", [])
        .unwrap();
    let miss: Vec<String> = missing_defaults(db.conn())
        .unwrap()
        .into_iter()
        .map(|m| m.path)
        .collect();
    assert_eq!(
        miss,
        ["Alimentação › Groceries"],
        "a renomeada sem chave volta"
    );
    set_current(Locale::PtBr);
}

#[test]
fn core_lookups_find_defaults_in_any_language() {
    set_current(Locale::Es);
    let (_d, mut db) = vault("BR");
    let delivery = id_of(&db, "Comida a domicilio");
    assert_eq!(
        find_br_child(db.conn(), CategoryKind::Expense, "Alimentação", "Delivery").unwrap(),
        Some(delivery)
    );
    let jcp = id_of(&db, "JCP (intereses sobre el capital propio)");
    assert_eq!(
        find_br_path(db.conn(), Some(CategoryKind::Income), "Rendimentos › JCP").unwrap(),
        Some(jcp.clone())
    );
    assert_eq!(br_names(db.conn()).unwrap()[&jcp], "JCP");
    // Regras prontas: as mesmas com as categorias em espanhol e em pt-BR.
    let es = record(db.conn_mut(), &Author::system("t"), "r", |tx| {
        crate::rules::builtin::seed_for(tx, "BR")
    })
    .unwrap()
    .value;
    set_current(Locale::PtBr);
    let (_d, mut pt) = vault("BR");
    let br = record(pt.conn_mut(), &Author::system("t"), "r", |tx| {
        crate::rules::builtin::seed_for(tx, "BR")
    })
    .unwrap()
    .value;
    assert!(br > 10);
    assert_eq!(es, br);
}

/// Grupo e "essencial" gravados de uma categoria pelo nome (único no conjunto inicial).
fn stored_group(db: &Db, name: &str) -> (Option<String>, bool) {
    db.conn()
        .query_row(
            "SELECT spend_group, is_essential FROM category WHERE name = ?1",
            [name],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
}

#[test]
fn old_vaults_get_the_seed_groups_once_without_undoing_the_persons_choices() {
    let (_d, mut db) = vault("BR");
    // Cofre de antes da V129, já migrado: o "essencial" de antes (dívidas eram essenciais) e o
    // grupo que a migração deu por ele. A pessoa desmarcou "Parcelas", marcou "Lazer" e criou
    // três subcategorias.
    record(db.conn_mut(), &Author::system("t"), "x", |tx| {
        tx.execute(
            "UPDATE category SET is_essential = 1 WHERE name IN
               ('Empréstimos', 'Financiamentos', 'Juros e multas', 'Tarifas bancárias', 'Lazer')",
            [],
        )?;
        tx.execute(
            "UPDATE category SET is_essential = 0 WHERE name = 'Parcelas'",
            [],
        )?;
        for (id, name, parent, essential) in [
            ("cripto", "Cripto", "Poupança e investimentos", 0),
            ("consignado", "Consignado", "Empréstimos", 1),
            ("jogos", "Jogos", "Lazer", 1),
        ] {
            tx.execute(
                "INSERT INTO category (id, parent_id, name, kind, is_essential, created_at, updated_at)
                 VALUES (?1, (SELECT id FROM category WHERE name = ?3), ?2, 'expense', ?4, 'a', 'a')",
                params![id, name, parent, essential],
            )?;
        }
        tx.execute(
            "UPDATE category SET spend_group = CASE WHEN is_essential = 1 THEN 'essential'
                                                     ELSE 'non_essential' END
             WHERE kind = 'expense'",
            [],
        )?;
        Ok(())
    })
    .unwrap();
    db.conn()
        .execute(
            "DELETE FROM setting WHERE key = ?1",
            [GROUPS_BACKFILL_SETTING],
        )
        .unwrap();
    let n = backfill_spend_groups(db.conn()).unwrap();
    // Essenciais e dívidas contam como essencial: as dívidas padrão seguem essenciais.
    let g = |s: &str| (Some(s.to_string()), s == "essential" || s == "debt");
    // As padrão com o "essencial" de antes ganham o grupo da semente.
    assert_eq!(stored_group(&db, "Empréstimos"), g("debt"));
    assert_eq!(stored_group(&db, "Financiamentos"), g("debt"));
    assert_eq!(stored_group(&db, "Juros e multas"), g("debt"));
    assert_eq!(stored_group(&db, "Tarifas bancárias"), g("debt"));
    assert_eq!(stored_group(&db, "Poupança e investimentos"), g("future"));
    assert_eq!(stored_group(&db, "Reserva de emergência"), g("future"));
    // A escolha da pessoa fica: "Parcelas" desmarcada e "Lazer" marcado.
    assert_eq!(stored_group(&db, "Parcelas"), g("non_essential"));
    assert_eq!(stored_group(&db, "Lazer"), g("essential"));
    // Subcategorias da pessoa acompanham o pai que mudou quando estavam no grupo dele.
    assert_eq!(stored_group(&db, "Cripto"), g("future"));
    assert_eq!(stored_group(&db, "Consignado"), g("debt"));
    assert_eq!(stored_group(&db, "Jogos"), g("essential"));
    // O resto não muda; receitas continuam sem grupo.
    assert_eq!(stored_group(&db, "Moradia"), g("essential"));
    assert_eq!(stored_group(&db, "Restaurantes"), g("non_essential"));
    assert_eq!(stored_group(&db, "Salário"), (None, false));
    // Empréstimos, Financiamentos, Juros, Tarifas, Poupança e investimentos (com 4 filhas),
    // Cripto e Consignado.
    assert_eq!(n, 4 + 5 + 2);
    // Uma vez só.
    assert_eq!(backfill_spend_groups(db.conn()).unwrap(), 0);
    // Num cofre novo, nada muda.
    let (_d2, fresh) = vault("BR");
    fresh
        .conn()
        .execute(
            "DELETE FROM setting WHERE key = ?1",
            [GROUPS_BACKFILL_SETTING],
        )
        .unwrap();
    assert_eq!(backfill_spend_groups(fresh.conn()).unwrap(), 0);
}

#[test]
fn brazil_only_seeds_stay_in_brazil() {
    // FGTS só no Brasil; 13º e férias só onde o app conhece os pagamentos extras.
    assert!(exists_in("BR", "Saque do FGTS"));
    for c in ["US", "PT", "ES", "DE"] {
        assert!(!exists_in(c, "Saque do FGTS"), "{c}");
    }
    assert!(exists_in("PT", "13º salário"));
    assert!(!exists_in("DE", "13º salário"));
    assert!(!exists_in("DE", "Férias"));
}
