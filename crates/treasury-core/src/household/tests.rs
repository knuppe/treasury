use super::*;
use crate::transactions::testkit::Fx;

fn member_input(name: &str, primary: Option<bool>) -> MemberInput {
    MemberInput {
        name: name.into(),
        color: Some("#ff8800".into()),
        is_primary: primary,
        profile: None,
    }
}

fn patch(id: &str) -> MemberPatch {
    MemberPatch {
        id: id.into(),
        name: None,
        color: None,
        is_primary: None,
        archived: None,
        sort_order: None,
        profile: None,
    }
}

#[test]
fn updates_household() {
    let mut fx = Fx::new();
    let h = fx
        .write(|tx| {
            update(
                tx,
                &HouseholdPatch {
                    name: Some("  Casa Knuppe ".into()),
                    base_currency: Some("usd".into()),
                },
            )
        })
        .unwrap();
    assert_eq!(h.name, "Casa Knuppe");
    assert_eq!(h.base_currency, "USD");
    assert!(
        fx.write(|tx| update(
            tx,
            &HouseholdPatch {
                name: None,
                base_currency: Some("REAL".into())
            }
        ))
        .is_err()
    );
    assert!(
        fx.write(|tx| update(
            tx,
            &HouseholdPatch {
                name: Some(" ".into()),
                base_currency: None
            }
        ))
        .is_err()
    );
    // Patch without changes does not produce a change set.
    let (_, cs) = fx
        .write_cs(|tx| {
            update(
                tx,
                &HouseholdPatch {
                    name: None,
                    base_currency: None,
                },
            )
        })
        .unwrap();
    assert!(cs.is_none());
}

#[test]
fn members_primary_archive_and_uniqueness() {
    let mut fx = Fx::new();
    let carlos = fx.member("Carlos");
    let ana = fx.member("Ana");
    assert!(get_member(fx.conn(), &carlos).unwrap().is_primary);

    let bia = fx
        .write(|tx| create_member(tx, &member_input("Bia", None)))
        .unwrap();
    assert!(!bia.is_primary, "there is already a primary");
    assert_eq!(bia.sort_order, 2);
    assert!(matches!(
        fx.write(|tx| create_member(tx, &member_input("bia", None))),
        Err(Error::Conflict(_))
    ));

    // Making Ana primary takes the post from Carlos.
    let mut p = patch(&ana);
    p.is_primary = Some(true);
    fx.write(|tx| update_member(tx, &p)).unwrap();
    assert!(!get_member(fx.conn(), &carlos).unwrap().is_primary);
    assert!(get_member(fx.conn(), &ana).unwrap().is_primary);

    // The primary cannot be archived.
    let mut p = patch(&ana);
    p.archived = Some(true);
    assert!(fx.write(|tx| update_member(tx, &p)).is_err());

    let mut p = patch(&bia.id);
    p.archived = Some(true);
    p.color = Some(None);
    let b = fx.write(|tx| update_member(tx, &p)).unwrap();
    assert!(b.archived);
    assert_eq!(b.color, None);
    assert_eq!(list_members(fx.conn(), false).unwrap().len(), 2);
    let all = list_members(fx.conn(), true).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all.last().unwrap().name, "Bia", "archived last");

    // Renaming to an existing name conflicts.
    let mut p = patch(&bia.id);
    p.name = Some("Carlos".into());
    assert!(matches!(
        fx.write(|tx| update_member(tx, &p)),
        Err(Error::Conflict(_))
    ));
}

#[test]
fn institutions_crud_and_delete_only_when_unused() {
    let mut fx = Fx::new();
    let inst = fx
        .write(|tx| {
            create_institution(
                tx,
                &InstitutionInput {
                    name: "Nubank".into(),
                    kind: None,
                    code: Some(" 260 ".into()),
                    website: Some("https://nubank.com.br".into()),
                },
            )
        })
        .unwrap();
    assert_eq!(inst.kind, InstitutionKind::Bank);
    assert_eq!(inst.code.as_deref(), Some("260"));
    let upd = fx
        .write(|tx| {
            update_institution(
                tx,
                &InstitutionPatch {
                    id: inst.id.clone(),
                    name: Some("Nu Pagamentos".into()),
                    kind: Some(InstitutionKind::Issuer),
                    code: Some(None),
                    website: None,
                },
            )
        })
        .unwrap();
    assert_eq!(upd.name, "Nu Pagamentos");
    assert_eq!(upd.code, None);
    assert_eq!(upd.website.as_deref(), Some("https://nubank.com.br"));
    assert_eq!(list_institutions(fx.conn()).unwrap().len(), 1);

    let acc = fx.checking("Conta", 0);
    fx.conn()
        .execute(
            "UPDATE account SET institution_id = ?1 WHERE id = ?2",
            [&inst.id, &acc],
        )
        .unwrap();
    assert!(matches!(
        fx.write(|tx| delete_institution(tx, &inst.id)),
        Err(Error::Conflict(_))
    ));
    fx.conn()
        .execute("UPDATE account SET institution_id = NULL", [])
        .unwrap();
    fx.write(|tx| delete_institution(tx, &inst.id)).unwrap();
    assert!(list_institutions(fx.conn()).unwrap().is_empty());
    assert!(matches!(
        fx.write(|tx| delete_institution(tx, &inst.id)),
        Err(Error::NotFound(_))
    ));
}
