//! Regras prontas para textos comuns de extrato (iFood, Netflix, Uber, postos, farmácias…).
//! Viram regras comuns, editáveis e desativáveis em Regras, criadas uma vez por cofre com
//! prioridade baixa (as do usuário vêm antes). A categoria é achada pelo nome do conjunto
//! inicial; se o usuário apagou ou renomeou, a regra daquela categoria não é criada.

use rusqlite::{Connection, Transaction};

use super::{RuleActions, RuleCondition, RuleField, RuleInput, RuleOp};
use crate::error::Result;
use crate::i18n::t;
use crate::import::text::fold;

/// Ajuste técnico: versão do conjunto de regras prontas já criado neste cofre.
pub const SEEDED_SETTING: &str = "rules.builtin_version";
/// 1: categorias comuns; 2: PIX, TED e DOC viram transferência; 3: restaurantes e os
/// créditos de vale-alimentação e refeição; 4: juros e encargos do cartão.
pub const VERSION: u32 = 4;
/// Depois das regras do usuário (padrão 100).
const PRIORITY: i64 = 900;

/// (nome da regra, categoria-pai, subcategoria, expressão sobre a descrição). A ordem é a
/// prioridade: "UBER EATS" casa delivery antes de transporte.
const BUILTIN: &[Builtin] = &[
    (
        "Delivery (iFood, Rappi, Uber Eats…)",
        "Alimentação",
        "Delivery",
        r"\b(ifood|ifd\*|rappi|uber\s*\*?\s*eats|ze\s+delivery|aiqfome|james\s+delivery)",
    ),
    (
        "Streaming (Netflix, YouTube, Spotify…)",
        "Assinaturas",
        "Streaming",
        r"\b(netflix|spotify|youtube|yt\s*premium|disney\s*(\+|plus)|prime\s*video|amazon\s*prime|hbo|max\.com|globoplay|paramount|deezer|apple\s*tv|crunchyroll|star\s*\+)",
    ),
    (
        "Apps e software (Apple, Google, Microsoft…)",
        "Assinaturas",
        "Software e apps",
        r"\b(apple\.com|itunes|google\s*\*?\s*(storage|one|play|workspace)|microsoft|office\s*365|dropbox|icloud|adobe|openai|chatgpt|anthropic|claude\.ai|notion|canva)",
    ),
    (
        "Telefone e celular",
        "Assinaturas",
        "Telefone",
        r"\b(vivo|claro|tim\s|tim$|oi\s*movel|recarga\s+(vivo|claro|tim|oi))",
    ),
    (
        "Aplicativos de transporte (Uber, 99…)",
        "Transporte",
        "Aplicativos",
        r"\b(uber|99\s*(app|pop|tecnologia|taxi)|cabify|indrive)",
    ),
    (
        "Combustível (postos)",
        "Transporte",
        "Combustível",
        r"\b(posto|auto\s*posto|shell|ipiranga|petrobras|br\s*mania|raizen|ale\s+combust)",
    ),
    (
        "Estacionamento e pedágio",
        "Transporte",
        "Estacionamento",
        r"\b(estacionamento|estapar|zona\s*azul|sem\s*parar|conectcar|veloe|taggy|pedagio)",
    ),
    (
        "Farmácia",
        "Saúde",
        "Farmácia",
        r"\b(drogasil|droga\s*raia|raia\s*drogasil|drogaria|farmacia|pague\s*menos|panvel|drogao|sao\s*joao\s*farm)",
    ),
    (
        "Mercado (supermercados e atacados)",
        "Alimentação",
        "Mercado",
        r"\b(supermerc|hipermerc|carrefour|pao\s*de\s*acucar|assai|atacadao|extra\s*(hiper|super)|big\s*bompreco|zaffari|angeloni|giassi|muffato|condor|sams\s*club|hortifruti)",
    ),
    (
        "Padaria",
        "Alimentação",
        "Padaria",
        r"\b(padaria|panificadora|panific)",
    ),
    (
        RESTAURANTS,
        "Alimentação",
        "Restaurantes",
        r"(\brestaurante|\bchurrascaria|\bpizzaria|\blanchonete|\bbuffet|grill\b|\bsushi|\bbistro|burger\b|burguer\b|\bcantina|\bhamburgueria)",
    ),
    (
        "Academia",
        "Cuidados pessoais",
        "Academia",
        r"\b(smart\s*fit|smartfit|bluefit|bodytech|selfit|academia|gympass|wellhub|totalpass)",
    ),
    (
        "Tarifas bancárias",
        "Impostos e taxas",
        "Tarifas bancárias",
        r"\b(tarifa|cesta\s+de\s+servicos|pacote\s+de\s+servicos|anuidade|tar\s+)",
    ),
    ("IOF", "Impostos e taxas", "IOF", r"\biof\b"),
    (
        CARD_CHARGES,
        "Impostos e taxas",
        "Juros e multas",
        r"\b(juros|encargos?|rotativo|multa\s+(por\s+)?atraso|mora\b|juros\s+de\s+mora)",
    ),
    (
        "Energia elétrica",
        "Moradia",
        "Energia",
        r"\b(enel|cemig|copel|celesc|light\s|cpfl|coelba|celpe|equatorial|energisa|neoenergia|elektro|rge\s)",
    ),
    (
        "Água e saneamento",
        "Moradia",
        "Água",
        r"\b(sabesp|copasa|sanepar|casan|cedae|embasa|compesa|caesb|corsan|saneamento)",
    ),
    (
        "Internet",
        "Moradia",
        "Internet",
        r"\b(net\s*virtua|vivo\s*fibra|oi\s*fibra|tim\s*live|desktop\s*fibra|brisanet|internet)",
    ),
];

const RESTAURANTS: &str = "Restaurantes (restaurante, pizzaria, churrascaria…)";
/// Encargos do cartão e da conta (docs/design/cards.md, fase 3).
const CARD_CHARGES: &str = "Juros e encargos (rotativo, multa, mora)";

/// (nome, categoria-pai, subcategoria, expressão): uma regra pronta.
type Builtin = (&'static str, &'static str, &'static str, &'static str);

const STREAMING_RE: &str = r"\b(netflix|spotify|youtube|yt\s*premium|disney\s*(\+|plus)|prime\s*video|amazon\s*prime|hbo|max\.com|paramount|deezer|apple\s*tv|crunchyroll)";
const SOFTWARE_RE: &str = r"\b(apple\.com|itunes|google\s*\*?\s*(storage|one|play|workspace)|microsoft|office\s*365|dropbox|icloud|adobe|openai|chatgpt|anthropic|claude\.ai|notion|canva)";

/// Estados Unidos (docs/countries.md): redes e empresas comuns nos extratos americanos.
const BUILTIN_US: &[Builtin] = &[
    (
        "Delivery (DoorDash, Uber Eats, Grubhub…)",
        "Alimentação",
        "Delivery",
        r"\b(doordash|uber\s*\*?\s*eats|grubhub|postmates|seamless|caviar)",
    ),
    (
        "Streaming (Netflix, Hulu, Spotify…)",
        "Assinaturas",
        "Streaming",
        r"\b(netflix|spotify|youtube|yt\s*premium|disney\s*(\+|plus)|hulu|prime\s*video|amazon\s*prime|hbo|max\.com|peacock|paramount|apple\s*tv|crunchyroll|sling|fubo|pandora|siriusxm)",
    ),
    (
        "Apps e software (Apple, Google, Microsoft…)",
        "Assinaturas",
        "Software e apps",
        SOFTWARE_RE,
    ),
    (
        "Internet (Xfinity, Spectrum, Fios…)",
        "Moradia",
        "Internet",
        r"\b(comcast|xfinity|spectrum|charter\s*comm|cox\s*comm|optimum|frontier\s*comm|centurylink|google\s*fiber|fios|starlink)",
    ),
    (
        "Celular (Verizon, AT&T, T-Mobile…)",
        "Assinaturas",
        "Telefone",
        r"\b(verizon|at&t|att\*|t-?mobile|mint\s*mobile|cricket\s*wireless|visible|google\s*fi|boost\s*mobile|metro\s*by)",
    ),
    (
        "Aplicativos de transporte (Uber, Lyft)",
        "Transporte",
        "Aplicativos",
        r"\b(uber|lyft)\b",
    ),
    (
        "Combustível (postos)",
        "Transporte",
        "Combustível",
        r"\b(shell|chevron|exxon|mobil|sunoco|marathon\s*petro|speedway|valero|citgo|wawa|sheetz|circle\s*k|arco|quiktrip|racetrac|costco\s*gas|bp#|bp\s+\d)",
    ),
    (
        "Estacionamento e pedágio",
        "Transporte",
        "Estacionamento",
        r"\b(parking|parkmobile|spothero|e-?z\s*pass|fastrak|sunpass|txtag|i-?pass|toll)",
    ),
    (
        "Farmácia (CVS, Walgreens…)",
        "Saúde",
        "Farmácia",
        r"\b(cvs|walgreens|rite\s*aid|duane\s*reade|pharmacy)",
    ),
    (
        "Mercado (Walmart, Costco, Kroger…)",
        "Alimentação",
        "Mercado",
        r"\b(walmart|wal-mart|target\b|costco|kroger|safeway|whole\s*foods|trader\s*joe|aldi|publix|h-?e-?b\b|wegmans|albertsons|food\s*lion|meijer|sprouts|instacart|sam'?s\s*club|winco|hy-?vee)",
    ),
    (
        "Restaurantes e cafés",
        "Alimentação",
        "Restaurantes",
        r"(\brestaurant|\bpizza|\bgrill\b|\bsushi|\bbistro|burger\b|\bdiner\b|starbucks|dunkin|mcdonald|chipotle|wendy'?s|taco\s*bell|chick-?fil-?a|panera|domino'?s|\bkfc\b)",
    ),
    (
        "Academia",
        "Cuidados pessoais",
        "Academia",
        r"\b(planet\s*fitness|la\s*fitness|equinox|24\s*hour\s*fitness|anytime\s*fitness|orangetheory|crunch\s*fitness|life\s*time\s*fitness|ymca|classpass|peloton)",
    ),
    (
        "Tarifas bancárias",
        "Impostos e taxas",
        "Tarifas bancárias",
        r"\b((monthly\s+)?(service|maintenance)\s+(fee|charge)|overdraft\s+fee|atm\s+fee|wire\s+fee|annual\s+fee|foreign\s+transaction\s+fee)",
    ),
    (
        "Energia elétrica",
        "Moradia",
        "Energia",
        r"\b(con\s*ed|coned|pg&e|duke\s*energy|dominion\s*energy|xcel\s*energy|so\s*cal\s*edison|florida\s*power|fpl\b|georgia\s*power|entergy|ameren|dte\s*energy|national\s*grid|eversource|pseg|comed|pepco)",
    ),
    (
        "Água e saneamento",
        "Moradia",
        "Água",
        r"\b(water\s*(dept|department|utility|utilities|district|bill)|american\s*water)",
    ),
];

/// Portugal (docs/countries.md): redes e empresas comuns nos extratos portugueses.
const BUILTIN_PT: &[Builtin] = &[
    (
        "Entregas de comida (Glovo, Uber Eats, Bolt Food)",
        "Alimentação",
        "Delivery",
        r"\b(glovo|uber\s*\*?\s*eats|bolt\s*food|just\s*eat|takeaway\.com)",
    ),
    (
        "Streaming (Netflix, Spotify, RTP Play…)",
        "Assinaturas",
        "Streaming",
        r"\b(netflix|spotify|youtube|yt\s*premium|disney\s*(\+|plus)|prime\s*video|amazon\s*prime|hbo|max\.com|apple\s*tv|rtp\s*play|sport\s*tv|dazn|filmin|skyshowtime)",
    ),
    (
        "Apps e software (Apple, Google, Microsoft…)",
        "Assinaturas",
        "Software e apps",
        SOFTWARE_RE,
    ),
    (
        "Internet e TV (MEO, NOS, Vodafone, DIGI)",
        "Moradia",
        "Internet",
        r"\b(meo\b|nos\s*comunica|nos\s*-|vodafone|nowo|digi\s*portugal|digi\b)",
    ),
    (
        "Telemóvel (UZO, WOO, Moche…)",
        "Assinaturas",
        "Telemóvel",
        r"\b(uzo|woo\b|moche|yorn|lycamobile|carregamento\s+telem)",
    ),
    (
        "Eletricidade (EDP, Endesa, Iberdrola…)",
        "Moradia",
        "Energia",
        r"\b(edp\b|edp\s*comercial|endesa|iberdrola|goldenergy|galp\s*energia|luzboa|coopernico|plenitude|mei\s*energia)",
    ),
    (
        "Gás",
        "Moradia",
        "Gás",
        r"\b(lisboagas|galp\s*gas|floene|gas\s*natural|ruby\s*gas)",
    ),
    (
        "Água e saneamento",
        "Moradia",
        "Água",
        r"\b(epal|aguas\s+d[eo]|smas\b|indaqua|be\s*water|servicos\s+municipalizados)",
    ),
    (
        "Transportes por aplicação (Uber, Bolt…)",
        "Transporte",
        "Aplicativos",
        r"\b(uber|bolt|free\s*now|cabify)\b",
    ),
    (
        "Combustível (postos)",
        "Transporte",
        "Combustível",
        r"\b(galp|repsol|cepsa|moeve|prio\b|shell|bp\b|combustiv)",
    ),
    (
        "Portagens e estacionamento (Via Verde, EMEL…)",
        "Transporte",
        "Estacionamento e portagens",
        r"\b(via\s*verde|portage|brisa|estacionamento|emel\b|saba\b|empark|telpark)",
    ),
    (
        "Transportes públicos (Metro, Carris, CP…)",
        "Transporte",
        "Transportes públicos (passe)",
        r"\b(metro(politano)?\s*(de\s*)?(lisboa|do\s*porto)|carris|comboios\s*de\s*portugal|cp\s*-?\s*comboios|fertagus|transtejo|navegante|andante|stcp)",
    ),
    (
        "Farmácia",
        "Saúde",
        "Farmácia",
        r"\b(farmacia|wells\b|parafarmacia)",
    ),
    (
        "Supermercado (Continente, Pingo Doce, Lidl…)",
        "Alimentação",
        "Mercado",
        r"\b(continente|pingo\s*doce|lidl|aldi|auchan|intermarche|minipreco|mini\s*preco|mercadona|spar\b|e\.?\s*leclerc|meu\s*super|coviran|froiz|recheio|makro)",
    ),
    (
        "Padaria e pastelaria",
        "Alimentação",
        "Padaria",
        r"\b(padaria|pastelaria|panificadora|confeitaria)",
    ),
    (
        "Restaurantes e cafés",
        "Alimentação",
        "Restaurantes",
        r"(\brestaurante|\bpizzaria|\bchurrasqueira|\bmarisqueira|\bcervejaria|\btasca|\bsushi|burger\b|\bcafe\b|\bcafetaria|mcdonald|telepizza|\bkfc\b)",
    ),
    (
        "Ginásio",
        "Cuidados pessoais",
        "Academia",
        r"\b(fitness\s*hut|solinca|holmes\s*place|virgin\s*active|go\s*fit|ginasio|gympass|wellhub)",
    ),
    (
        "Imposto do selo",
        "Impostos e taxas",
        "Imposto do selo",
        r"\b(imposto\s+(do\s+)?selo|imp\.?\s*selo)\b",
    ),
    (
        "Comissões bancárias",
        "Impostos e taxas",
        "Tarifas bancárias",
        r"\b(comissao|manutencao\s+(de\s+)?conta|anuidade)",
    ),
];

/// Espanha (docs/international.md): redes e empresas comuns nos extratos espanhóis.
const BUILTIN_ES: &[Builtin] = &[
    (
        "Entregas de comida (Glovo, Just Eat, Uber Eats)",
        "Alimentação",
        "Delivery",
        r"\b(glovo|just\s*eat|uber\s*\*?\s*eats|deliveroo|telepizza\s*online)",
    ),
    (
        "Streaming (Netflix, Spotify, Movistar Plus+…)",
        "Assinaturas",
        "Streaming",
        r"\b(netflix|spotify|youtube|yt\s*premium|disney\s*(\+|plus)|prime\s*video|amazon\s*prime|hbo|max\.com|apple\s*tv|movistar\s*plus|filmin|dazn|skyshowtime|atresplayer)",
    ),
    (
        "Apps e software (Apple, Google, Microsoft…)",
        "Assinaturas",
        "Software e apps",
        SOFTWARE_RE,
    ),
    (
        "Internet e TV (Movistar, Vodafone, Orange, MásMóvil)",
        "Moradia",
        "Internet",
        r"\b(movistar|telefonica|vodafone|orange\b|masmovil|mas\s*movil|jazztel|digi\b|pepephone|o2\s*espana|lowi|simyo|yoigo|finetwork)",
    ),
    (
        "Eletricidade (Iberdrola, Endesa, Naturgy…)",
        "Moradia",
        "Energia",
        r"\b(iberdrola|endesa|naturgy|repsol\s*(luz|electricidad)|holaluz|octopus\s*energy|totalenergies|curenergia|energia\s*xxi|som\s*energia)",
    ),
    (
        "Gás",
        "Moradia",
        "Gás",
        r"\b(naturgy\s*gas|gas\s*natural|nedgia|butano)",
    ),
    (
        "Água",
        "Moradia",
        "Água",
        r"\b(canal\s+de\s+isabel|aguas\s+de|aigues\s+de|emasesa|emaya|aqualia|agbar|suministro\s+agua)",
    ),
    (
        "Transportes por aplicação (Uber, Cabify, Bolt…)",
        "Transporte",
        "Aplicativos",
        r"\b(uber|cabify|bolt|free\s*now)\b",
    ),
    (
        "Combustível (gasolineiras)",
        "Transporte",
        "Combustível",
        r"\b(repsol|cepsa|moeve|galp|shell|bp\b|ballenoil|plenoil|petronor|avia\b|gasolinera|estacion\s+de\s+servicio)",
    ),
    (
        "Estacionamento e portagens (peajes)",
        "Transporte",
        "Estacionamento",
        r"\b(peaje|autopista|parking|aparcamiento|empark|telpark|saba\b|ser\s+madrid|via\s*t\b)",
    ),
    (
        "Transporte público (Metro, Renfe, EMT…)",
        "Transporte",
        "Transporte público",
        r"\b(metro\s*(de\s*)?(madrid|barcelona|valencia|bilbao|sevilla)|renfe|cercanias|emt\b|tmb\b|abono\s+transporte|alsa\b|tarjeta\s+transporte)",
    ),
    (
        "Farmácia",
        "Saúde",
        "Farmácia",
        r"\b(farmacia|parafarmacia)",
    ),
    (
        "Supermercado (Mercadona, Carrefour, Lidl…)",
        "Alimentação",
        "Mercado",
        r"\b(mercadona|carrefour|lidl|aldi|supermercados?\s+dia|dia\s+retail|alcampo|eroski|consum\b|hipercor|el\s*corte\s*ingles\s*super|bonarea|bon\s*area|caprabo|ahorramas|gadis|froiz|condis|spar\b|makro)",
    ),
    (
        "Padaria e pastelaria",
        "Alimentação",
        "Padaria",
        r"\b(panaderia|pasteleria|horno\s|forn\s|obrador)",
    ),
    (
        "Restaurantes e cafés",
        "Alimentação",
        "Restaurantes",
        r"(\brestaurante|\bpizzeria|\bmarisqueria|\bcerveceria|\btaberna|\bsushi|burger\b|\bcafeteria|\bcafe\b|\bbar\s|mcdonald|telepizza|\bkfc\b|100\s*montaditos|vips\b|foster)",
    ),
    (
        "Ginásio",
        "Cuidados pessoais",
        "Academia",
        r"\b(basic\s*fit|mcfit|altafit|go\s*fit|holmes\s*place|dir\s+fitness|anytime\s*fitness|gimnasio|wellhub|gympass)",
    ),
    (
        "Comissões bancárias",
        "Impostos e taxas",
        "Tarifas bancárias",
        r"\b(comision|comisiones|cuota\s+(de\s+)?(mantenimiento|tarjeta)|mantenimiento\s+cuenta)",
    ),
];

/// Reino Unido (docs/international.md): redes e empresas comuns nos extratos britânicos.
const BUILTIN_GB: &[Builtin] = &[
    (
        "Streaming (Netflix, Spotify, YouTube…)",
        "Assinaturas",
        "Streaming",
        STREAMING_RE,
    ),
    (
        "Apps e software (Apple, Google, Microsoft…)",
        "Assinaturas",
        "Software e apps",
        SOFTWARE_RE,
    ),
    (
        "Supermercado (Tesco, Sainsbury's, Asda…)",
        "Alimentação",
        "Mercado",
        r"\b(tesco|sainsbury|asda|morrisons|aldi|lidl|waitrose|co-?op\b|iceland\b|m&s\s*food|marks\s*(and|&)\s*spencer\s*food|ocado)",
    ),
    (
        "Entregas de comida (Deliveroo, Just Eat, Uber Eats)",
        "Alimentação",
        "Delivery",
        r"\b(deliveroo|just\s*eat|uber\s*\*?\s*eats)",
    ),
    (
        "Restaurantes e cafés (Pret, Costa, Greggs…)",
        "Alimentação",
        "Restaurantes",
        r"\b(pret\s*a\s*manger|costa\s*coffee|starbucks|greggs|nando|wagamama|mcdonald|kfc\b|pizza\s*express|wetherspoon|pub\b|restaurant|cafe\b)",
    ),
    (
        "Combustível (Shell, BP, Esso…)",
        "Transporte",
        "Combustível",
        r"\b(shell|bp\b|esso|texaco|jet\s*petrol|petrol|fuel)",
    ),
    (
        "Transporte público (TfL, National Rail…)",
        "Transporte",
        "Transporte público",
        r"\b(tfl\b|transport\s*for\s*london|national\s*rail|trainline|avanti|lner|gwr\b|northern\s*rail|oyster|stagecoach|first\s*bus|arriva)",
    ),
    (
        "Apps de transporte (Uber, Bolt…)",
        "Transporte",
        "Aplicativos",
        r"\b(uber|bolt|free\s*now|addison\s*lee)\b",
    ),
    (
        "Energia (British Gas, Octopus, EDF…)",
        "Moradia",
        "Energia",
        r"\b(british\s*gas|octopus\s*energy|edf\s*energy|e\.?\s*on\b|ovo\s*energy|scottish\s*power|so\s*energy|eon\s*next)",
    ),
    (
        "Água (Thames Water…)",
        "Moradia",
        "Água",
        r"\b(thames\s*water|severn\s*trent|anglian\s*water|united\s*utilities|yorkshire\s*water|southern\s*water|welsh\s*water)",
    ),
    (
        "Internet e TV (BT, Sky, Virgin Media)",
        "Moradia",
        "Internet",
        r"\b(bt\s*group|british\s*telecom|bt\b|sky\s*(digital|uk|broadband)|virgin\s*media|talktalk|plusnet|hyperoptic)",
    ),
    (
        "Telefone (EE, Vodafone, O2, Three…)",
        "Assinaturas",
        "Telefone",
        r"\b(ee\s*limited|\bee\b|vodafone|o2\b|three\.co|giffgaff|lebara|smarty|id\s*mobile)",
    ),
    (
        "Farmácia (Boots, Superdrug)",
        "Saúde",
        "Farmácia",
        r"\b(boots|superdrug|lloyds\s*pharmacy|pharmacy)",
    ),
    ("Council Tax", "Moradia", "IPTU", r"\bcouncil\s*tax\b"),
];

/// Canadá (docs/international.md): redes e empresas comuns nos extratos canadenses.
const BUILTIN_CA: &[Builtin] = &[
    (
        "Streaming (Netflix, Spotify, YouTube…)",
        "Assinaturas",
        "Streaming",
        STREAMING_RE,
    ),
    (
        "Apps e software (Apple, Google, Microsoft…)",
        "Assinaturas",
        "Software e apps",
        SOFTWARE_RE,
    ),
    (
        "Supermercado (Loblaws, Sobeys, Metro…)",
        "Alimentação",
        "Mercado",
        r"\b(loblaws|no\s*frills|sobeys|metro\s*inc|metro\s*#|safeway|real\s*canadian\s*superstore|superstore|food\s*basics|freshco|costco|walmart|iga\b|save[\s-]*on[\s-]*foods|t\s*&\s*t\s*supermarket)",
    ),
    (
        "Entregas de comida (DoorDash, Uber Eats, SkipTheDishes)",
        "Alimentação",
        "Delivery",
        r"\b(doordash|uber\s*\*?\s*eats|skip\s*the\s*dishes|skipthedishes)",
    ),
    (
        "Restaurantes e cafés (Tim Hortons…)",
        "Alimentação",
        "Restaurantes",
        r"\b(tim\s*hortons|starbucks|mcdonald|a&w\b|subway|harvey|second\s*cup|restaurant|cafe\b)",
    ),
    (
        "Combustível (Petro-Canada, Esso, Shell…)",
        "Transporte",
        "Combustível",
        r"\b(petro[\s-]*canada|esso|shell|husky|ultramar|chevron|pioneer|fuel)",
    ),
    (
        "Transporte público (TTC, Presto, STM…)",
        "Transporte",
        "Transporte público",
        r"\b(ttc\b|presto|go\s*transit|stm\b|translink|compass\s*card|oc\s*transpo|via\s*rail)",
    ),
    (
        "Apps de transporte (Uber, Lyft)",
        "Transporte",
        "Aplicativos",
        r"\b(uber|lyft)\b",
    ),
    (
        "Energia (Hydro, Enbridge…)",
        "Moradia",
        "Energia",
        r"\b(hydro\s*one|toronto\s*hydro|hydro[\s-]*qu[eé]bec|bc\s*hydro|alectra|enbridge|fortis|epcor)",
    ),
    (
        "Internet e telefone (Bell, Rogers, Telus…)",
        "Moradia",
        "Internet",
        r"\b(bell\s*canada|bell\s*mobility|rogers|telus|fido|koodo|virgin\s*plus|freedom\s*mobile|videotron|shaw|cogeco)",
    ),
    (
        "Farmácia (Shoppers Drug Mart, Rexall)",
        "Saúde",
        "Farmácia",
        r"\b(shoppers\s*drug\s*mart|rexall|jean\s*coutu|pharmaprix|london\s*drugs|pharmacy)",
    ),
    (
        "Imposto predial (property tax)",
        "Moradia",
        "IPTU",
        r"\bproperty\s*tax",
    ),
];

/// França (docs/international.md): redes e empresas comuns nos extratos franceses.
const BUILTIN_FR: &[Builtin] = &[
    (
        "Streaming (Netflix, Spotify, YouTube…)",
        "Assinaturas",
        "Streaming",
        STREAMING_RE,
    ),
    (
        "Apps e software (Apple, Google, Microsoft…)",
        "Assinaturas",
        "Software e apps",
        SOFTWARE_RE,
    ),
    (
        "Supermercado (Carrefour, Leclerc, Auchan…)",
        "Alimentação",
        "Mercado",
        r"\b(carrefour|e\.?\s*leclerc|leclerc|auchan|intermarche|super\s*u|hyper\s*u|u\s*express|lidl|aldi|casino|monoprix|franprix|picard|leader\s*price|grand\s*frais)",
    ),
    (
        "Entregas de comida (Uber Eats, Deliveroo, Just Eat)",
        "Alimentação",
        "Delivery",
        r"\b(uber\s*\*?\s*eats|deliveroo|just\s*eat)",
    ),
    (
        "Padaria (boulangerie)",
        "Alimentação",
        "Padaria",
        r"\b(boulangerie|patisserie|paul\b)",
    ),
    (
        "Restaurantes e cafés",
        "Alimentação",
        "Restaurantes",
        r"\b(restaurant|brasserie|bistrot|cafe\b|mcdonald|burger\s*king|quick\b|kfc\b|starbucks)",
    ),
    (
        "Combustível (TotalEnergies, Esso…)",
        "Transporte",
        "Combustível",
        r"\b(total\s*energies|totalenergies|total\s*access|esso|shell|bp\b|avia\b|station[\s-]*service|carburant)",
    ),
    (
        "Transporte público (RATP, SNCF, Navigo…)",
        "Transporte",
        "Transporte público",
        r"\b(ratp|sncf|navigo|tcl\b|rtm\b|tisseo|transilien|ter\b|ouigo)",
    ),
    (
        "Apps de transporte (Uber, Bolt, G7…)",
        "Transporte",
        "Aplicativos",
        r"\b(uber|bolt|heetch|g7\b|free\s*now)\b",
    ),
    (
        "Pedágios (autoroutes)",
        "Transporte",
        "Estacionamento",
        r"\b(peage|autoroute|sanef|aprr|vinci\s*autoroutes|asf\b|parking|indigo)",
    ),
    (
        "Energia (EDF, Engie…)",
        "Moradia",
        "Energia",
        r"\b(edf\b|engie|totalenergies\s*electricite|ekwateur|enercoop|eni\s*gas)",
    ),
    (
        "Internet e telefone (Orange, SFR, Free, Bouygues)",
        "Moradia",
        "Internet",
        r"\b(orange\s*sa|orange\b|sfr\b|free\s*(mobile|telecom)|bouygues\s*telecom|sosh|red\s*by\s*sfr|b\s*and\s*you)",
    ),
    (
        "Farmácia",
        "Saúde",
        "Farmácia",
        r"\b(pharmacie|parapharmacie)",
    ),
    (
        "Taxe foncière e impostos",
        "Moradia",
        "IPTU",
        r"\b(taxe\s*fonciere|dgfip\s*taxe)",
    ),
];

/// Alemanha (docs/international.md): redes e empresas comuns nos extratos alemães.
const BUILTIN_DE: &[Builtin] = &[
    (
        "Streaming (Netflix, Spotify, YouTube…)",
        "Assinaturas",
        "Streaming",
        STREAMING_RE,
    ),
    (
        "Apps e software (Apple, Google, Microsoft…)",
        "Assinaturas",
        "Software e apps",
        SOFTWARE_RE,
    ),
    (
        "Supermercado (Rewe, Edeka, Aldi, Lidl…)",
        "Alimentação",
        "Mercado",
        r"\b(rewe|edeka|aldi|lidl|netto|penny|kaufland|real\s*markt|tegut|globus|norma\b|denns|alnatura)",
    ),
    (
        "Entregas de comida (Lieferando, Wolt, Uber Eats)",
        "Alimentação",
        "Delivery",
        r"\b(lieferando|wolt|uber\s*\*?\s*eats|flink|gorillas)",
    ),
    (
        "Padaria (Bäckerei)",
        "Alimentação",
        "Padaria",
        r"\b(backerei|baeckerei|backhaus|backwerk|kamps)",
    ),
    (
        "Restaurantes e cafés",
        "Alimentação",
        "Restaurantes",
        r"\b(restaurant|gaststatte|imbiss|cafe\b|mcdonald|burger\s*king|vapiano|starbucks)",
    ),
    (
        "Combustível (Aral, Shell, Esso…)",
        "Transporte",
        "Combustível",
        r"\b(aral|shell|esso|jet\s*tankstelle|jet\b|total\s*energies|star\s*tankstelle|agip|tankstelle)",
    ),
    (
        "Transporte público (Deutsche Bahn, BVG, MVV…)",
        "Transporte",
        "Transporte público",
        r"\b(deutsche\s*bahn|db\s*vertrieb|db\s*fernverkehr|bvg\b|mvv\b|hvv\b|vrr\b|rmv\b|deutschlandticket|flixbus)",
    ),
    (
        "Apps de transporte (Uber, Bolt, FreeNow)",
        "Transporte",
        "Aplicativos",
        r"\b(uber|bolt|free\s*now)\b",
    ),
    (
        "Energia (Stadtwerke, E.ON, Vattenfall…)",
        "Moradia",
        "Energia",
        r"\b(stadtwerke|e\.?\s*on\b|vattenfall|enbw|rwe\b|lichtblick|yello\s*strom|eprimo|naturstrom)",
    ),
    (
        "Internet e telefone (Telekom, Vodafone, O2…)",
        "Moradia",
        "Internet",
        r"\b(telekom|vodafone|o2\b|telefonica\s*germany|1&1|1\s*und\s*1|congstar|o2\s*germany)",
    ),
    (
        "Farmácia (Apotheke, dm, Rossmann)",
        "Saúde",
        "Farmácia",
        r"\b(apotheke|dm[\s-]*drogerie|rossmann|mueller\s*drogerie)",
    ),
    (
        "Rundfunkbeitrag e Grundsteuer",
        "Moradia",
        "IPTU",
        r"\b(grundsteuer)",
    ),
];

/// Argentina (docs/international.md): redes e empresas comuns nos extratos argentinos.
const BUILTIN_AR: &[Builtin] = &[
    (
        "Streaming (Netflix, Spotify, YouTube…)",
        "Assinaturas",
        "Streaming",
        STREAMING_RE,
    ),
    (
        "Apps e software (Apple, Google, Microsoft…)",
        "Assinaturas",
        "Software e apps",
        SOFTWARE_RE,
    ),
    (
        "Supermercado (Coto, Carrefour, Día…)",
        "Alimentação",
        "Mercado",
        r"\b(coto\b|carrefour|supermercados?\s+dia|jumbo|disco\b|vea\b|la\s*anonima|changomas|chango\s*mas|walmart|makro|maxiconsumo)",
    ),
    (
        "Entregas de comida (PedidosYa, Rappi)",
        "Alimentação",
        "Delivery",
        r"\b(pedidos\s*ya|pedidosya|rappi)",
    ),
    (
        "Restaurantes e cafés",
        "Alimentação",
        "Restaurantes",
        r"\b(restaurante|parrilla|pizzeria|cafe\b|mcdonald|burger\s*king|havanna|starbucks)",
    ),
    (
        "Combustível (YPF, Shell, Axion…)",
        "Transporte",
        "Combustível",
        r"\b(ypf|shell|axion|puma\s*energy|gulf\b|estacion\s+de\s+servicio)",
    ),
    (
        "Transporte público (SUBE…)",
        "Transporte",
        "Transporte público",
        r"\b(sube\b|carga\s*sube|subte|trenes\s*argentinos)",
    ),
    (
        "Apps de transporte (Uber, Cabify, DiDi)",
        "Transporte",
        "Aplicativos",
        r"\b(uber|cabify|didi)\b",
    ),
    (
        "Energia e gás (Edenor, Edesur, Metrogas…)",
        "Moradia",
        "Energia",
        r"\b(edenor|edesur|epec\b|edelap|metrogas|naturgy\s*ban|camuzzi|ecogas)",
    ),
    (
        "Internet e telefone (Personal, Movistar, Claro…)",
        "Moradia",
        "Internet",
        r"\b(personal\b|telecom\s*argentina|movistar|claro\b|fibertel|telecentro|flow\b)",
    ),
    ("Farmácia", "Saúde", "Farmácia", r"\b(farmacia|farmacity)"),
];

/// Chile (docs/international.md): redes e empresas comuns nos extratos chilenos.
const BUILTIN_CL: &[Builtin] = &[
    (
        "Streaming (Netflix, Spotify, YouTube…)",
        "Assinaturas",
        "Streaming",
        STREAMING_RE,
    ),
    (
        "Apps e software (Apple, Google, Microsoft…)",
        "Assinaturas",
        "Software e apps",
        SOFTWARE_RE,
    ),
    (
        "Supermercado (Líder, Jumbo, Unimarc…)",
        "Alimentação",
        "Mercado",
        r"\b(lider\b|jumbo|unimarc|santa\s*isabel|tottus|acuenta|mayorista\s*10|alvi\b|walmart)",
    ),
    (
        "Entregas de comida (PedidosYa, Rappi, Uber Eats)",
        "Alimentação",
        "Delivery",
        r"\b(pedidos\s*ya|pedidosya|rappi|uber\s*\*?\s*eats|justo\b)",
    ),
    (
        "Restaurantes e cafés",
        "Alimentação",
        "Restaurantes",
        r"\b(restaurante|picada|cafe\b|mcdonald|burger\s*king|starbucks|juan\s*maestro|doggis)",
    ),
    (
        "Combustível (Copec, Shell, Aramco…)",
        "Transporte",
        "Combustível",
        r"\b(copec|shell|aramco|petrobras|enex|estacion\s+de\s+servicio)",
    ),
    (
        "Transporte público (bip!, Metro, Red…)",
        "Transporte",
        "Transporte público",
        r"\b(bip\b|tarjeta\s*bip|metro\s*de\s*santiago|red\s*movilidad|efe\b)",
    ),
    (
        "Apps de transporte (Uber, Cabify, DiDi)",
        "Transporte",
        "Aplicativos",
        r"\b(uber|cabify|didi)\b",
    ),
    (
        "Energia e gás (Enel, CGE, Metrogas…)",
        "Moradia",
        "Energia",
        r"\b(enel\b|cge\b|chilquinta|saesa|metrogas|abastible|lipigas|gasco)",
    ),
    (
        "Água (Aguas Andinas…)",
        "Moradia",
        "Água",
        r"\b(aguas\s*andinas|esval|essbio|nuevosur|aguas\s*del\s*valle)",
    ),
    (
        "Internet e telefone (Entel, Movistar, WOM…)",
        "Moradia",
        "Internet",
        r"\b(entel|movistar|wom\b|claro\b|vtr\b|mundo\s*telecomunicaciones|gtd\b)",
    ),
    (
        "Farmácia (Cruz Verde, Salcobrand, Ahumada)",
        "Saúde",
        "Farmácia",
        r"\b(cruz\s*verde|salcobrand|ahumada|farmacia)",
    ),
];

/// Outros países: só o que é igual no mundo todo.
const BUILTIN_GENERIC: &[Builtin] = &[
    (
        "Delivery (Uber Eats, Glovo, Deliveroo…)",
        "Alimentação",
        "Delivery",
        r"\b(uber\s*\*?\s*eats|glovo|deliveroo|doordash|rappi|just\s*eat|wolt|foodpanda)",
    ),
    (
        "Streaming (Netflix, YouTube, Spotify…)",
        "Assinaturas",
        "Streaming",
        STREAMING_RE,
    ),
    (
        "Apps e software (Apple, Google, Microsoft…)",
        "Assinaturas",
        "Software e apps",
        SOFTWARE_RE,
    ),
    (
        "Aplicativos de transporte (Uber, Bolt, Lyft…)",
        "Transporte",
        "Aplicativos",
        r"\b(uber|bolt|lyft|cabify|didi|grab)\b",
    ),
];

/// As regras prontas do país da família.
fn list_for(country: &str) -> &'static [Builtin] {
    match country {
        "BR" => BUILTIN,
        "US" => BUILTIN_US,
        "PT" => BUILTIN_PT,
        "ES" => BUILTIN_ES,
        "GB" => BUILTIN_GB,
        "CA" => BUILTIN_CA,
        "FR" => BUILTIN_FR,
        "DE" => BUILTIN_DE,
        "AR" => BUILTIN_AR,
        "CL" => BUILTIN_CL,
        _ => BUILTIN_GENERIC,
    }
}

/// Regra de tipo do país: os meios que movimentam dinheiro entre contas viram transferência.
fn transfer_for(country: &str) -> Option<(&'static str, &'static str)> {
    match country {
        "BR" => Some((TRANSFER_RULE, TRANSFER_RE)),
        "US" => Some((
            "Zelle, Venmo e transferências são transferências",
            r"\b(zelle|venmo|cash\s*app|ach\s+(transfer|debit|credit)|wire\s+transfer|online\s+transfer)\b",
        )),
        "PT" => Some((
            "MB WAY e transferências SEPA são transferências",
            r"\b(mb\s*way|mbway|trf\.?\s|transf\.?\s|transferencia|sepa)\b",
        )),
        "GB" => Some((
            "Faster Payments e transferências são transferências",
            r"\b(faster\s*payment|fps\b|bank\s*transfer|standing\s*order|transfer\s+(to|from))\b",
        )),
        "CA" => Some((
            "Interac e-Transfer e transferências são transferências",
            r"\b(interac\s*e[\s-]*transfer|e[\s-]*transfer|eft\b|transfer\s+(to|from))\b",
        )),
        "FR" => Some((
            "Virements, Wero e Lydia são transferências",
            r"\b(virement|vir\s+sepa|vir\.?\s|wero|lydia|paylib)\b",
        )),
        "DE" => Some((
            "Überweisungen e PayPal entre amigos são transferências",
            r"\b(uberweisung|ueberweisung|dauerauftrag|sepa[\s-]*uberweisung|umbuchung)\b",
        )),
        "AR" => Some((
            "Transferências (CBU, CVU, alias) são transferências",
            r"\b(transferencia|transf\.?\s|cbu\b|cvu\b|debin\b)\b",
        )),
        "CL" => Some((
            "Transferências (TEF) são transferências",
            r"\b(transferencia|transf\.?\s|tef\b|traspaso)\b",
        )),
        "ES" => Some((
            "Bizum e transferências SEPA são transferências",
            r"\b(bizum|transferencia|transf\.?\s|traspaso|sepa)\b",
        )),
        _ => None,
    }
}

/// Créditos do empregador no cartão de benefício (Flash: "Depósito transferido").
pub const BENEFIT_RULE: &str = "Créditos de vale-alimentação e refeição";
const BENEFIT_RE: &str = r"\bdeposito\s+transferido\b";

/// A categoria de receita `pai › filha` pelos nomes em pt-BR (vale com os nomes em qualquer
/// idioma, ver `categories::br_categories`).
fn income_category(conn: &Connection, parent: &str, child: &str) -> Result<Option<String>> {
    crate::categories::find_br_child(conn, crate::categories::CategoryKind::Income, parent, child)
}

fn seed_benefit_rule(tx: &Transaction<'_>) -> Result<u32> {
    benefit_rule(
        tx,
        BENEFIT_RULE,
        BENEFIT_RE,
        ("Benefícios", "Vale-alimentação e refeição"),
        BUILTIN.len(),
    )
}

fn benefit_rule(
    tx: &Transaction<'_>,
    name: &str,
    re: &str,
    (parent, child): (&str, &str),
    after: usize,
) -> Result<u32> {
    let exists: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM rule WHERE name = ?1)",
        [name],
        |r| r.get(0),
    )?;
    if exists {
        return Ok(0);
    }
    let Some(cat) = income_category(tx, parent, child)? else {
        return Ok(0);
    };
    super::create(
        tx,
        &RuleInput {
            name: name.to_string(),
            priority: Some(PRIORITY + after as i64 + 1),
            active: Some(true),
            match_all: Some(true),
            conditions: vec![
                RuleCondition {
                    field: RuleField::Description,
                    op: RuleOp::Regex,
                    value: serde_json::Value::String(re.to_string()),
                    value2: None,
                },
                RuleCondition {
                    field: RuleField::Amount,
                    op: RuleOp::Gte,
                    value: serde_json::json!(1),
                    value2: None,
                },
            ],
            actions: RuleActions {
                category_id: Some(cat),
                ..RuleActions::default()
            },
        },
    )?;
    Ok(1)
}

/// Regra pronta de tipo: PIX, TED e DOC movimentam dinheiro entre contas (da família ou de
/// terceiros); viram transferência. Com categoria, a de uma perna só continua contando no
/// fluxo de caixa pela categoria.
pub const TRANSFER_RULE: &str = "PIX, TED e DOC são transferências";
pub const TRANSFER_RE: &str = r"\b(pix|ted|doc)\b";

fn seed_transfer_rule(tx: &Transaction<'_>) -> Result<u32> {
    transfer_rule(tx, TRANSFER_RULE, TRANSFER_RE, BUILTIN.len())
}

fn transfer_rule(tx: &Transaction<'_>, name: &str, re: &str, after: usize) -> Result<u32> {
    let exists: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM rule WHERE name = ?1)",
        [name],
        |r| r.get(0),
    )?;
    if exists {
        return Ok(0);
    }
    super::create(
        tx,
        &RuleInput {
            name: name.to_string(),
            priority: Some(PRIORITY + after as i64),
            active: Some(true),
            match_all: Some(true),
            conditions: vec![RuleCondition {
                field: RuleField::Description,
                op: RuleOp::Regex,
                value: serde_json::Value::String(re.to_string()),
                value2: None,
            }],
            actions: RuleActions {
                kind: Some(crate::transactions::TxnKind::Transfer),
                ..RuleActions::default()
            },
        },
    )?;
    Ok(1)
}

/// Lançamentos já gravados com PIX, TED ou DOC na descrição (receitas e despesas soltas,
/// sem divisão nem parcelamento) passam a transferência; a categoria fica. Devolve quantos.
pub fn mark_existing_transfers(tx: &Transaction<'_>) -> Result<u32> {
    let re = regex::RegexBuilder::new(TRANSFER_RE)
        .case_insensitive(true)
        .build()
        .map_err(|e| crate::error::Error::invalid(e.to_string()))?;
    let ids: Vec<(String, String)> = {
        let mut stmt = tx.prepare(
            "SELECT t.id, t.description FROM txn t
              WHERE t.kind IN ('income', 'expense') AND t.transfer_id IS NULL
                AND t.installment_plan_id IS NULL
                AND NOT EXISTS (SELECT 1 FROM txn_split s WHERE s.txn_id = t.id)",
        )?;
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?
    };
    let now = crate::time::now_utc();
    let mut n = 0;
    for (id, desc) in ids {
        if re.is_match(&fold(&desc)) {
            tx.execute(
                "UPDATE txn SET kind = 'transfer', updated_at = ?2 WHERE id = ?1",
                rusqlite::params![id, now],
            )?;
            n += 1;
        }
    }
    Ok(n)
}

/// A categoria `pai › filha` pelos nomes em pt-BR (sem acentos nem maiúsculas; vale com os
/// nomes em qualquer idioma, ver `categories::br_categories`), só de despesa.
fn category(conn: &Connection, parent: &str, child: &str) -> Result<Option<String>> {
    crate::categories::find_br_child(
        conn,
        crate::categories::CategoryKind::Expense,
        parent,
        child,
    )
}

/// Cria a regra pronta `i` de `BUILTIN`, se ainda não existe e a categoria existe.
fn create_one(tx: &Transaction<'_>, i: usize) -> Result<u32> {
    create_from(tx, BUILTIN, i)
}

/// Cria a regra pronta `i` da lista, se ainda não existe e a categoria existe.
fn create_from(tx: &Transaction<'_>, list: &[Builtin], i: usize) -> Result<u32> {
    let (name, parent, child, re) = list[i];
    let exists: bool = tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM rule WHERE name = ?1)",
        [name],
        |r| r.get(0),
    )?;
    if exists {
        return Ok(0);
    }
    let Some(cat) = category(tx, parent, child)? else {
        return Ok(0);
    };
    super::create(
        tx,
        &RuleInput {
            name: name.to_string(),
            priority: Some(PRIORITY + i as i64),
            active: Some(true),
            match_all: Some(true),
            conditions: vec![
                RuleCondition {
                    field: RuleField::Description,
                    op: RuleOp::Regex,
                    value: serde_json::Value::String(re.to_string()),
                    value2: None,
                },
                // Só saídas: estorno ou cashback não vira despesa.
                RuleCondition {
                    field: RuleField::Amount,
                    op: RuleOp::Lte,
                    value: serde_json::json!(-1),
                    value2: None,
                },
            ],
            actions: RuleActions {
                category_id: Some(cat),
                ..RuleActions::default()
            },
        },
    )?;
    Ok(1)
}

/// Cria as regras prontas que ainda não existem (pelo nome). Devolve quantas criou.
pub fn seed(tx: &Transaction<'_>) -> Result<u32> {
    seed_for(tx, "BR")
}

/// Cria as regras prontas do país da família (docs/countries.md).
pub fn seed_for(tx: &Transaction<'_>, country: &str) -> Result<u32> {
    let list = list_for(country);
    let mut n = 0;
    for i in 0..list.len() {
        n += create_from(tx, list, i)?;
    }
    if let Some((name, re)) = transfer_for(country) {
        n += transfer_rule(tx, name, re, list.len())?;
    }
    n += match country {
        "BR" => seed_benefit_rule(tx)?,
        "PT" => benefit_rule(
            tx,
            "Créditos do subsídio de alimentação",
            r"\b(carregamento|credito)\b.*\b(refeicao|alimentacao|edenred|coverflex|pluxee)\b",
            ("Benefícios", "Subsídio de alimentação"),
            list.len(),
        )?,
        _ => 0,
    };
    Ok(n)
}

/// Quem cria as regras prontas no histórico ("Sistema · regras prontas").
fn author() -> crate::changes::Author {
    crate::changes::Author::system(t("nucleo-lancamentos-autor-regras-prontas"))
}

/// Ao abrir o cofre: cria as regras prontas uma vez (e as novas de versões futuras).
pub fn seed_once(conn: &mut Connection) -> Result<()> {
    let done: u32 = crate::settings::get_or(conn, SEEDED_SETTING, 0)?;
    if done >= VERSION {
        return Ok(());
    }
    // As versões 1 a 4 são do conjunto do Brasil; cofres de outros países nascem com o deles.
    if !crate::country::is_brazil(conn) {
        return mark_seeded(conn);
    }
    if done < 1 {
        crate::changes::record(
            conn,
            &author(),
            &t("nucleo-lancamentos-historico-regras-prontas"),
            seed,
        )?;
    }
    if done < 2 {
        // Desfazível pelo histórico, como qualquer alteração.
        crate::changes::record(
            conn,
            &author(),
            &t("nucleo-lancamentos-historico-regras-prontas-pix"),
            |tx| {
                seed_transfer_rule(tx)?;
                mark_existing_transfers(tx)
            },
        )?;
    }
    if (1..3).contains(&done) {
        // Só as regras novas (as apagadas pelo usuário não voltam).
        crate::changes::record(
            conn,
            &author(),
            &t("nucleo-lancamentos-historico-regras-prontas-restaurantes"),
            |tx| {
                let i = BUILTIN
                    .iter()
                    .position(|r| r.0 == RESTAURANTS)
                    .unwrap_or_default();
                Ok(create_one(tx, i)? + seed_benefit_rule(tx)?)
            },
        )?;
    }
    if (1..4).contains(&done) {
        crate::changes::record(
            conn,
            &author(),
            &t("nucleo-lancamentos-historico-regras-prontas-encargos"),
            |tx| {
                let i = BUILTIN
                    .iter()
                    .position(|r| r.0 == CARD_CHARGES)
                    .unwrap_or_default();
                create_one(tx, i)
            },
        )?;
    }
    mark_seeded(conn)
}

/// O cofre já tem as regras prontas desta versão (criadas junto com ele).
pub fn mark_seeded(conn: &Connection) -> Result<()> {
    crate::settings::set(conn, SEEDED_SETTING, &VERSION)
}

/// Teste: as regras prontas do país compilam e casam textos típicos dos extratos de lá.
#[cfg(test)]
pub(crate) fn check_list(country: &str) {
    use regex::RegexBuilder;
    let list = list_for(country);
    let first = |text: &str| -> Option<&'static str> {
        let t = fold(text);
        list.iter()
            .find(|(_, _, _, re)| {
                RegexBuilder::new(re)
                    .case_insensitive(true)
                    .build()
                    .unwrap()
                    .is_match(&t)
            })
            .map(|(_, _, child, _)| *child)
    };
    for (n, _, _, re) in list {
        assert!(RegexBuilder::new(re).build().is_ok(), "{country}: {n}");
    }
    let transfer = |text: &str| {
        let (_, re) = transfer_for(country).unwrap();
        RegexBuilder::new(re)
            .case_insensitive(true)
            .build()
            .unwrap()
            .is_match(&fold(text))
    };
    let cases: &[(&str, &str)] = match country {
        "US" => &[
            ("DOORDASH*CHIPOTLE", "Delivery"),
            ("WALMART SUPERCENTER #1234", "Mercado"),
            ("SHELL OIL 57444", "Combustível"),
            ("COMCAST XFINITY", "Internet"),
            ("VERIZON WIRELESS", "Telefone"),
            ("UBER *TRIP", "Aplicativos"),
            ("STARBUCKS STORE 123", "Restaurantes"),
            ("CVS/PHARMACY #0042", "Farmácia"),
            ("MONTHLY SERVICE FEE", "Tarifas bancárias"),
        ],
        "PT" => &[
            ("CONTINENTE MATOSINHOS", "Mercado"),
            ("COMPRA PINGO DOCE LISBOA", "Mercado"),
            ("VIA VERDE PORTUGAL", "Estacionamento e portagens"),
            ("GLOVO*PEDIDO", "Delivery"),
            ("EDP COMERCIAL", "Energia"),
            ("GALP ENERGIA SA", "Energia"),
            ("GALP OEIRAS", "Combustível"),
            ("MEO SA", "Internet"),
            ("NAVEGANTE METROPOLITANO", "Transportes públicos (passe)"),
            ("IMPOSTO DO SELO", "Imposto do selo"),
        ],
        "GB" => &[
            ("TESCO STORES 1234", "Mercado"),
            ("DELIVEROO", "Delivery"),
            ("TFL TRAVEL CH", "Transporte público"),
            ("OCTOPUS ENERGY", "Energia"),
            ("COUNCIL TAX", "IPTU"),
        ],
        "CA" => &[
            ("LOBLAWS #123", "Mercado"),
            ("TIM HORTONS #1", "Restaurantes"),
            ("PETRO-CANADA 555", "Combustível"),
            ("PRESTO FARE", "Transporte público"),
            ("ROGERS WIRELESS", "Internet"),
        ],
        "FR" => &[
            ("CARREFOUR MARKET PARIS", "Mercado"),
            ("BOULANGERIE DU COIN", "Padaria"),
            ("RATP NAVIGO", "Transporte público"),
            ("EDF CLIENTS", "Energia"),
            ("PHARMACIE CENTRALE", "Farmácia"),
        ],
        "DE" => &[
            ("REWE SAGT DANKE", "Mercado"),
            ("LIEFERANDO.DE", "Delivery"),
            ("DEUTSCHE BAHN", "Transporte público"),
            ("STADTWERKE MUENCHEN", "Energia"),
            ("DM-DROGERIE MARKT", "Farmácia"),
        ],
        "AR" => &[
            ("COTO CICSA", "Mercado"),
            ("PEDIDOSYA", "Delivery"),
            ("YPF 1234", "Combustível"),
            ("CARGA SUBE", "Transporte público"),
            ("EDENOR", "Energia"),
        ],
        "CL" => &[
            ("LIDER EXPRESS", "Mercado"),
            ("COPEC 123", "Combustível"),
            ("ENEL DISTRIBUCION", "Energia"),
            ("AGUAS ANDINAS", "Água"),
            ("CRUZ VERDE", "Farmácia"),
        ],
        "ES" => &[
            ("MERCADONA VALENCIA", "Mercado"),
            ("COMPRA EN CARREFOUR MADRID", "Mercado"),
            ("GLOVO*PEDIDO", "Delivery"),
            ("IBERDROLA CLIENTES", "Energia"),
            ("REPSOL E.S. 1234", "Combustível"),
            ("MOVISTAR FUSION", "Internet"),
            ("RENFE CERCANIAS", "Transporte público"),
            ("CABIFY*VIAJE", "Aplicativos"),
            ("COMISION MANTENIMIENTO", "Tarifas bancárias"),
        ],
        "BR" => &[
            ("IFOOD *RESTAURANTE", "Delivery"),
            ("POSTO IPIRANGA", "Combustível"),
            ("ENCARGOS ROTATIVO", "Juros e multas"),
            ("MULTA POR ATRASO", "Juros e multas"),
        ],
        _ => &[("NETFLIX.COM", "Streaming"), ("UBER *TRIP", "Aplicativos")],
    };
    for (text, want) in cases {
        assert_eq!(first(text), Some(*want), "{country}: {text}");
    }
    match country {
        "US" => {
            assert!(transfer("ZELLE PAYMENT TO JOHN"));
            assert!(!transfer("ZELLWEGER"));
        }
        "PT" => {
            assert!(transfer("MB WAY P/ JOAO"));
            assert!(transfer("TRF P/ MARIA"));
        }
        "GB" => {
            assert!(transfer("FASTER PAYMENT TO JOHN"));
            assert!(!transfer("TESCO"));
        }
        "CA" => {
            assert!(transfer("INTERAC E-TRANSFER"));
            assert!(!transfer("TIM HORTONS"));
        }
        "FR" => {
            assert!(transfer("VIR SEPA LOYER"));
            assert!(!transfer("CARREFOUR"));
        }
        "DE" => {
            assert!(transfer("UEBERWEISUNG MIETE"));
            assert!(!transfer("REWE"));
        }
        "AR" => {
            assert!(transfer("TRANSFERENCIA A CVU"));
            assert!(!transfer("COTO"));
        }
        "CL" => {
            assert!(transfer("TRANSFERENCIA A TERCEROS"));
            assert!(!transfer("LIDER"));
        }
        "ES" => {
            assert!(transfer("BIZUM ENVIADO A LUCIA"));
            assert!(transfer("TRANSFERENCIA A FAVOR DE"));
            assert!(!transfer("BIZUMANIA"));
        }
        "BR" => assert!(transfer("PIX ENVIADO")),
        _ => assert!(transfer_for(country).is_none()),
    }
}

#[cfg(test)]
mod tests {
    use regex::RegexBuilder;

    use super::BUILTIN;

    /// The first one that matches (the order is the priority).
    fn hits(text: &str) -> Vec<&'static str> {
        BUILTIN
            .iter()
            .filter(|(_, _, _, re)| {
                RegexBuilder::new(re)
                    .case_insensitive(true)
                    .build()
                    .unwrap()
                    .is_match(text)
            })
            .map(|(n, ..)| *n)
            .take(1)
            .collect()
    }

    #[test]
    fn common_statement_texts() {
        assert_eq!(
            hits("IFOOD *RESTAURANTE"),
            ["Delivery (iFood, Rappi, Uber Eats…)"]
        );
        assert_eq!(
            hits("UBER *EATS PENDING"),
            ["Delivery (iFood, Rappi, Uber Eats…)"]
        );
        assert_eq!(
            hits("UBER *TRIP HELP.UBER.COM"),
            ["Aplicativos de transporte (Uber, 99…)"]
        );
        assert_eq!(
            hits("NETFLIX.COM"),
            ["Streaming (Netflix, YouTube, Spotify…)"]
        );
        assert_eq!(
            hits("GOOGLE YOUTUBE PREMIUM"),
            ["Streaming (Netflix, YouTube, Spotify…)"]
        );
        assert_eq!(
            hits("DL*GOOGLE YOUTUBE"),
            ["Streaming (Netflix, YouTube, Spotify…)"]
        );
        assert_eq!(
            hits("Spotify P0123"),
            ["Streaming (Netflix, YouTube, Spotify…)"]
        );
        assert_eq!(hits("RAIA DROGASIL S/A"), ["Farmácia"]);
        assert_eq!(hits("AUTO POSTO SAO JORGE"), ["Combustível (postos)"]);
        assert_eq!(hits("TARIFA PACOTE DE SERVICOS"), ["Tarifas bancárias"]);
        assert_eq!(hits("IOF COMPRA INTERNACIONAL"), ["IOF"]);
        assert_eq!(hits("JUROS DO ROTATIVO"), [super::CARD_CHARGES]);
        assert!(hits("MORADIA ALUGUEL").is_empty());
        assert!(hits("PIX RECEBIDO CARLA").is_empty());
        assert!(hits("LIVRARIA CULTURA VIVA").is_empty());
        assert!(hits("PAGAMENTO FATURA").is_empty());
    }
}
