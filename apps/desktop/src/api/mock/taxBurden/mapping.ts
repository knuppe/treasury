/**
 * Categoria → grupo de imposto, como `tax_burden::mapping` do núcleo: os grupos de imposto
 * direto e fora do consumo, o mapeamento padrão pelo nome das categorias do conjunto
 * inicial (sem acentos nem maiúsculas) e a resolução escolha do usuário → padrão → pai.
 */
import type { GroupKind } from '@/api/generated/GroupKind';

import { fold } from '../ledger/text';

/** Grupos de imposto direto: o que foi lançado na categoria é imposto pago. */
export const DIRECT: readonly (readonly [string, string])[] = [
  ['tax_income', 'Imposto de renda pago'],
  ['tax_iof', 'IOF pago'],
  ['tax_property_iptu', 'IPTU'],
  ['tax_property_ipva', 'IPVA e licenciamento'],
  ['tax_other', 'Outros tributos pagos'],
];

/** Grupos fora do consumo. */
export const NOT_CONSUMPTION: readonly (readonly [string, string])[] = [
  ['financial', 'Financeiro (dívidas, juros, investimentos)'],
  ['transfers', 'Transferências a pessoas (pensão, mesada, doações)'],
];

export function kindOf(group: string): GroupKind {
  if (DIRECT.some(([k]) => k === group)) return 'direct_tax';
  if (NOT_CONSUMPTION.some(([k]) => k === group)) return 'not_consumption';
  return 'consumption';
}

/** Mapeamento padrão pelo nome da categoria (cópia de `mapping::DEFAULTS`). */
const DEFAULTS: readonly (readonly [string, string])[] = [
  // Moradia
  ['Moradia', 'contas_diversas'],
  ['Aluguel', 'aluguel'],
  ['Condomínio', 'condominio'],
  ['Energia', 'energia_eletrica'],
  ['Água', 'agua_esgoto'],
  ['Gás', 'glp'],
  ['Internet', 'telefonia_internet'],
  ['IPTU', 'tax_property_iptu'],
  ['Manutenção', 'materiais_construcao'],
  ['Diarista e serviços domésticos', 'servicos_domesticos'],
  // Alimentação
  ['Alimentação', 'mercado'],
  ['Mercado', 'mercado'],
  ['Restaurantes', 'restaurantes'],
  ['Delivery', 'restaurantes'],
  ['Padaria', 'alimentos_basicos'],
  // Transporte
  ['Transporte', 'compras_diversas'],
  ['Combustível', 'gasolina'],
  ['Transporte público', 'transporte_publico'],
  ['Aplicativos', 'transporte_aplicativo'],
  ['Estacionamento', 'estacionamento'],
  ['Manutenção do veículo', 'pecas_manutencao_veiculo'],
  ['IPVA e licenciamento', 'tax_property_ipva'],
  ['Seguro do veículo', 'seguros'],
  // Saúde
  ['Saúde', 'saude_plano_consulta'],
  ['Plano de saúde', 'saude_plano_consulta'],
  ['Farmácia', 'medicamentos'],
  ['Consultas', 'saude_plano_consulta'],
  ['Pediatra', 'saude_plano_consulta'],
  ['Exames', 'saude_plano_consulta'],
  ['Dentista', 'saude_plano_consulta'],
  ['Terapias e psicólogo', 'saude_plano_consulta'],
  ['Vacinas', 'saude_plano_consulta'],
  ['Óculos e lentes', 'optica'],
  ['Itens de saúde e primeiros socorros', 'medicamentos'],
  // Educação
  ['Educação', 'educacao_mensalidade'],
  ['Escola', 'educacao_mensalidade'],
  ['Creche', 'educacao_mensalidade'],
  ['Material escolar', 'material_escolar'],
  ['Uniforme', 'vestuario'],
  ['Transporte escolar', 'transporte_publico'],
  ['Cantina e lanche escolar', 'restaurantes'],
  ['Faculdade', 'educacao_mensalidade'],
  ['Idiomas', 'educacao_mensalidade'],
  ['Cursos', 'educacao_mensalidade'],
  ['Livros e material', 'educacao_livros'],
  // Lazer
  ['Lazer', 'lazer'],
  ['Viagens', 'hotel_turismo'],
  ['Passeios', 'lazer'],
  ['Cinema e shows', 'lazer'],
  ['Hobbies', 'lazer'],
  // Compras
  ['Compras', 'compras_diversas'],
  ['Vestuário', 'vestuario'],
  ['Eletrônicos', 'eletronicos_nacionais'],
  ['Casa e decoração', 'moveis'],
  // Cuidados pessoais
  ['Cuidados pessoais', 'higiene_pessoal'],
  ['Cabelo e estética', 'servicos_pessoais'],
  ['Academia', 'servicos_pessoais'],
  ['Cosméticos', 'cosmeticos_perfumes'],
  // Assinaturas
  ['Assinaturas', 'tv_assinatura_streaming'],
  ['Streaming', 'tv_assinatura_streaming'],
  ['Software e apps', 'tv_assinatura_streaming'],
  ['Telefone', 'telefonia_internet'],
  // Impostos e taxas
  ['Impostos e taxas', 'tax_other'],
  ['Tarifas bancárias', 'tarifas_bancarias'],
  ['Juros e multas', 'financial'],
  ['IOF', 'tax_iof'],
  ['Imposto de renda', 'tax_income'],
  // Seguros
  ['Seguros', 'seguros'],
  ['Seguro de vida', 'seguros'],
  ['Seguro residencial', 'seguros'],
  ['Seguro viagem', 'seguros'],
  ['Seguro educacional', 'seguros'],
  // Pets
  ['Pets', 'pet'],
  ['Ração', 'pet'],
  ['Veterinário', 'pet'],
  // Presentes e doações
  ['Presentes e doações', 'presentes'],
  ['Presentes', 'presentes'],
  ['Doações', 'transfers'],
  // Filhos
  ['Filhos', 'compras_diversas'],
  ['Mesada', 'transfers'],
  ['Atividades', 'servicos_pessoais'],
  ['Esportes e aulas', 'servicos_pessoais'],
  ['Fraldas e higiene', 'higiene_pessoal'],
  ['Roupas e calçados infantis', 'vestuario'],
  ['Brinquedos', 'brinquedos'],
  ['Babá e cuidadores', 'servicos_domesticos'],
  ['Festas e aniversários', 'lazer'],
  ['Pensão alimentícia', 'transfers'],
  // Pagamentos
  ['Pagamentos', 'contas_diversas'],
  ['Contas e boletos', 'contas_diversas'],
  ['Empréstimos', 'financial'],
  ['Financiamentos', 'financial'],
  ['Parcelas', 'compras_diversas'],
  ['Fatura do cartão', 'compras_diversas'],
  // Poupança e investimentos
  ['Poupança e investimentos', 'financial'],
  ['Poupança dos filhos', 'financial'],
  ['Reserva de emergência', 'financial'],
  ['Previdência privada', 'financial'],
  ['Aportes em investimentos', 'financial'],
  ['Outras despesas', 'compras_diversas'],
];

const BY_NAME = new Map(DEFAULTS.map(([n, g]) => [fold(n), g]));

export function defaultGroup(name: string): string | undefined {
  return BY_NAME.get(fold(name));
}

export interface CategoryNode {
  name: string;
  parent_id: string | null;
}

export interface CategoryChoice {
  group_key: string | null;
  rate: number | null;
}

export type Origin = 'user' | 'default' | 'parent';

/** Grupo da categoria: escolha do usuário, padrão pelo nome ou o do pai (`null` = sem grupo). */
export function resolve(
  id: string,
  nodes: ReadonlyMap<string, CategoryNode>,
  choices: ReadonlyMap<string, CategoryChoice>,
): { group: string; origin: Origin } | null {
  let cur: string | null = id;
  let self = true;
  while (cur) {
    const chosen = choices.get(cur)?.group_key;
    if (chosen) return { group: chosen, origin: self ? 'user' : 'parent' };
    const node = nodes.get(cur);
    if (!node) return null;
    const g = defaultGroup(node.name);
    if (g) return { group: g, origin: self ? 'default' : 'parent' };
    cur = node.parent_id;
    self = false;
  }
  return null;
}
