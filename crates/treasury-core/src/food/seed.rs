//! Categorias de produto da semente (docs/consumption.md, seção 5.2): os 17 grupos do cupom
//! viram categorias e os subitens do IPCA de "alimentação no domicílio" viram subcategorias, com
//! a classe sensata de cada uma (a pessoa muda à vontade). Desde a V136 (docs/consumption.md) as
//! raízes são os segmentos de consumo (Alimentação, Vestuário, Eletrônicos…), e os grupos do
//! cupom ficam dentro deles. Os nomes nascem no idioma da instalação.

use rusqlite::{Connection, params};

use crate::error::Result;
use crate::i18n::t;
use crate::ids::new_id;
use crate::receipts::classify::ItemGroup;
use crate::time::now_utc;

/// Raiz da semente: o grupo do cupom, se é alimento e a classe.
pub struct Root {
    pub group: ItemGroup,
    pub is_food: bool,
    pub class: Option<&'static str>,
}

pub const ROOTS: &[Root] = &[
    Root {
        group: ItemGroup::Hortifruti,
        is_food: true,
        class: Some("essential"),
    },
    Root {
        group: ItemGroup::Carnes,
        is_food: true,
        class: Some("essential"),
    },
    Root {
        group: ItemGroup::Laticinios,
        is_food: true,
        class: Some("essential"),
    },
    Root {
        group: ItemGroup::Mercearia,
        is_food: true,
        class: Some("essential"),
    },
    Root {
        group: ItemGroup::Padaria,
        is_food: true,
        class: Some("essential"),
    },
    Root {
        group: ItemGroup::DocesLanches,
        is_food: true,
        class: Some("superfluous"),
    },
    Root {
        group: ItemGroup::Congelados,
        is_food: true,
        class: Some("complementary"),
    },
    Root {
        group: ItemGroup::Bebidas,
        is_food: true,
        class: Some("complementary"),
    },
    Root {
        group: ItemGroup::BebidasAlcoolicas,
        is_food: true,
        class: Some("superfluous"),
    },
    Root {
        group: ItemGroup::Limpeza,
        is_food: false,
        class: Some("essential"),
    },
    Root {
        group: ItemGroup::Higiene,
        is_food: false,
        class: Some("essential"),
    },
    Root {
        group: ItemGroup::Bebe,
        is_food: false,
        class: Some("essential"),
    },
    Root {
        group: ItemGroup::Pet,
        is_food: false,
        class: Some("complementary"),
    },
    Root {
        group: ItemGroup::Farmacia,
        is_food: false,
        class: Some("essential"),
    },
    Root {
        group: ItemGroup::Casa,
        is_food: false,
        class: Some("complementary"),
    },
    Root {
        group: ItemGroup::Cigarros,
        is_food: false,
        class: Some("superfluous"),
    },
    Root {
        group: ItemGroup::Outros,
        is_food: false,
        class: None,
    },
];

/// Subcategoria da semente: chave, chave da categoria-pai, chave do nome, classe (`None`: a do
/// pai) e as expressões (sobre a descrição sem acentos, em minúsculas) que levam o item para ela.
pub struct Sub {
    pub key: &'static str,
    pub parent: &'static str,
    pub name: &'static str,
    pub class: Option<&'static str>,
    pub pattern: &'static str,
}

pub const SUBS: &[Sub] = &[
    Sub {
        key: "hortifruti.frutas",
        parent: "hortifruti",
        name: "nucleo-alimentacao-cat-frutas",
        class: None,
        pattern: r"\b(banana|maca|laranja|mamao|melao|melancia|uva|abacaxi|manga|pera|limao|morango|abacate|goiaba|tangerina|mexerica|kiwi)",
    },
    Sub {
        key: "hortifruti.legumes",
        parent: "hortifruti",
        name: "nucleo-alimentacao-cat-legumes",
        class: None,
        pattern: r"\b(tomate|cebola|alho|cenoura|batata|abobora|abobrinha|pepino|pimentao|beterraba|chuchu|mandioca|aipim|alface|couve|repolho|brocolis|rucula|espinafre|cheiro verde|salsa|coentro)",
    },
    Sub {
        key: "carnes.bovina",
        parent: "carnes",
        name: "nucleo-alimentacao-cat-carne-bovina",
        class: None,
        pattern: r"\b(patinho|acem|alcatra|contra ?file|maminha|picanha|cupim|fraldinha|musculo|costela bov|carne moida|bov)",
    },
    Sub {
        key: "carnes.aves_ovos",
        parent: "carnes",
        name: "nucleo-alimentacao-cat-aves-ovos",
        class: None,
        pattern: r"\b(frango|sobrecoxa|coxa|peito fgo|file fgo|ovo|ovos|chester|peru)",
    },
    Sub {
        key: "carnes.suinos",
        parent: "carnes",
        name: "nucleo-alimentacao-cat-suinos",
        class: None,
        pattern: r"\b(suin|lombo|pernil|bisteca|toucinho|bacon)",
    },
    Sub {
        key: "carnes.peixes",
        parent: "carnes",
        name: "nucleo-alimentacao-cat-peixes",
        class: None,
        pattern: r"\b(peixe|tilapia|sardinha|atum|salmao|merluza|pescada|camarao)",
    },
    Sub {
        key: "carnes.frios",
        parent: "carnes",
        name: "nucleo-alimentacao-cat-frios",
        class: Some("complementary"),
        pattern: r"\b(presunto|apresuntado|mortadela|salame|peito peru|linguica|salsicha|blanquet)",
    },
    Sub {
        key: "laticinios.leite",
        parent: "laticinios",
        name: "nucleo-alimentacao-cat-leite",
        class: None,
        pattern: r"\b(leite|lte)\b",
    },
    Sub {
        key: "laticinios.queijos",
        parent: "laticinios",
        name: "nucleo-alimentacao-cat-queijos",
        class: Some("complementary"),
        pattern: r"\b(queijo|qjo|mussarela|requeijao|parmesao|ricota|cream cheese)",
    },
    Sub {
        key: "laticinios.iogurtes",
        parent: "laticinios",
        name: "nucleo-alimentacao-cat-iogurtes",
        class: Some("complementary"),
        pattern: r"\b(iogurte|iog|bebida lactea|petit suisse)",
    },
    Sub {
        key: "mercearia.cereais",
        parent: "mercearia",
        name: "nucleo-alimentacao-cat-cereais",
        class: None,
        pattern: r"\b(arroz|feijao|lentilha|grao de bico|ervilha seca|milho pipoca)",
    },
    Sub {
        key: "mercearia.farinhas",
        parent: "mercearia",
        name: "nucleo-alimentacao-cat-farinhas",
        class: None,
        pattern: r"\b(farinha|fuba|macarrao|massa|espaguete|aveia|tapioca)",
    },
    Sub {
        key: "mercearia.oleos",
        parent: "mercearia",
        name: "nucleo-alimentacao-cat-oleos",
        class: None,
        pattern: r"\b(oleo|azeite|margarina|manteiga)",
    },
    Sub {
        key: "mercearia.acucar_cafe",
        parent: "mercearia",
        name: "nucleo-alimentacao-cat-acucar-cafe",
        class: None,
        pattern: r"\b(acucar|cafe|achocolatado|cha)\b",
    },
    Sub {
        key: "mercearia.temperos",
        parent: "mercearia",
        name: "nucleo-alimentacao-cat-temperos",
        class: Some("complementary"),
        pattern: r"\b(sal|tempero|molho|extrato|ketchup|maionese|mostarda|caldo)\b",
    },
    Sub {
        key: "mercearia.conservas",
        parent: "mercearia",
        name: "nucleo-alimentacao-cat-conservas",
        class: Some("complementary"),
        pattern: r"\b(milho verde|ervilha|seleta|azeitona|palmito|conserva)",
    },
    Sub {
        key: "padaria.paes",
        parent: "padaria",
        name: "nucleo-alimentacao-cat-paes",
        class: None,
        pattern: r"\b(pao|paes|bisnaguinha)",
    },
    Sub {
        key: "padaria.bolos",
        parent: "padaria",
        name: "nucleo-alimentacao-cat-bolos",
        class: Some("complementary"),
        pattern: r"\b(bolo|torta|sonho|rosca)",
    },
    Sub {
        key: "doces.biscoitos",
        parent: "doces_lanches",
        name: "nucleo-alimentacao-cat-biscoitos",
        class: Some("complementary"),
        pattern: r"\b(biscoito|bisc|bolacha|cream cracker|wafer)",
    },
    Sub {
        key: "doces.doces",
        parent: "doces_lanches",
        name: "nucleo-alimentacao-cat-doces",
        class: None,
        pattern: r"\b(chocolate|choc|bombom|bala|chiclete|doce|sorvete|pirulito)",
    },
    Sub {
        key: "doces.salgadinhos",
        parent: "doces_lanches",
        name: "nucleo-alimentacao-cat-salgadinhos",
        class: None,
        pattern: r"\b(salgadinho|batata palha|chips|amendoim|pipoca)",
    },
    Sub {
        key: "bebidas.agua",
        parent: "bebidas",
        name: "nucleo-alimentacao-cat-agua",
        class: Some("essential"),
        pattern: r"\bagua\b",
    },
    Sub {
        key: "bebidas.sucos",
        parent: "bebidas",
        name: "nucleo-alimentacao-cat-sucos",
        class: None,
        pattern: r"\b(suco|nectar)",
    },
    Sub {
        key: "bebidas.refrigerantes",
        parent: "bebidas",
        name: "nucleo-alimentacao-cat-refrigerantes",
        class: Some("superfluous"),
        pattern: r"\b(refri|refrigerante|coca|guarana|soda|energetico)",
    },
    // Segmentos de consumo (V136, docs/consumption.md). As expressões trazem também palavras em
    // inglês e espanhol, para a compra cadastrada à mão fora do Brasil.
    Sub {
        key: "casa.cama_mesa_banho",
        parent: "seg.casa_limpeza",
        name: "nucleo-alimentacao-cat-cama-mesa-banho",
        class: Some("complementary"),
        pattern: r"\b(toalha|lencol|fronha|edredom|cobertor|travesseiro|jogo de cama|towel|sheet|pillow|blanket|sabana|almohada|manta)",
    },
    Sub {
        key: "higiene.cosmeticos",
        parent: "seg.higiene_beleza",
        name: "nucleo-alimentacao-cat-cosmeticos",
        class: Some("complementary"),
        pattern: r"\b(perfume|colonia|batom|maquiagem|base liquida|rimel|esmalte|hidratante|protetor solar|makeup|lipstick|nail polish|lotion|sunscreen|maquillaje|labial)",
    },
    Sub {
        key: "vestuario.roupas",
        parent: "seg.vestuario",
        name: "nucleo-alimentacao-cat-roupas",
        class: None,
        pattern: r"\b(camisa|camiseta|blusa|calca|bermuda|short|vestido|saia|jaqueta|casaco|moletom|pijama|cueca|calcinha|sutia|meia|biquini|maio|shirt|t shirt|pants|jeans|dress|skirt|jacket|coat|sweater|hoodie|socks|pantalon|falda|chaqueta|abrigo|calcetines)",
    },
    Sub {
        key: "vestuario.calcados",
        parent: "seg.vestuario",
        name: "nucleo-alimentacao-cat-calcados",
        class: None,
        pattern: r"\b(tenis|sapato|sandalia|chinelo|bota|sapatilha|shoes|sneakers|boots|sandals|zapato|zapatilla|chancla)",
    },
    Sub {
        key: "vestuario.acessorios",
        parent: "seg.vestuario",
        name: "nucleo-alimentacao-cat-acessorios",
        class: Some("superfluous"),
        pattern: r"\b(bolsa|mochila|cinto|bone|oculos de sol|relogio|brinco|colar|pulseira|handbag|backpack|belt|watch|cap|bolso|cinturon|gorra|reloj)",
    },
    Sub {
        key: "eletronicos.informatica",
        parent: "seg.eletronicos",
        name: "nucleo-alimentacao-cat-informatica",
        class: None,
        pattern: r"\b(celular|smartphone|iphone|notebook|laptop|tablet|ipad|computador|monitor|teclado|mouse|fone|headset|carregador|cabo usb|pendrive|ssd|impressora|cartucho|toner|phone|charger|keyboard|printer|computadora|cargador|audifonos)",
    },
    Sub {
        key: "eletronicos.eletrodomesticos",
        parent: "seg.eletronicos",
        name: "nucleo-alimentacao-cat-eletrodomesticos",
        class: None,
        pattern: r"\b(geladeira|refrigerador|fogao|cooktop|forno|micro ondas|microondas|liquidificador|batedeira|cafeteira|air ?fryer|fritadeira|aspirador|ventilador|ar condicionado|maquina de lavar|lava loucas|ferro de passar|secador|chapinha|fridge|oven|microwave|blender|vacuum|dishwasher|washing machine|nevera|horno|licuadora|aspiradora|lavadora)",
    },
    Sub {
        key: "eletronicos.audio_tv",
        parent: "seg.eletronicos",
        name: "nucleo-alimentacao-cat-audio-tv",
        class: Some("complementary"),
        pattern: r"\b(tv|televisao|televisor|smart tv|caixa de som|soundbar|home theater|headphone|television|speaker|altavoz)",
    },
    Sub {
        key: "decoracao.moveis",
        parent: "seg.casa_decoracao",
        name: "nucleo-alimentacao-cat-moveis",
        class: None,
        pattern: r"\b(sofa|poltrona|cadeira|guarda roupa|armario|estante|rack|escrivaninha|colchao|cama box|comoda|furniture|chair|wardrobe|mattress|silla|colchon|ropero)",
    },
    Sub {
        key: "decoracao.decoracao",
        parent: "seg.casa_decoracao",
        name: "nucleo-alimentacao-cat-decoracao",
        class: Some("superfluous"),
        pattern: r"\b(quadro|tapete|cortina|almofada|luminaria|abajur|porta retrato|vela aromatica|enfeite|decor|rug|curtain|cushion|lamp|cuadro|alfombra|cojin)",
    },
    Sub {
        key: "decoracao.ferramentas",
        parent: "seg.casa_decoracao",
        name: "nucleo-alimentacao-cat-ferramentas",
        class: None,
        pattern: r"\b(furadeira|parafusadeira|martelo|chave de fenda|alicate|parafuso|bucha|lampada|tinta|pincel|rolo de pintura|mangueira|adubo|terra vegetal|drill|hammer|screwdriver|paint|taladro|martillo|pintura)",
    },
    Sub {
        key: "educacao.livros",
        parent: "seg.educacao",
        name: "nucleo-alimentacao-cat-livros",
        class: None,
        pattern: r"\b(livro|apostila|revista|book|libro)",
    },
    Sub {
        key: "educacao.escolar",
        parent: "seg.educacao",
        name: "nucleo-alimentacao-cat-material-escolar",
        class: None,
        pattern: r"\b(caderno|lapis|caneta|borracha|apontador|estojo|marca texto|papel sulfite|cola bastao|regua|notebook paper|pencil|pen|eraser|cuaderno|lapiz|boligrafo)",
    },
    Sub {
        key: "lazer.brinquedos",
        parent: "seg.lazer",
        name: "nucleo-alimentacao-cat-brinquedos",
        class: None,
        pattern: r"\b(brinquedo|boneca|boneco|lego|pelucia|quebra cabeca|carrinho de brinquedo|toy|doll|puzzle|juguete|muneca)",
    },
    Sub {
        key: "lazer.jogos",
        parent: "seg.lazer",
        name: "nucleo-alimentacao-cat-jogos",
        class: None,
        pattern: r"\b(jogo|game|playstation|xbox|nintendo|controle sem fio|juego)",
    },
    Sub {
        key: "lazer.esporte",
        parent: "seg.lazer",
        name: "nucleo-alimentacao-cat-esporte",
        class: Some("complementary"),
        pattern: r"\b(bola|bicicleta|raquete|halter|colchonete|patins|skate|chuteira|ball|bike|racket|pelota|raqueta)",
    },
    Sub {
        key: "lazer.presentes",
        parent: "seg.lazer",
        name: "nucleo-alimentacao-cat-presentes",
        class: None,
        pattern: r"\b(presente|gift|regalo)",
    },
    Sub {
        key: "pet.racao",
        parent: "pet",
        name: "nucleo-alimentacao-cat-racao",
        class: Some("essential"),
        pattern: r"\b(racao|petisco|sache pet|dog food|cat food|pienso|croquetas)",
    },
    Sub {
        key: "pet.cuidados",
        parent: "pet",
        name: "nucleo-alimentacao-cat-cuidados-pet",
        class: None,
        pattern: r"\b(areia sanitaria|areia gato|antipulgas|vermifugo|coleira|tapete higienico|shampoo pet|cat litter|flea|arena para gatos|collar)",
    },
    Sub {
        key: "automotivo.pecas",
        parent: "seg.automotivo",
        name: "nucleo-alimentacao-cat-pecas",
        class: Some("essential"),
        pattern: r"\b(pneu|oleo motor|oleo do motor|filtro de oleo|filtro de ar|bateria automotiva|pastilha de freio|palheta|aditivo radiador|tire|motor oil|brake|neumatico|aceite de motor)",
    },
    Sub {
        key: "automotivo.acessorios",
        parent: "seg.automotivo",
        name: "nucleo-alimentacao-cat-acessorios-auto",
        class: None,
        pattern: r"\b(tapete automotivo|capa de banco|suporte veicular|cera automotiva|shampoo automotivo|aromatizante carro|car wax|car mat)",
    },
];

/// Segmento (V136, docs/consumption.md): uma raiz nova da árvore, e as raízes da semente da V135
/// que descem para ela. Pet, Cigarros e Outros continuam raízes e já são segmentos.
pub struct Segment {
    pub key: &'static str,
    pub name: &'static str,
    pub is_food: bool,
    pub class: Option<&'static str>,
    pub children: &'static [&'static str],
}

pub const SEGMENTS: &[Segment] = &[
    Segment {
        key: "seg.alimentacao",
        name: "nucleo-alimentacao-seg-alimentacao",
        is_food: true,
        class: None,
        children: &[
            "hortifruti",
            "carnes",
            "laticinios",
            "mercearia",
            "padaria",
            "doces_lanches",
            "congelados",
            "bebidas",
            "bebidas_alcoolicas",
        ],
    },
    Segment {
        key: "seg.casa_limpeza",
        name: "nucleo-alimentacao-seg-casa-limpeza",
        is_food: false,
        class: None,
        children: &["limpeza", "casa"],
    },
    Segment {
        key: "seg.higiene_beleza",
        name: "nucleo-alimentacao-seg-higiene-beleza",
        is_food: false,
        class: None,
        children: &["higiene"],
    },
    Segment {
        key: "seg.saude",
        name: "nucleo-alimentacao-seg-saude",
        is_food: false,
        class: Some("essential"),
        children: &["farmacia", "bebe"],
    },
    Segment {
        key: "seg.vestuario",
        name: "nucleo-alimentacao-seg-vestuario",
        is_food: false,
        class: Some("complementary"),
        children: &[],
    },
    Segment {
        key: "seg.eletronicos",
        name: "nucleo-alimentacao-seg-eletronicos",
        is_food: false,
        class: Some("complementary"),
        children: &[],
    },
    Segment {
        key: "seg.casa_decoracao",
        name: "nucleo-alimentacao-seg-casa-decoracao",
        is_food: false,
        class: Some("complementary"),
        children: &[],
    },
    Segment {
        key: "seg.educacao",
        name: "nucleo-alimentacao-seg-educacao",
        is_food: false,
        class: Some("essential"),
        children: &[],
    },
    Segment {
        key: "seg.lazer",
        name: "nucleo-alimentacao-seg-lazer",
        is_food: false,
        class: Some("superfluous"),
        children: &[],
    },
    Segment {
        key: "pet",
        name: "",
        is_food: false,
        class: None,
        children: &[],
    },
    Segment {
        key: "seg.automotivo",
        name: "nucleo-alimentacao-seg-automotivo",
        is_food: false,
        class: Some("complementary"),
        children: &[],
    },
    Segment {
        key: "cigarros",
        name: "",
        is_food: false,
        class: None,
        children: &[],
    },
    Segment {
        key: "outros",
        name: "",
        is_food: false,
        class: None,
        children: &[],
    },
];

/// Chave da raiz de um grupo.
pub fn root_key(g: ItemGroup) -> String {
    // Os grupos de fora do mercado caem no segmento (as subcategorias dele decidem o resto).
    match g {
        ItemGroup::Vestuario | ItemGroup::Calcados => "seg.vestuario",
        ItemGroup::Eletronicos | ItemGroup::Eletrodomesticos => "seg.eletronicos",
        ItemGroup::CasaDecoracao | ItemGroup::Ferramentas => "seg.casa_decoracao",
        ItemGroup::Papelaria => "seg.educacao",
        ItemGroup::Brinquedos => "seg.lazer",
        other => other.key(),
    }
    .to_string()
}

/// Cria as categorias-semente que faltam (uma vez por cofre; as apagadas pela pessoa não
/// voltam, porque as marcas `food.seeded` e `food.seeded_segments` ficam gravadas).
pub fn ensure(conn: &Connection) -> Result<u32> {
    let mut n = 0;
    if !crate::settings::get_or::<bool>(conn, "food.seeded", false)? {
        n += seed_roots(conn)?;
        crate::settings::set(conn, "food.seeded", &true)?;
    }
    if !crate::settings::get_or::<bool>(conn, "food.seeded_segments", false)? {
        n += seed_segments(conn)?;
        crate::settings::set(conn, "food.seeded_segments", &true)?;
    }
    Ok(n)
}

/// Os grupos do cupom como raízes e as subcategorias deles (V135).
fn seed_roots(conn: &Connection) -> Result<u32> {
    let now = now_utc();
    let mut n = 0;
    for (i, r) in ROOTS.iter().enumerate() {
        n += conn.execute(
            "INSERT INTO product_category (id, parent_id, seed_key, name, is_food, class, tax_group,
                                           sort, created_at, updated_at)
             VALUES (?1, NULL, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)
             ON CONFLICT (seed_key) DO NOTHING",
            params![
                new_id(),
                root_key(r.group),
                // Sem os exemplos entre parênteses do rótulo do cupom.
                r.group
                    .label()
                    .split(" (")
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
                r.is_food,
                r.class,
                r.group.tax_group(),
                i as i64 * 10,
                now
            ],
        )? as u32;
    }
    n += seed_subs(conn, |s| !s.parent.starts_with("seg.") && s.parent != "pet")?;
    Ok(n)
}

/// Os segmentos (V136): as raízes novas, as raízes da V135 que descem para elas (só as que a
/// pessoa não moveu) e as subcategorias dos segmentos.
fn seed_segments(conn: &Connection) -> Result<u32> {
    let now = now_utc();
    let mut n = 0;
    for (i, g) in SEGMENTS.iter().enumerate() {
        let sort = i as i64 * 10;
        if g.key.starts_with("seg.") {
            n += conn.execute(
                "INSERT INTO product_category (id, parent_id, seed_key, name, is_food, class,
                                               sort, created_at, updated_at)
                 VALUES (?1, NULL, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
                 ON CONFLICT (seed_key) DO NOTHING",
                params![new_id(), g.key, t(g.name), g.is_food, g.class, sort, now],
            )? as u32;
        } else {
            // Raiz da V135 que já é um segmento: só a ordem.
            conn.execute(
                "UPDATE product_category SET sort = ?2 WHERE seed_key = ?1 AND parent_id IS NULL",
                params![g.key, sort],
            )?;
        }
        for child in g.children {
            conn.execute(
                "UPDATE product_category
                    SET parent_id = (SELECT id FROM product_category WHERE seed_key = ?2)
                  WHERE seed_key = ?1 AND parent_id IS NULL",
                params![child, g.key],
            )?;
        }
    }
    n += seed_subs(conn, |s| s.parent.starts_with("seg.") || s.parent == "pet")?;
    Ok(n)
}

fn seed_subs(conn: &Connection, which: impl Fn(&Sub) -> bool) -> Result<u32> {
    let now = now_utc();
    let mut n = 0;
    for (i, s) in SUBS.iter().enumerate().filter(|(_, s)| which(s)) {
        n += conn.execute(
            "INSERT INTO product_category (id, parent_id, seed_key, name, is_food, class, tax_group,
                                           sort, created_at, updated_at)
             SELECT ?1, p.id, ?2, ?3, p.is_food, ?4, p.tax_group, ?5, ?6, ?6
               FROM product_category p WHERE p.seed_key = ?7
             ON CONFLICT (seed_key) DO NOTHING",
            params![new_id(), s.key, t(s.name), s.class, i as i64, now, s.parent],
        )? as u32;
    }
    Ok(n)
}
