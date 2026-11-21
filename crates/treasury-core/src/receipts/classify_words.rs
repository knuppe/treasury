//! Palavras do classificador em outros idiomas (docs/design/consumption-review.md, C4 e D1): o
//! grupo do item pela descrição de cupons em inglês e em espanhol, com as abreviações mais
//! comuns ("WHL MLK", "ORG BNNS", "GRND BF"). A primeira expressão que casa vale: as
//! específicas vêm antes das gerais. O português (com o nome genérico do produto) fica em `classify`.
//! O idioma vem do perfil de consumo do país (`food::profile`).

use std::sync::OnceLock;

use regex::{Regex, RegexBuilder};

use super::classify::ItemGroup;

use ItemGroup::*;

const EN: &[(&str, ItemGroup)] = &[
    (r"\bcigar|\btobacco|\bvape\b", Cigarros),
    // Taxas que são dinheiro mas não são produto.
    (
        r"\bbottle\s*dep|\bdeposit\b|\bcrv\b|\bbag\s*fee|\bbag\s*charge|\brecycl\w*\s*fee|\benviro\w*\s*fee",
        Outros,
    ),
    (
        r"\bt-?shirt|\bshirt|\bjeans\b|\bjacket|\bhoodie|\bsweater|\bdress\b|\bpants\b|\bshorts\b|\bsocks?\b|\bunderwear|\bleggings",
        Vestuario,
    ),
    (
        r"\bsneaker|\bshoes?\b|\bboots?\b|\bsandal|\bslipper",
        Calcados,
    ),
    (
        r"\bphone\b|\blaptop|\btablet\b|\bheadphone|\bearbud|\bcharger|\busb\b|\bhdmi\b|\bmouse\b|\bkeyboard|\bmonitor\b|\btv\b|\bspeaker",
        Eletronicos,
    ),
    (
        r"\bblender|\btoaster|\bmicrowave|\bvacuum|\bair\s*fryer|\bcoffee\s*maker|\bkettle\b",
        Eletrodomesticos,
    ),
    (
        r"\bdiaper|\bbaby\s*wipes?|\bformula\b|\bpacifier|\bbaby\s*food",
        Bebe,
    ),
    (
        r"\bdog\s*food|\bcat\s*food|\bcat\s*litter|\bkibble|\bpet\s*treat|\bdog\s*treat",
        Pet,
    ),
    (
        r"\bibuprofen|\bacetamin|\btylenol|\badvil\b|\baspirin|\bvitamin|\bcough|\ballergy\b|\bantacid|\bbandage|\bband-?aid|\bmelatonin",
        Farmacia,
    ),
    (
        r"\bbeer\b|\bwine\b|\bvodka|\bwhiske?y|\brum\b|\btequila|\bgin\b|\bcider\b|\blager\b|\bipa\b|\bseltzer\s*hard|\bhard\s*seltzer|\bchampagne",
        BebidasAlcoolicas,
    ),
    (
        r"\bdetergent|\bbleach\b|\bdish\s*soap|\bdishwash|\bcleaner|\bsponge|\btrash\s*bags?|\bgarbage\s*bags?|\blaundry|\bfabric\s*soft|\bdisinfect|\bwipes\b",
        Limpeza,
    ),
    (
        r"\btoilet\s*paper|\bbath\s*tissue|\btp\b|\btoothpaste|\btoothbrush|\bshampoo|\bconditioner|\bdeodorant|\bbody\s*wash|\bsoap\b|\brazor|\bfloss\b|\blotion|\btampon|\bpads\b|\bmouthwash",
        Higiene,
    ),
    (
        r"\bpaper\s*towel|\bnapkin|\bfoil\b|\bplastic\s*wrap|\bzip\w*\s*bags?|\bbatter(y|ies)|\blight\s*bulb|\bcandle|\bcharcoal|\bpaper\s*plates?|\bcups\b",
        Casa,
    ),
    (
        r"\bnotebook|\bpencil|\bpens?\b|\bmarker|\bcrayon|\bbinder|\bstapl|\bprinter\s*paper",
        Papelaria,
    ),
    (
        r"\btoy\b|\btoys\b|\blego\b|\bdoll\b|\bpuzzle|\bboard\s*game",
        Brinquedos,
    ),
    // Comida: as formas específicas antes ("ice cream" antes de "cream", "peanut butter"
    // antes de "butter", "hot dog buns" antes de "hot dog").
    (
        r"\bice\s*cream|\bfrozen|\bfrzn\b|\bfz\b|\bpizza\b|\bpopsicle|\bwaffles?\b|\btater\s*tots|\bfries\b",
        Congelados,
    ),
    (
        r"\bpeanut\s*butter|\bpb\b|\bjelly\b|\bjam\b|\bhoney\b|\bmaple\s*syrup",
        Mercearia,
    ),
    (
        r"\bbuns?\b|\bbread\b|\bbrd\b|\bbagel|\bmuffin|\bcroissant|\btortilla|\bdonut|\bdoughnut|\bbaguette|\brolls?\b|\bpita\b",
        Padaria,
    ),
    (
        r"\bcookie|\bcandy|\bchocolate|\bchoc\b|\bchips\b|\bcrackers?\b|\bpretzel|\bpopcorn|\bgranola\s*bar|\bsnack|\bgum\b|\bbrownie|\bcake\b|\bpie\b|\bpudding|\bmarshmallow",
        DocesLanches,
    ),
    (
        r"\bmilk\b|\bmlk\b|\bcheese|\bchs\b|\bchz\b|\bcheddar|\bmozz|\byogurt|\byog\b|\bygrt\b|\bbutter\b|\bbttr\b|\bcream\b|\bcrm\b|\beggs?\b|\bhalf\s*(&|and)\s*half|\bsour\s*cream|\bcottage",
        Laticinios,
    ),
    (
        r"\bchicken|\bchkn\b|\bchix\b|\bbeef\b|\bbf\b|\bgrnd\b|\bground\s*(beef|turkey|pork)|\bsteak|\bpork\b|\bbacon|\bsausage|\bham\b|\bturkey|\btrky\b|\bsalmon|\btuna\b|\bshrimp|\bfish\b|\btilapia|\bcod\b|\bribs?\b|\bbrisket|\bhot\s*dogs?|\bdeli\b|\bsalami|\bpepperoni|\blamb\b",
        Carnes,
    ),
    (
        r"\bwater\b|\bwtr\b|\bsoda\b|\bcola\b|\bjuice\b|\bjce\b|\boj\b|\blemonade|\biced\s*tea|\bsports?\s*drink|\benergy\s*drink|\bsparkling|\bseltzer|\bkombucha|\bcoffee\s*drink",
        Bebidas,
    ),
    (
        r"\bbanana|\bbnns?\b|\bapples?\b|\bappl\b|\borange|\bgrapes?\b|\bstrawberr|\bblueberr|\braspberr|\bberries|\blemon|\blime\b|\bavocado|\bavcdo\b|\btomato|\bpotato|\bonions?\b|\bgarlic|\blettuce|\bspinach|\bkale\b|\bcarrot|\bbroccoli|\bcucumber|\bpeppers?\b|\bcelery|\bmushroom|\bzucchini|\bsquash|\bcorn\b|\bmelon|\bpeach|\bpears?\b|\bmango|\bpineapple|\bcilantro|\bginger|\bcabbage|\bsalad\b|\bproduce\b|\borganic\b|\borg\b",
        Hortifruti,
    ),
    (
        r"\brice\b|\bbeans?\b|\bpasta\b|\bspaghetti|\bnoodle|\bmacaroni|\bflour\b|\bsugar\b|\bsalt\b|\boil\b|\bolive\s*oil|\bvinegar|\bcereal|\boat(s|meal)?\b|\bcoffee\b|\btea\b|\bsoup\b|\bbroth|\bsauce\b|\bketchup|\bmustard|\bmayo|\bsalsa\b|\bspice|\bseasoning|\bcanned|\bnuts\b|\balmonds?\b|\bpeanuts?\b|\btofu\b|\blentil",
        Mercearia,
    ),
];

const ES: &[(&str, ItemGroup)] = &[
    (r"\bcigarr|\btabaco", Cigarros),
    (
        r"\bcamis(a|eta)|\bpantalon|\bvaquero|\bchaqueta|\bsudadera|\bvestido|\bcalcetin|\bfalda\b",
        Vestuario,
    ),
    (r"\bzapat|\bzapatilla|\bbotas?\b|\bsandalia", Calcados),
    (
        r"\bmovil\b|\bcelular|\bportatil|\btableta|\bauricular|\bcargador|\btelevisor|\bteclado|\braton\b",
        Eletronicos,
    ),
    (r"\bpanal|\btoallitas\s*bebe|\bchupete|\bpotito", Bebe),
    (
        r"\bpienso|\bcomida\s*(para\s*)?(perro|gato)|\barena\s*(para\s*)?gato",
        Pet,
    ),
    (
        r"\bibuprofeno|\bparacetamol|\baspirina|\bvitamina|\bjarabe|\btiritas?\b",
        Farmacia,
    ),
    (
        r"\bcerveza|\bcerv\b|\bvino\b|\bvodka|\bwhisky|\bron\b|\bginebra|\bsidra\b|\bcava\b|\bfernet|\bvermut",
        BebidasAlcoolicas,
    ),
    (
        r"\bdetergente|\blejia|\blavandina|\blavavajillas|\bsuavizante|\bfriegasuelos|\blimpiador|\bestropajo|\bbolsas?\s*(de\s*)?basura|\bdesinfect",
        Limpeza,
    ),
    (
        r"\bpapel\s*hig|\bpasta\s*(de\s*)?dient|\bdentifrico|\bcepillo\s*dent|\bchampu|\bshampoo|\bdesodorante|\bgel\s*(de\s*)?(ducha|bano)|\bjabon|\bmaquinilla|\bcompresas?\b|\btampon",
        Higiene,
    ),
    (
        r"\bpapel\s*(de\s*)?(cocina|aluminio)|\bservilleta|\bfilm\b|\bpilas?\b|\bbombilla|\bvela\b|\bcarbon\b",
        Casa,
    ),
    (
        r"\bcuaderno|\blapiz|\bboligrafo|\brotulador|\bfolios",
        Papelaria,
    ),
    (r"\bjuguete|\bmuneca|\bpuzzle|\brompecabezas", Brinquedos),
    (r"\bhelado|\bcongelad|\bpizza\b", Congelados),
    (
        r"\bpan\b|\bbarra\b|\bbaguette|\bbolleria|\bcroissant|\bmedialuna|\bfactura\b|\btortilla\s*(de\s*)?(trigo|maiz)|\bmagdalena",
        Padaria,
    ),
    (
        r"\bgalleta|\bchocolate|\bcaramelo|\bgolosina|\bpatatas\s*fritas|\bpapas\s*fritas|\bsnack|\balfajor|\bbizcocho|\btarta\b|\bturron|\bchicle",
        DocesLanches,
    ),
    (
        r"\bleche\b|\bqueso|\byogur|\bmantequilla|\bmanteca\b|\bnata\b|\bcrema\b|\bhuevos?\b|\bcuajada|\brequeson",
        Laticinios,
    ),
    (
        r"\bpollo|\bternera|\bvacuno|\bcarne\b|\bcerdo|\bjamon|\bchorizo|\bsalchich|\bbacon|\bpanceta|\bpavo\b|\bsalmon|\batun\b|\bmerluza|\bpescado|\bgambas?\b|\blangostino|\bcordero|\bfiambre|\bmortadela|\basado\b|\bmilanesa",
        Carnes,
    ),
    (
        r"\bagua\b|\brefresco|\bgaseosa|\bzumo|\bjugo\b|\bcola\b|\bte\s*frio|\bbebida\s*(isotonica|energetica)|\btonica",
        Bebidas,
    ),
    (
        r"\bplatano|\bbanana|\bmanzana|\bnaranja|\buvas?\b|\bfresa|\bfrutilla|\blimon|\baguacate|\bpalta\b|\btomate|\bpatata|\bpapas?\b|\bcebolla|\bajo\b|\blechuga|\bespinaca|\bzanahoria|\bbrocoli|\bpepino|\bpimiento|\bcalabacin|\bzapallo|\bchampinon|\bmelon|\bsandia|\bpera\b|\bmelocoton|\bdurazno|\bpina\b|\bensalada|\bverdura|\bfruta",
        Hortifruti,
    ),
    (
        r"\barroz|\bjudias?\b|\balubias?\b|\bporotos?\b|\blentejas?\b|\bgarbanzo|\bpasta\b|\bespagueti|\bfideos?\b|\bmacarron|\bharina|\bazucar|\bsal\b|\baceite|\bvinagre|\bcereal|\bavena|\bcafe\b|\byerba\b|\binfusion|\bsopa\b|\bcaldo\b|\bsalsa\b|\bketchup|\bmostaza|\bmayonesa|\bespecias?\b|\bconserva|\bfrutos\s*secos|\balmendra",
        Mercearia,
    ),
];

type Compiled = Vec<(Regex, ItemGroup)>;

fn compile(rules: &[(&str, ItemGroup)]) -> Compiled {
    rules
        .iter()
        .map(|(re, g)| {
            (
                RegexBuilder::new(re)
                    .case_insensitive(true)
                    .build()
                    .expect("regex do item"),
                *g,
            )
        })
        .collect()
}

fn rules(lang: &str) -> Option<&'static Compiled> {
    static EN_RE: OnceLock<Compiled> = OnceLock::new();
    static ES_RE: OnceLock<Compiled> = OnceLock::new();
    match lang {
        "en" => Some(EN_RE.get_or_init(|| compile(EN))),
        "es" => Some(ES_RE.get_or_init(|| compile(ES))),
        _ => None,
    }
}

/// O grupo de um item pela descrição (já sem acentos e em minúsculas), num idioma que não é o
/// português. `None`: nada casou, ou o idioma não tem lista.
pub fn classify(folded: &str, lang: &str) -> Option<ItemGroup> {
    rules(lang)?
        .iter()
        .find(|(re, _)| re.is_match(folded))
        .map(|(_, g)| *g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::text::fold;

    #[test]
    fn english_receipt_lines_with_the_usual_abbreviations() {
        let g = |d: &str| classify(&fold(d), "en");
        assert_eq!(g("GV WHL MLK GAL"), Some(Laticinios));
        assert_eq!(g("ORG BNNS"), Some(Hortifruti));
        assert_eq!(g("GRND BF 80/20 2LB"), Some(Carnes));
        assert_eq!(g("LARGE EGGS 12 CT"), Some(Laticinios));
        assert_eq!(g("WHEAT BREAD"), Some(Padaria));
        assert_eq!(g("HOT DOG BUNS 8CT"), Some(Padaria));
        assert_eq!(g("ICE CREAM VANILLA"), Some(Congelados));
        assert_eq!(g("PEANUT BUTTER CRNCHY"), Some(Mercearia));
        assert_eq!(g("LONG GRAIN RICE 5 LB"), Some(Mercearia));
        assert_eq!(g("COLA 12PK"), Some(Bebidas));
        assert_eq!(g("LAGER BEER 6PK"), Some(BebidasAlcoolicas));
        assert_eq!(g("PAPER TOWELS 6 ROLLS"), Some(Casa));
        assert_eq!(g("TOILET PAPER 12 ROLL"), Some(Higiene));
        assert_eq!(g("DISH SOAP 24 FL OZ"), Some(Limpeza));
        assert_eq!(g("IBUPROFEN 200MG"), Some(Farmacia));
        assert_eq!(g("DOG FOOD 30 LB"), Some(Pet));
        // Depósito de garrafa e taxa de sacola são dinheiro, não produto: Outros.
        assert_eq!(g("BOTTLE DEPOSIT"), Some(Outros));
        assert_eq!(g("BAG FEE"), Some(Outros));
        assert_eq!(g("XYZ 123"), None);
    }

    #[test]
    fn spanish_receipt_lines() {
        let g = |d: &str| classify(&fold(d), "es");
        assert_eq!(g("LECHE ENTERA 1L"), Some(Laticinios));
        assert_eq!(g("PECHUGA DE POLLO"), Some(Carnes));
        assert_eq!(g("PLÁTANO DE CANARIAS"), Some(Hortifruti));
        assert_eq!(g("BARRA DE PAN"), Some(Padaria));
        assert_eq!(g("ACEITE DE OLIVA 1L"), Some(Mercearia));
        assert_eq!(g("CERVEZA LATA 33CL"), Some(BebidasAlcoolicas));
        assert_eq!(g("PAPEL HIGIÉNICO 12 ROLLOS"), Some(Higiene));
        assert_eq!(g("DETERGENTE LÍQUIDO"), Some(Limpeza));
        assert_eq!(g("YERBA MATE 1KG"), Some(Mercearia));
        assert_eq!(classify("leite integral", "fr"), None);
    }
}
