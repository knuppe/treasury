//! Grupo de cada item do cupom (hortifrúti, carnes, limpeza…) e o nome genérico do produto que
//! dá a alíquota do item (docs/receipts.md). As descrições dos cupons vêm abreviadas
//! ("SAB PO OMO 1KG", "LTE COND MOCOCA"): as expressões cobrem as formas mais comuns. Sem
//! produto, vale a mediana do grupo de imposto do item.

use std::sync::OnceLock;

use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::import::text::fold;

/// Grupo do item no cupom: o que a família compra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ItemGroup {
    Hortifruti,
    Carnes,
    Laticinios,
    Mercearia,
    Padaria,
    DocesLanches,
    Congelados,
    Bebidas,
    BebidasAlcoolicas,
    Limpeza,
    Higiene,
    Bebe,
    Pet,
    Farmacia,
    Casa,
    // Compras fora do mercado (loja de roupa, de eletrônicos, papelaria…): docs/consumption.md.
    Vestuario,
    Calcados,
    Eletronicos,
    Eletrodomesticos,
    CasaDecoracao,
    Ferramentas,
    Papelaria,
    Brinquedos,
    Cigarros,
    Outros,
}

impl ItemGroup {
    pub const ALL: [ItemGroup; 25] = [
        ItemGroup::Hortifruti,
        ItemGroup::Carnes,
        ItemGroup::Laticinios,
        ItemGroup::Mercearia,
        ItemGroup::Padaria,
        ItemGroup::DocesLanches,
        ItemGroup::Congelados,
        ItemGroup::Bebidas,
        ItemGroup::BebidasAlcoolicas,
        ItemGroup::Limpeza,
        ItemGroup::Higiene,
        ItemGroup::Bebe,
        ItemGroup::Pet,
        ItemGroup::Farmacia,
        ItemGroup::Casa,
        ItemGroup::Vestuario,
        ItemGroup::Calcados,
        ItemGroup::Eletronicos,
        ItemGroup::Eletrodomesticos,
        ItemGroup::CasaDecoracao,
        ItemGroup::Ferramentas,
        ItemGroup::Papelaria,
        ItemGroup::Brinquedos,
        ItemGroup::Cigarros,
        ItemGroup::Outros,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Hortifruti => "hortifruti",
            Self::Carnes => "carnes",
            Self::Laticinios => "laticinios",
            Self::Mercearia => "mercearia",
            Self::Padaria => "padaria",
            Self::DocesLanches => "doces_lanches",
            Self::Congelados => "congelados",
            Self::Bebidas => "bebidas",
            Self::BebidasAlcoolicas => "bebidas_alcoolicas",
            Self::Limpeza => "limpeza",
            Self::Higiene => "higiene",
            Self::Bebe => "bebe",
            Self::Pet => "pet",
            Self::Farmacia => "farmacia",
            Self::Casa => "casa",
            Self::Vestuario => "vestuario",
            Self::Calcados => "calcados",
            Self::Eletronicos => "eletronicos",
            Self::Eletrodomesticos => "eletrodomesticos",
            Self::CasaDecoracao => "casa_decoracao",
            Self::Ferramentas => "ferramentas",
            Self::Papelaria => "papelaria",
            Self::Brinquedos => "brinquedos",
            Self::Cigarros => "cigarros",
            Self::Outros => "outros",
        }
    }

    pub fn parse(s: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|g| g.key() == s)
            .unwrap_or(Self::Outros)
    }

    /// Nome do grupo no idioma da instalação.
    pub fn label(self) -> String {
        crate::i18n::t(match self {
            Self::Hortifruti => "nucleo-importacao-cupom-grupo-hortifruti",
            Self::Carnes => "nucleo-importacao-cupom-grupo-carnes",
            Self::Laticinios => "nucleo-importacao-cupom-grupo-laticinios",
            Self::Mercearia => "nucleo-importacao-cupom-grupo-mercearia",
            Self::Padaria => "nucleo-importacao-cupom-grupo-padaria",
            Self::DocesLanches => "nucleo-importacao-cupom-grupo-doces",
            Self::Congelados => "nucleo-importacao-cupom-grupo-congelados",
            Self::Bebidas => "nucleo-importacao-cupom-grupo-bebidas",
            Self::BebidasAlcoolicas => "nucleo-importacao-cupom-grupo-bebidas-alcoolicas",
            Self::Limpeza => "nucleo-importacao-cupom-grupo-limpeza",
            Self::Higiene => "nucleo-importacao-cupom-grupo-higiene",
            Self::Bebe => "nucleo-importacao-cupom-grupo-bebe",
            Self::Pet => "nucleo-importacao-cupom-grupo-pet",
            Self::Farmacia => "nucleo-importacao-cupom-grupo-farmacia",
            Self::Casa => "nucleo-importacao-cupom-grupo-casa",
            Self::Vestuario => "nucleo-importacao-cupom-grupo-vestuario",
            Self::Calcados => "nucleo-importacao-cupom-grupo-calcados",
            Self::Eletronicos => "nucleo-importacao-cupom-grupo-eletronicos",
            Self::Eletrodomesticos => "nucleo-importacao-cupom-grupo-eletrodomesticos",
            Self::CasaDecoracao => "nucleo-importacao-cupom-grupo-casa-decoracao",
            Self::Ferramentas => "nucleo-importacao-cupom-grupo-ferramentas",
            Self::Papelaria => "nucleo-importacao-cupom-grupo-papelaria",
            Self::Brinquedos => "nucleo-importacao-cupom-grupo-brinquedos",

            Self::Cigarros => "nucleo-importacao-cupom-grupo-cigarros",
            Self::Outros => "nucleo-importacao-cupom-grupo-outros",
        })
    }

    /// Grupo de imposto sobre consumo (`tax_burden`) quando o item não casa com um produto.
    pub fn tax_group(self) -> &'static str {
        match self {
            Self::Hortifruti
            | Self::Carnes
            | Self::Laticinios
            | Self::Mercearia
            | Self::Padaria => "alimentos_basicos",
            Self::DocesLanches | Self::Congelados => "alimentos_industrializados",
            Self::Bebidas => "bebidas_nao_alcoolicas",
            Self::BebidasAlcoolicas => "bebidas_alcoolicas",
            Self::Limpeza => "limpeza",
            Self::Higiene | Self::Bebe => "higiene_pessoal",
            Self::Pet => "pet",
            Self::Farmacia => "medicamentos",
            Self::Cigarros => "cigarros",
            Self::Vestuario => "vestuario",
            Self::Calcados => "calcados",
            Self::Eletronicos => "eletronicos_nacionais",
            Self::Eletrodomesticos => "eletrodomesticos",
            Self::CasaDecoracao => "moveis",
            Self::Ferramentas => "materiais_construcao",
            Self::Papelaria => "material_escolar",
            Self::Brinquedos => "brinquedos",
            Self::Casa | Self::Outros => "mercado",
        }
    }
}

/// (expressão, grupo, nome genérico do produto). A primeira que casa vale: as específicas
/// ("AGUA SANITARIA", "LEITE CONDENSADO") vêm antes das gerais ("AGUA", "LEITE").
const RULES: &[(&str, ItemGroup, Option<&str>)] = &[
    (r"\bcigarr", ItemGroup::Cigarros, Some("Cigarro")),
    // Roupas e calçados antes das frutas: "CAMISETA MANGA CURTA" não é manga (fruta). O imposto
    // vem da roupa.
    (
        r"\bcamis(a|eta|ete)|\bt-?shirt\b|\bpolo\b|\bregata\b",
        ItemGroup::Vestuario,
        Some("Camisa/Camiseta"),
    ),
    (r"\bbermuda", ItemGroup::Vestuario, Some("Bermuda")),
    (
        r"\bcalca\s*(jeans|jean)\b|\bjeans\b",
        ItemGroup::Vestuario,
        Some("Calça jeans (tecido)"),
    ),
    (
        r"\bjaqueta|\bcasaco|\bblazer",
        ItemGroup::Vestuario,
        Some("Jaqueta (roupa)"),
    ),
    (
        r"\btenis\b|\bsneaker",
        ItemGroup::Calcados,
        Some("Tênis Nacional"),
    ),
    (
        r"\bsapato|\bsandalia|\bchinelo|\bbota\b|\bsapatilha|\brasteira|\bmocassim",
        ItemGroup::Calcados,
        Some("Sapatos"),
    ),
    (
        r"\bblusa|\bvestido|\bsaia\b|\bshort|\bcalca\b|\bmeias?\b|\bcueca|\bcalcinha|\bsutia|\bpijama|\bmoletom|\bcardigan|\bmacacao|\bbiquini|\bmaio\b|\blegging|\bcamisola|\bjardineira|\bcolete\b|\bsueter",
        ItemGroup::Vestuario,
        Some("Roupas"),
    ),
    // Eletrônicos, eletrodomésticos, casa, ferramentas, papelaria e brinquedos (lojas fora do
    // mercado): o imposto vem da mediana do grupo.
    (
        r"\bcelular|\bsmartphone|\bnotebook|\btablet\b|\bfone\s*(de\s*)?ouvido|\bheadset|\bcarregador|\bcabo\s*usb|\bpen\s*drive|\bmouse\b|\bteclado|\bsmart\s*tv|\btelevisor|\bcaixa\s*de\s*som|\bmonitor\b",
        ItemGroup::Eletronicos,
        None,
    ),
    (
        r"\bliquidificador|\bbatedeira|\bcafeteira|\bair\s*fryer|\bfritadeira|\bventilador|\baspirador|\bmicro\s*-?\s*ondas|\bferro\s*de\s*passar|\bsanduicheira|\bgeladeira|\bfogao\b",
        ItemGroup::Eletrodomesticos,
        None,
    ),
    (
        r"\bcortina|\balmofada|\bluminaria|\babajur|\bsofa\b|\bpoltrona|\bcolchao|\bporta\s*retrato|\bquadro\s*decorativo|\bjogo\s*de\s*cama|\blencol|\bedredom|\btravesseiro",
        ItemGroup::CasaDecoracao,
        None,
    ),
    (
        r"\bfuradeira|\bparafusadeira|\bparafuso|\bmartelo|\balicate|\bchave\s*de\s*fenda|\bpincel|\brolo\s*de\s*pintura|\btinta\s*(acrilica|latex|esmalte|parede)|\bmangueira",
        ItemGroup::Ferramentas,
        None,
    ),
    (
        r"\bcaderno|\blapis\b|\bcaneta|\bborracha\s*(escolar|branca)|\bapontador|\bestojo\b|\bmarca\s*texto|\bsulfite|\bcola\s*bastao",
        ItemGroup::Papelaria,
        None,
    ),
    (
        r"\bbrinquedo|\bboneca|\bboneco\b|\blego\b|\bpelucia|\bquebra\s*-?\s*cabeca",
        ItemGroup::Brinquedos,
        None,
    ),
    // Limpeza.
    (
        r"\bagua\s*sanit|\bqboa\b|\bcandida\b",
        ItemGroup::Limpeza,
        Some("Água sanitária"),
    ),
    (
        r"\bsab(ao|\.)?\s*(em\s*)?po\b|\bsab\s*liq|\blava\s*roupa|\bomo\b|\bbrilhante\b|\bariel\b",
        ItemGroup::Limpeza,
        Some("Sabão em pó"),
    ),
    (
        r"\bsab(ao|\.)?\s*(em\s*)?barra\b|\bsab\s*pedra|\bsabao\s*ype",
        ItemGroup::Limpeza,
        Some("Sabão em barra"),
    ),
    (
        r"\bdeterg|\blava\s*louca",
        ItemGroup::Limpeza,
        Some("Detergente"),
    ),
    (
        r"\bamac(iante)?\b|\bcomfort\b|\bdowny\b",
        ItemGroup::Limpeza,
        Some("Amaciante"),
    ),
    (
        r"\bdesinf|\bpinho\s*sol|\bveja\b|\blimpa\s*(vidro|piso|aluminio)|\bmultiuso|\blysoform",
        ItemGroup::Limpeza,
        Some("Desinfetante"),
    ),
    (
        r"\besponja|\bbom\s*bril|\bpalha\s*de\s*aco|\bassolan",
        ItemGroup::Limpeza,
        Some("Esponja de Aço (pacote com 4 unidades)"),
    ),
    (
        r"\bvassoura|\brodo\b|\bpa\s*de\s*lixo",
        ItemGroup::Limpeza,
        Some("Vassoura"),
    ),
    (
        r"\balcool\s*(70|liq|gel|46)|\balcool\b",
        ItemGroup::Limpeza,
        Some("Álcool (material de limpeza)"),
    ),
    (
        r"\bsapon|\bsaponaceo|\bcif\b",
        ItemGroup::Limpeza,
        Some("Saponáceo"),
    ),
    (r"\bcera\b|\blustra", ItemGroup::Limpeza, Some("Cera")),
    (
        r"\bsaco\s*(de\s*)?lixo|\bpano\s*(de\s*)?(chao|prato|multiuso)|\bluva\s*(latex|limpeza)|\binset|\bsbp\b|\bbaygon|\braid\b",
        ItemGroup::Limpeza,
        None,
    ),
    // Bebê e higiene.
    (
        r"\bfralda|\blenco\s*umed|\btoalha\s*umed",
        ItemGroup::Bebe,
        None,
    ),
    (
        r"\bpapel\s*hig|\bpap\s*hig|\bp\.?\s*hig",
        ItemGroup::Higiene,
        Some("Papel Higiênico (com 4 rolos)"),
    ),
    (
        r"\bcreme\s*dent|\bcr\s*dent|\bpasta\s*(de\s*)?dent|\bcolgate|\bsorriso\b|\boral\s*b",
        ItemGroup::Higiene,
        Some("Pasta de dentes"),
    ),
    (
        r"\bescova\s*(de\s*)?dent|\bescova\s*dental",
        ItemGroup::Higiene,
        Some("Escova de Dente"),
    ),
    (
        r"\bsabonete|\bsab(\.)?\s*(liq\s*)?(dove|lux|protex|nivea|palmolive)",
        ItemGroup::Higiene,
        Some("Sabonete em barra"),
    ),
    (
        r"\bshampoo|\bsh\s|\bxampu",
        ItemGroup::Higiene,
        Some("Shampoo"),
    ),
    (
        r"\bcondic|\bcreme\s*(de\s*)?pentear",
        ItemGroup::Higiene,
        Some("Condicionadores para cabelos"),
    ),
    (
        r"\bdesod|\bantitransp|\brexona|\bdove\s*aero",
        ItemGroup::Higiene,
        Some("Desodorantes"),
    ),
    (
        r"\babsorv|\bprotetor\s*diario",
        ItemGroup::Higiene,
        Some("Absorvente higiênico"),
    ),
    (
        r"\baparelho\s*(de\s*)?barb|\bgillette|\bprestobarba",
        ItemGroup::Higiene,
        Some("Aparelho de barbear"),
    ),
    (
        r"\balgodao|\bcotonete|\bhastes",
        ItemGroup::Higiene,
        Some("Algodão de limpeza"),
    ),
    (
        r"\bprotetor\s*solar|\bfps\b",
        ItemGroup::Farmacia,
        Some("Protetor solar"),
    ),
    // Farmácia.
    (
        r"\bdipirona|\bparacetamol|\bibuprofeno|\bdorflex|\bneosaldina|\btylenol|\bmedicam|\bcomprimido|\bcomp\b|\bxarope|\bvitamina\b",
        ItemGroup::Farmacia,
        Some("Medicamentos de uso humano"),
    ),
    (
        r"\bcurativo|\bband\s*aid|\besparadrapo|\bgaze\b",
        ItemGroup::Farmacia,
        Some("Band-aid (curativo)"),
    ),
    (
        r"\brepelente|\bexposis|\bautan",
        ItemGroup::Farmacia,
        Some("Repelente de Mosquitos"),
    ),
    // Pet.
    (
        r"\bracao|\bpedigree|\bwhiskas|\bgolden\b|\bpremier\s*pet|\bareia\s*(sanit|gato)|\bpetisco",
        ItemGroup::Pet,
        Some("Ração para animais"),
    ),
    // Bebidas alcoólicas.
    (
        r"\bcerv|\bbrahma|\bskol\b|\bheineken|\bamstel|\bitaipava|\bantarctica|\bbudweiser|\bstella\b|\bspaten|\boriginal\s*\d",
        ItemGroup::BebidasAlcoolicas,
        Some("Cerveja (lata)"),
    ),
    (
        r"\bchope|\bchopp",
        ItemGroup::BebidasAlcoolicas,
        Some("Chope"),
    ),
    (
        r"\bespumante",
        ItemGroup::BebidasAlcoolicas,
        Some("Espumante"),
    ),
    (
        r"\bvinho\b|\bvho\b|\bvh\s",
        ItemGroup::BebidasAlcoolicas,
        Some("Vinho nacional"),
    ),
    (
        r"\bcachaca|\bpinga\b|\baguardente|\b51\b|\bypioca|\bvelho\s*barreiro",
        ItemGroup::BebidasAlcoolicas,
        Some("Cachaça"),
    ),
    (
        r"\bvodka|\bvodca|\bsmirnoff|\babsolut\b",
        ItemGroup::BebidasAlcoolicas,
        Some("Vodca"),
    ),
    (
        r"\bwhisk|\bjohnnie|\bred\s*label|\bjack\s*daniel",
        ItemGroup::BebidasAlcoolicas,
        Some("Wisky"),
    ),
    (
        r"\bgin\b|\brum\b|\blicor|\bcatuaba|\bsidra\b|\bvermute|\bmartini\b",
        ItemGroup::BebidasAlcoolicas,
        Some("Sidra"),
    ),
    // Bebidas.
    (
        r"\bagua\s*(de\s*)?coco",
        ItemGroup::Bebidas,
        Some("Água de coco"),
    ),
    (
        r"\bagua\s*(min|mineral|s/gas|c/gas|sem\s*gas|com\s*gas|crystal|bonafont|indaia)|\bagua\b",
        ItemGroup::Bebidas,
        Some("Água mineral"),
    ),
    (
        r"\brefri|\bcoca\b|\bcoca\s*cola|\bguarana|\bpepsi|\bfanta|\bsprite|\bsoda\b|\bschweppes|\bkuat",
        ItemGroup::Bebidas,
        Some("Refrigerante (lata)"),
    ),
    (
        r"\bsuco\b|\bsuco\s|\bdel\s*valle|\bnectar|\bades\b|\bkapo",
        ItemGroup::Bebidas,
        Some("Suco pronto"),
    ),
    (
        r"\brefresco|\btang\b|\bclight|\bfresh\b|\bmid\b",
        ItemGroup::Bebidas,
        Some("Refresco em pó"),
    ),
    (
        r"\bcha\b|\bcha\s|\bmate\s*leao|\bisotonico|\bgatorade|\benerg|\bred\s*bull|\bmonster",
        ItemGroup::Bebidas,
        Some("Chá Verde"),
    ),
    // Leite e derivados (antes de "leite" vêm os doces e o leite em pó).
    (
        r"\blei?te?\s*cond|\bleite\s*moca|\bmoca\b|\bcondensado",
        ItemGroup::DocesLanches,
        Some("Leite condensado lata"),
    ),
    (
        r"\bleite\s*(em\s*)?po\b|\blte\s*po|\bninho\b",
        ItemGroup::Laticinios,
        Some("Leite em pó"),
    ),
    (
        r"\bachoc|\bnescau|\btoddy|\bovomaltine",
        ItemGroup::DocesLanches,
        Some("Achocolatado"),
    ),
    (
        r"\biog|\bdanone|\bactivia|\bdanoninho|\bbebida\s*lactea",
        ItemGroup::Laticinios,
        Some("Iogurte"),
    ),
    (
        r"\bqueijo|\bqjo\b|\bmussarela|\bmucarela|\bprato\b|\bparmesao|\brequeij|\bricota|\bminas\s*frescal|\bcream\s*cheese",
        ItemGroup::Laticinios,
        Some("Queijo"),
    ),
    (
        r"\bmanteiga|\bmant\b",
        ItemGroup::Laticinios,
        Some("Manteiga"),
    ),
    (
        r"\bmargarina|\bmarg\b|\bqualy|\bdoriana|\bbecel",
        ItemGroup::Laticinios,
        Some("Margarina (500g)"),
    ),
    (
        r"\bcreme\s*(de\s*)?leite|\bcr\s*leite|\bnata\b",
        ItemGroup::Laticinios,
        Some("Leite de vaca (pacote)"),
    ),
    (
        r"\bleite\b|\blte\b|\blt\s*(int|desn|semi)",
        ItemGroup::Laticinios,
        Some("Leite de vaca (pacote)"),
    ),
    (
        r"\bovos?\b|\bovo\s*(branco|vermelho|caipira)",
        ItemGroup::Laticinios,
        Some("Ovos de galinha"),
    ),
    (
        r"\bpresunto|\bapresuntado|\bmortadela|\bsalame|\bpeito\s*(de\s*)?peru|\bblanquet",
        ItemGroup::Laticinios,
        Some("Chester/Perú/Pernil"),
    ),
    // Carnes, aves e peixes.
    (
        r"\bfrango|\bfgo\b|\bcoxa\b|\bsobrecoxa|\bpeito\s*(de\s*)?frango|\bfile\s*(de\s*)?peito|\basa\s*(de\s*)?frango|\bchester",
        ItemGroup::Carnes,
        Some("Frango"),
    ),
    (
        r"\bpeixe|\btilapia|\bsalmao|\bmerluza|\bsardinha|\batum\b|\bbacalhau|\bpescada",
        ItemGroup::Carnes,
        Some("Peixes (carne)"),
    ),
    (r"\bcamarao", ItemGroup::Carnes, Some("Camarão")),
    (
        r"\bperu\b|\bpernil|\blombo|\bsuino|\bcostela\s*suina|\bbisteca",
        ItemGroup::Carnes,
        Some("Chester/Perú/Pernil"),
    ),
    (
        r"\blinguica|\bsalsicha|\bbacon|\bcalabresa",
        ItemGroup::Carnes,
        Some("Chester/Perú/Pernil"),
    ),
    (
        r"\bcarne|\bbovin|\bpatinho|\bacem\b|\bmusculo|\bcontra\s*file|\bpicanha|\balcatra|\bfraldinha|\bcostela|\bcoxao|\bmaminha|\bfile\s*mignon|\bmoida|\bbife\b|\bcupim|\bhamburguer",
        ItemGroup::Carnes,
        Some("Carne bovina"),
    ),
    // Padaria (pão de forma antes do pão).
    (
        r"\bpao\s*(de\s*)?forma|\bpullman|\bwickbold|\bvisconti\s*pao|\bpao\s*(de\s*)?hot\s*dog|\bpao\s*(de\s*)?hamb",
        ItemGroup::Padaria,
        Some("Pão de forma"),
    ),
    (
        r"\bpao\b|\bpaes\b|\bbisnaguinha|\bbaguete|\bciabatta",
        ItemGroup::Padaria,
        Some("Pão Francês"),
    ),
    (
        r"\bbolo\b|\bsonho\b|\brosca\b|\btorta\b|\bsalgado\b|\bcoxinha",
        ItemGroup::Padaria,
        Some("Bolo de brigadeiro"),
    ),
    // Doces e lanches.
    (
        r"\bbisc|\bbolacha|\bcookie|\bwafer|\bcream\s*cracker|\brecheado",
        ItemGroup::DocesLanches,
        Some("Biscoito"),
    ),
    (
        r"\bchoc|\bbombom|\bbombons|\bbis\b|\bkit\s*kat|\btrento|\blacta|\bgaroto\b|\bnestle\s*(classic|alpino)",
        ItemGroup::DocesLanches,
        Some("Chocolate"),
    ),
    (
        r"\bsalgadinho|\bbatata\s*(frita\s*)?(lays|ruffles|pringles|chips)|\belma\s*chips|\bdoritos|\bcheetos|\bfandangos|\bamendoim",
        ItemGroup::DocesLanches,
        Some("Salgadinhos em pacote"),
    ),
    (
        r"\bsorvete|\bpicole|\bkibon",
        ItemGroup::Congelados,
        Some("Sorvete de massa/Picolé"),
    ),
    (
        r"\bbala\b|\bbalas\b|\bchiclete|\btrident|\bpirulito|\bdoce\s*de|\bgoiabada|\bpacoca|\bgelatina|\bpudim|\bmistura\s*(p/|para)?\s*bolo",
        ItemGroup::DocesLanches,
        Some("Gelatina"),
    ),
    // Congelados e prontos.
    (
        r"\bpizza|\blasanha|\bcongel|\bnuggets|\bempanado|\bsadia\b|\bperdigao|\bseara\b|\bbatata\s*(pre\s*frita|congel)|\bpao\s*de\s*queijo",
        ItemGroup::Congelados,
        Some("Pizza Semi Pronta"),
    ),
    // Mercearia.
    (r"\barroz\b|\barr\b", ItemGroup::Mercearia, Some("Arroz")),
    (
        r"\bfeijao|\bfeij\b|\bfjo\b",
        ItemGroup::Mercearia,
        Some("Feijão"),
    ),
    (
        r"\bmacarr|\bmac\b|\bespaguete|\bparafuso\b|\bpenne|\blamen|\bmiojo|\bnissin|\bmassa\s*(p/|de)?",
        ItemGroup::Mercearia,
        Some("Macarrão"),
    ),
    (
        r"\bacucar|\bacuc\b|\bunião\b|\buniao\b|\bcristal\b",
        ItemGroup::Mercearia,
        Some("Açúcar"),
    ),
    (r"\badocante", ItemGroup::Mercearia, Some("Adoçante")),
    (
        r"\bcafe\b|\bcafe\s|\bpilao|\bmelitta|\b3\s*coracoes|\bnescafe|\bcapsula",
        ItemGroup::Mercearia,
        Some("Café em pó não solúvel"),
    ),
    (
        r"\boleo|\bazeite|\bsoya\b|\bliza\b",
        ItemGroup::Mercearia,
        Some("Óleo de cozinha"),
    ),
    (
        r"\bfarinha|\bfar\s*(trigo|mand|milho)|\bfuba\b|\bamido|\bmaizena|\btapioca|\bpolvilho|\baveia|\bgranola",
        ItemGroup::Mercearia,
        Some("Farinha de Trigo"),
    ),
    (
        r"\bsal\b|\bsal\s*(refin|grosso|marinho)",
        ItemGroup::Mercearia,
        Some("Sal de mesa"),
    ),
    (
        r"\bmolho\s*(de\s*)?tomate|\bextrato\s*(de\s*)?tomate|\bpomarola|\bquero\b|\bmolho\b",
        ItemGroup::Mercearia,
        Some("Molho de tomate (lata)"),
    ),
    (
        r"\bmaionese|\bhellmann",
        ItemGroup::Mercearia,
        Some("Maionese (vidro)"),
    ),
    (
        r"\bketchup|\bcatchup|\bmostarda",
        ItemGroup::Mercearia,
        Some("Catchup"),
    ),
    (r"\bvinagre", ItemGroup::Mercearia, Some("Vinagre")),
    (
        r"\bmilho\s*(verde|cozido)|\bervilha|\bseleta|\bpalmito|\bazeitona|\bconserva|\blata\b",
        ItemGroup::Mercearia,
        Some("Ervilhas em lata"),
    ),
    (
        r"\btempero|\bcaldo\s*(knorr|maggi|sazon)|\bsazon|\bcolorau|\boregano|\bpimenta|\bcanela|\bcominho",
        ItemGroup::Mercearia,
        None,
    ),
    (r"\bmel\b", ItemGroup::Mercearia, Some("Mel")),
    (
        r"\blentilha|\bgrao\s*de\s*bico|\bsoja\b",
        ItemGroup::Mercearia,
        Some("Lentilha"),
    ),
    // Hortifrúti.
    (r"\btomate|\btom\b", ItemGroup::Hortifruti, Some("Tomate")),
    (
        r"\bbatata|\bmandioca|\baipim|\bmacaxeira|\binhame|\bcara\b",
        ItemGroup::Hortifruti,
        Some("Batata"),
    ),
    (r"\bcebola|\balho\b", ItemGroup::Hortifruti, Some("Cebola")),
    (
        r"\balface|\brucula|\bcouve|\bespinafre|\bagriao|\brepolho|\bbrocolis|\bcouve\s*flor|\bcheiro\s*verde|\bsalsa\b|\bcebolinha|\bcoentro|\bverdura|\bhortalica",
        ItemGroup::Hortifruti,
        Some("Verduras em geral"),
    ),
    (
        r"\bcenoura|\bbeterraba|\babobrinha|\babobora|\bchuchu|\bpepino|\bpimentao|\bberinjela|\bquiabo|\bvagem|\blegume",
        ItemGroup::Hortifruti,
        Some("Verduras em geral"),
    ),
    (
        r"\bbanana|\bmaca\b|\blaranja|\blimao|\bmamao|\bmanga\b|\bmelancia|\bmelao|\buva\b|\bmorango|\babacaxi|\bpera\b|\bkiwi|\bgoiaba|\bmaracuja|\btangerina|\bponkan|\bfruta|\babacate|\bcoco\b",
        ItemGroup::Hortifruti,
        Some("Frutas"),
    ),
    // Casa e utilidades.
    (
        r"\bpilha|\bbateria\b",
        ItemGroup::Casa,
        Some("Pilhas/Baterias"),
    ),
    (
        r"\bvela\b|\bfosforo|\bisqueiro",
        ItemGroup::Casa,
        Some("Vela"),
    ),
    (
        r"\blampada|\bcarvao|\bguardanapo|\bpapel\s*(toalha|aluminio|filme)|\bfilme\s*pvc|\bcopo\s*desc|\bprato\s*desc|\btalher|\bpote\b|\bvasilha|\bfiltro\s*(de\s*)?papel|\bsacola",
        ItemGroup::Casa,
        Some("Plástico  em geral"),
    ),
];

fn compiled() -> &'static [(Regex, ItemGroup, Option<&'static str>)] {
    static RE: OnceLock<Vec<(Regex, ItemGroup, Option<&'static str>)>> = OnceLock::new();
    RE.get_or_init(|| {
        RULES
            .iter()
            .map(|(re, g, p)| {
                (
                    RegexBuilder::new(re)
                        .case_insensitive(true)
                        .build()
                        .expect("regex do item"),
                    *g,
                    *p,
                )
            })
            .collect()
    })
}

/// Grupo (e nome genérico do produto, quando o idioma é o português) de um item pela descrição, nos
/// idiomas do perfil de consumo do país, na ordem. O primeiro idioma que reconhece o item vale.
pub fn classify_in(description: &str, langs: &[String]) -> (ItemGroup, Option<&'static str>) {
    let f = fold(description);
    for lang in langs {
        if lang == "pt" {
            let (g, p) = classify(description);
            if g != ItemGroup::Outros || p.is_some() {
                return (g, p);
            }
        } else if let Some(g) = super::classify_words::classify(&f, lang) {
            return (g, None);
        }
    }
    (ItemGroup::Outros, None)
}

/// Grupo e nome genérico do produto de um item pela descrição do cupom.
pub fn classify(description: &str) -> (ItemGroup, Option<&'static str>) {
    let f = fold(description);
    compiled()
        .iter()
        .find(|(re, ..)| re.is_match(&f))
        .map(|(_, g, p)| (*g, *p))
        .unwrap_or((ItemGroup::Outros, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_receipt_lines() {
        let g = |d: &str| classify(d).0;
        assert_eq!(
            classify("ARROZ TIO JOAO T1 5KG"),
            (ItemGroup::Mercearia, Some("Arroz"))
        );
        assert_eq!(g("FEIJAO CARIOCA KICALDO 1KG"), ItemGroup::Mercearia);
        assert_eq!(g("BANANA PRATA KG"), ItemGroup::Hortifruti);
        assert_eq!(g("ALFACE CRESPA UN"), ItemGroup::Hortifruti);
        assert_eq!(g("CARNE MOIDA PATINHO KG"), ItemGroup::Carnes);
        assert_eq!(g("FILE PEITO FRANGO SADIA 1KG"), ItemGroup::Carnes);
        assert_eq!(g("LEITE INTEGRAL ITALAC 1L"), ItemGroup::Laticinios);
        assert_eq!(
            classify("LEITE COND MOCOCA 395G"),
            (ItemGroup::DocesLanches, Some("Leite condensado lata"))
        );
        assert_eq!(g("QUEIJO MUSSARELA FATIADO"), ItemGroup::Laticinios);
        assert_eq!(g("PAO FRANCES KG"), ItemGroup::Padaria);
        assert_eq!(classify("PAO DE FORMA PULLMAN").1, Some("Pão de forma"));
        assert_eq!(g("BISC RECHEADO NEGRESCO"), ItemGroup::DocesLanches);
        assert_eq!(g("REFRIG COCA COLA 2L"), ItemGroup::Bebidas);
        assert_eq!(g("AGUA MINERAL S/GAS 1,5L"), ItemGroup::Bebidas);
        assert_eq!(g("AGUA SANITARIA QBOA 2L"), ItemGroup::Limpeza);
        assert_eq!(g("CERV BRAHMA LT 350ML"), ItemGroup::BebidasAlcoolicas);
        assert_eq!(g("DETERG YPE NEUTRO 500ML"), ItemGroup::Limpeza);
        assert_eq!(classify("SAB PO OMO 1KG").1, Some("Sabão em pó"));
        assert_eq!(g("PAPEL HIG NEVE 12 ROLOS"), ItemGroup::Higiene);
        assert_eq!(g("CREME DENTAL COLGATE 90G"), ItemGroup::Higiene);
        assert_eq!(g("FRALDA PAMPERS M 30UN"), ItemGroup::Bebe);
        assert_eq!(g("RACAO PEDIGREE 1KG"), ItemGroup::Pet);
        assert_eq!(g("DIPIRONA 500MG 10 COMP"), ItemGroup::Farmacia);
        assert_eq!(g("PILHA DURACELL AA"), ItemGroup::Casa);
        assert_eq!(g("XYZ 123"), ItemGroup::Outros);
        // Roupa com "manga" não é fruta (cupom da Renner).
        assert_eq!(
            classify("CAMISETA MASC MANGA CURTA ESTAMPADA"),
            (ItemGroup::Vestuario, Some("Camisa/Camiseta"))
        );
        assert_eq!(classify("BERMUDA SARJA MASC").1, Some("Bermuda"));
        assert_eq!(classify("BLUSA MANGA LONGA FEM").1, Some("Roupas"));
        assert_eq!(classify("TENIS CASUAL ADULTO").1, Some("Tênis Nacional"));
        assert_eq!(classify("MANGA PALMER KG").1, Some("Frutas"));
    }
}
