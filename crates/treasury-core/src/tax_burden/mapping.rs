//! Categoria do app → grupo de imposto. Os grupos são os mesmos em todos os países (cada
//! país dá as alíquotas deles no `dataset`); o mapeamento padrão vem pelo nome das
//! categorias do conjunto inicial (sem acentos nem maiúsculas) e o usuário pode trocar o
//! grupo de qualquer categoria ou fixar uma alíquota própria (`tax_category`, V109).
//! Subcategoria sem mapeamento herda o do grupo pai.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ids::Id;
use crate::import::text::fold;

/// O que a categoria representa para a estimativa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum GroupKind {
    /// Consumo: imposto embutido no preço, pela alíquota do grupo.
    Consumption,
    /// Imposto pago direto (IR, IOF, IPTU, IPVA…): conta pelo valor lançado.
    DirectTax,
    /// Fora do consumo (dívidas, investimentos, transferências a pessoas): não estima.
    NotConsumption,
}

/// Grupos de imposto direto: o que foi lançado na categoria é imposto pago.
pub const DIRECT: &[(&str, &str)] = &[
    ("tax_income", "Imposto de renda pago"),
    ("tax_iof", "IOF pago"),
    ("tax_property_iptu", "IPTU"),
    ("tax_property_ipva", "IPVA e licenciamento"),
    ("tax_other", "Outros tributos pagos"),
];

/// Grupos fora do consumo.
pub const NOT_CONSUMPTION: &[(&str, &str)] = &[
    ("financial", "Financeiro (dívidas, juros, investimentos)"),
    (
        "transfers",
        "Transferências a pessoas (pensão, mesada, doações)",
    ),
];

pub fn kind_of(group: &str) -> GroupKind {
    if DIRECT.iter().any(|(k, _)| *k == group) {
        GroupKind::DirectTax
    } else if NOT_CONSUMPTION.iter().any(|(k, _)| *k == group) {
        GroupKind::NotConsumption
    } else {
        GroupKind::Consumption
    }
}

/// Mapeamento padrão pelo nome da categoria do conjunto inicial (`categories::EXPENSES`).
/// Grupos sem dado confiável no país (aluguel, plano de saúde…) ficam como "sem dados": o
/// gasto aparece, mas não vira imposto estimado.
const DEFAULTS: &[(&str, &str)] = &[
    // Moradia
    ("Moradia", "contas_diversas"),
    ("Aluguel", "aluguel"),
    ("Condomínio", "condominio"),
    ("Energia", "energia_eletrica"),
    ("Água", "agua_esgoto"),
    ("Gás", "glp"),
    ("Internet", "telefonia_internet"),
    ("IPTU", "tax_property_iptu"),
    ("Manutenção", "materiais_construcao"),
    ("Diarista e serviços domésticos", "servicos_domesticos"),
    // Alimentação
    ("Alimentação", "mercado"),
    ("Mercado", "mercado"),
    ("Restaurantes", "restaurantes"),
    ("Delivery", "restaurantes"),
    ("Padaria", "alimentos_basicos"),
    // Transporte
    ("Transporte", "compras_diversas"),
    ("Combustível", "gasolina"),
    ("Transporte público", "transporte_publico"),
    ("Aplicativos", "transporte_aplicativo"),
    ("Estacionamento", "estacionamento"),
    ("Manutenção do veículo", "pecas_manutencao_veiculo"),
    ("IPVA e licenciamento", "tax_property_ipva"),
    ("Seguro do veículo", "seguros"),
    // Saúde
    ("Saúde", "saude_plano_consulta"),
    ("Plano de saúde", "saude_plano_consulta"),
    ("Farmácia", "medicamentos"),
    ("Consultas", "saude_plano_consulta"),
    ("Pediatra", "saude_plano_consulta"),
    ("Exames", "saude_plano_consulta"),
    ("Dentista", "saude_plano_consulta"),
    ("Terapias e psicólogo", "saude_plano_consulta"),
    ("Vacinas", "saude_plano_consulta"),
    ("Óculos e lentes", "optica"),
    ("Itens de saúde e primeiros socorros", "medicamentos"),
    // Educação
    ("Educação", "educacao_mensalidade"),
    ("Escola", "educacao_mensalidade"),
    ("Creche", "educacao_mensalidade"),
    ("Material escolar", "material_escolar"),
    ("Uniforme", "vestuario"),
    ("Transporte escolar", "transporte_publico"),
    ("Cantina e lanche escolar", "restaurantes"),
    ("Faculdade", "educacao_mensalidade"),
    ("Idiomas", "educacao_mensalidade"),
    ("Cursos", "educacao_mensalidade"),
    ("Livros e material", "educacao_livros"),
    // Lazer
    ("Lazer", "lazer"),
    ("Viagens", "hotel_turismo"),
    ("Passeios", "lazer"),
    ("Cinema e shows", "lazer"),
    ("Hobbies", "lazer"),
    // Compras
    ("Compras", "compras_diversas"),
    ("Vestuário", "vestuario"),
    ("Eletrônicos", "eletronicos_nacionais"),
    ("Casa e decoração", "moveis"),
    // Cuidados pessoais
    ("Cuidados pessoais", "higiene_pessoal"),
    ("Cabelo e estética", "servicos_pessoais"),
    ("Academia", "servicos_pessoais"),
    ("Cosméticos", "cosmeticos_perfumes"),
    // Assinaturas
    ("Assinaturas", "tv_assinatura_streaming"),
    ("Streaming", "tv_assinatura_streaming"),
    ("Software e apps", "tv_assinatura_streaming"),
    ("Telefone", "telefonia_internet"),
    // Impostos e taxas
    ("Impostos e taxas", "tax_other"),
    ("Tarifas bancárias", "tarifas_bancarias"),
    ("Juros e multas", "financial"),
    ("IOF", "tax_iof"),
    ("Imposto de renda", "tax_income"),
    // Seguros
    ("Seguros", "seguros"),
    ("Seguro de vida", "seguros"),
    ("Seguro residencial", "seguros"),
    ("Seguro viagem", "seguros"),
    ("Seguro educacional", "seguros"),
    // Pets
    ("Pets", "pet"),
    ("Ração", "pet"),
    ("Veterinário", "pet"),
    // Presentes e doações
    ("Presentes e doações", "presentes"),
    ("Presentes", "presentes"),
    ("Doações", "transfers"),
    // Filhos
    ("Filhos", "compras_diversas"),
    ("Mesada", "transfers"),
    ("Atividades", "servicos_pessoais"),
    ("Esportes e aulas", "servicos_pessoais"),
    ("Fraldas e higiene", "higiene_pessoal"),
    ("Roupas e calçados infantis", "vestuario"),
    ("Brinquedos", "brinquedos"),
    ("Babá e cuidadores", "servicos_domesticos"),
    ("Festas e aniversários", "lazer"),
    ("Pensão alimentícia", "transfers"),
    // Pagamentos
    ("Pagamentos", "contas_diversas"),
    ("Contas e boletos", "contas_diversas"),
    ("Empréstimos", "financial"),
    ("Financiamentos", "financial"),
    ("Parcelas", "compras_diversas"),
    ("Fatura do cartão", "compras_diversas"),
    // Poupança e investimentos
    ("Poupança e investimentos", "financial"),
    ("Poupança dos filhos", "financial"),
    ("Reserva de emergência", "financial"),
    ("Previdência privada", "financial"),
    ("Aportes em investimentos", "financial"),
    ("Outras despesas", "compras_diversas"),
];

/// Grupo padrão de uma categoria pelo nome (sem acentos nem maiúsculas).
pub fn default_group(name: &str) -> Option<&'static str> {
    let f = fold(name);
    DEFAULTS.iter().find(|(n, _)| fold(n) == f).map(|(_, g)| *g)
}

/// Escolha do usuário para uma categoria.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CategoryChoice {
    /// Grupo escolhido (`null` = o padrão).
    pub group_key: Option<String>,
    /// Alíquota própria em % do preço (`null` = a do país e do cenário).
    pub rate: Option<f64>,
}

/// Categoria (folha) para a estimativa.
pub struct CategoryNode {
    pub name: String,
    pub parent_id: Option<Id>,
}

/// Resolve o grupo de cada categoria: escolha do usuário, padrão pelo nome, ou o do pai.
/// Devolve (grupo, origem: `user` | `default` | `parent`) — `None` quando não há grupo.
pub fn resolve(
    id: &str,
    nodes: &HashMap<Id, CategoryNode>,
    choices: &HashMap<Id, CategoryChoice>,
) -> Option<(String, &'static str)> {
    let mut cur = Some(id.to_string());
    let mut origin_self = true;
    while let Some(c) = cur {
        if let Some(g) = choices.get(&c).and_then(|x| x.group_key.clone()) {
            return Some((g, if origin_self { "user" } else { "parent" }));
        }
        let node = nodes.get(&c)?;
        if let Some(g) = default_group(&node.name) {
            return Some((
                g.to_string(),
                if origin_self { "default" } else { "parent" },
            ));
        }
        cur = node.parent_id.clone();
        origin_self = false;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_the_seed_names_and_parents() {
        assert_eq!(default_group("Combustível"), Some("gasolina"));
        assert_eq!(default_group("combustivel"), Some("gasolina"));
        assert_eq!(kind_of("tax_property_ipva"), GroupKind::DirectTax);
        assert_eq!(kind_of("financial"), GroupKind::NotConsumption);
        assert_eq!(kind_of("gasolina"), GroupKind::Consumption);
        let mut nodes = HashMap::new();
        nodes.insert(
            "t".to_string(),
            CategoryNode {
                name: "Transporte".into(),
                parent_id: None,
            },
        );
        nodes.insert(
            "x".to_string(),
            CategoryNode {
                name: "Pedágio".into(),
                parent_id: Some("t".into()),
            },
        );
        let mut choices = HashMap::new();
        assert_eq!(
            resolve("x", &nodes, &choices),
            Some(("compras_diversas".into(), "parent"))
        );
        choices.insert(
            "x".to_string(),
            CategoryChoice {
                group_key: Some("gasolina".into()),
                rate: None,
            },
        );
        assert_eq!(
            resolve("x", &nodes, &choices),
            Some(("gasolina".into(), "user"))
        );
    }

    #[test]
    fn every_seed_category_has_a_default() {
        for (name, _, children) in crate::categories::seed_expenses() {
            assert!(default_group(name).is_some(), "{name}");
            for c in *children {
                assert!(default_group(c).is_some(), "{name} › {c}");
            }
        }
    }
}
