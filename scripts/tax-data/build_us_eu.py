"""Esboço dos EUA e da União Europeia no formato do app (só imposto sobre vendas / IVA)."""
import json, sys, os
src, out_dir = sys.argv[1], sys.argv[2]
BR = json.load(open(os.path.join(out_dir, 'br.json')))
labels = {g['key']: g['label'] for g in BR['consumption']}

def srcs(d):
    return [{k: s.get(k) for k in ('id','publisher','title','url','published')} | {'accessed': s.get('accessed','2026-10-17')} for s in d['sources']]

def group(key, low, central, high, basis, sids):
    return {'key': key, 'label': labels[key], 'low': low, 'central': central, 'high': high,
            'basis': basis, 'source_ids': sids if low is not None else [], 'icms_modal': False}

ALL = [g['key'] for g in BR['consumption']]

# ---- EUA
u = json.load(open(os.path.join(src, 'us.json')))
S, X = 'sales_tax', 'exempt'
us_rules = {
 'alimentos_basicos': (X, X, S), 'alimentos_industrializados': (X, X, S), 'mercado': (X, X, S),
 'bebidas_nao_alcoolicas': (X, S, S), 'medicamentos': (X, X, S),
}
for k in ['bebidas_alcoolicas','restaurantes','eletronicos_nacionais','eletronicos_importados','eletrodomesticos','moveis',
          'vestuario','calcados','higiene_pessoal','cosmeticos_perfumes','brinquedos','materiais_construcao',
          'pecas_manutencao_veiculo','automovel_novo','motocicleta','pet','hotel_turismo','educacao_livros',
          'presentes','compras_diversas','optica','material_escolar']:
    us_rules[k] = (S, S, S)
for k in ['educacao_mensalidade','saude_plano_consulta','transporte_publico','aluguel','condominio','tarifas_bancarias',
          'servicos_domesticos','servicos_pessoais']:
    us_rules[k] = (X, X, X)
us_groups = []
for k in ALL:
    if k in us_rules:
        lo, ce, hi = us_rules[k]
        us_groups.append(group(k, lo, ce, hi,
            'Imposto estadual e local sobre vendas (média combinada do estado, Tax Foundation). Alimentos de mercado e remédios: isentos na maioria dos estados (Conservador e Provável), tributados no Elevado. Serviços em geral fora do imposto sobre vendas. Não inclui impostos especiais (combustível, bebida, cigarro) nem tributos embutidos na cadeia.',
            ['tf_sales_2026_midyear']))
    else:
        us_groups.append(group(k, None, None, None, 'Sem dado confiável neste esboço (impostos especiais e de serviços públicos não levantados).', []))
us_regions = [{'code': r['code'], 'name': r['state'], 'icms_modal': None, 'ipva_car': None, 'ipva_motorcycle': None,
               'sales_tax': r['combined_rate'], 'vat_standard': None, 'vat_reduced': None, 'vat_super_reduced': None,
               'source_ids': r['source_ids']} for r in u['consumption']]
US = {'country': 'US', 'name': 'Estados Unidos', 'currency': 'USD', 'version': '2026-10', 'updated': '2026-10-17',
      'coverage': 'esboço', 'sources': srcs(u), 'consumption': us_groups, 'regions': us_regions, 'reference_icms': None,
      'payroll': [], 'context': None,
      'methodology': ['Esboço: só o imposto estadual e local sobre vendas, pela alíquota combinada média do estado (Tax Foundation, meados de 2026), como fração do valor pago: alíquota / (100 + alíquota).',
                      'Renda: o imposto federal e estadual sobre a renda e o FICA não são estimados a partir do líquido; lance os informes de rendimentos (W-2) para que entrem.'],
      'disclaimer': 'Esboço de estimativa: considera só o imposto sobre vendas médio do estado e os impostos lançados ou informados. Os valores reais variam por cidade, produto e situação de cada pessoa. Fonte: Tax Foundation.'}

# ---- União Europeia
e = json.load(open(os.path.join(src, 'eu.json')))
STD, RED, SUP = 'vat_standard', 'vat_reduced', 'vat_super_reduced'
eu_rules = {
 'alimentos_basicos': (SUP, RED, STD), 'alimentos_industrializados': (SUP, RED, STD), 'mercado': (SUP, RED, STD),
 'bebidas_nao_alcoolicas': (RED, RED, STD), 'restaurantes': (RED, RED, STD), 'medicamentos': (SUP, RED, STD),
 'educacao_livros': (SUP, RED, RED), 'hotel_turismo': (RED, RED, STD), 'transporte_publico': (RED, RED, STD),
 'passagens_aereas': (X, RED, STD), 'energia_eletrica': (RED, STD, STD), 'agua_esgoto': (RED, RED, STD),
 'glp': (RED, STD, STD), 'pet': (RED, STD, STD), 'servicos_pessoais': (RED, STD, STD),
 'educacao_mensalidade': (X, X, X), 'saude_plano_consulta': (X, X, X), 'aluguel': (X, X, X), 'tarifas_bancarias': (X, X, X),
}
for k in ['bebidas_alcoolicas','eletronicos_nacionais','eletronicos_importados','eletrodomesticos','moveis','vestuario',
          'calcados','higiene_pessoal','cosmeticos_perfumes','brinquedos','materiais_construcao','pecas_manutencao_veiculo',
          'automovel_novo','motocicleta','telefonia_internet','tv_assinatura_streaming','gasolina','diesel','presentes',
          'compras_diversas','optica','material_escolar']:
    eu_rules[k] = (STD, STD, STD)
eu_groups = []
for k in ALL:
    if k in eu_rules:
        lo, ce, hi = eu_rules[k]
        eu_groups.append(group(k, lo, ce, hi,
            'IVA do país escolhido (tabela da Comissão Europeia), como fração do valor pago: alíquota / (100 + alíquota). Conservador usa a alíquota mais baixa que costuma valer para o grupo; Elevado, a padrão. Não inclui impostos especiais (combustível, bebida, tabaco, energia).',
            ['youreurope_vat']))
    else:
        eu_groups.append(group(k, None, None, None, 'Sem dado confiável neste esboço.', []))
eu_regions = []
for r in e['consumption']:
    red = sorted(r.get('reduced_rates') or [])
    eu_regions.append({'code': r['code'], 'name': r['country'], 'icms_modal': None, 'ipva_car': None, 'ipva_motorcycle': None,
        'sales_tax': None, 'vat_standard': r['standard_rate'], 'vat_reduced': red[0] if red else None,
        'vat_super_reduced': r.get('super_reduced_rate'), 'source_ids': r['source_ids']})
EU = {'country': 'EU', 'name': 'União Europeia', 'currency': 'EUR', 'version': '2026-10', 'updated': '2026-10-17',
      'coverage': 'esboço', 'sources': srcs(e), 'consumption': eu_groups, 'regions': eu_regions, 'reference_icms': None,
      'payroll': [], 'context': None,
      'methodology': ['Esboço: só o IVA, pelas alíquotas padrão, reduzida (a menor) e super-reduzida do país escolhido (tabela "Your Europe" da Comissão Europeia, conferida em 13/07/2026).',
                      'Renda e contribuições sociais não são estimadas a partir do líquido; lance os informes de rendimentos para que entrem.'],
      'disclaimer': 'Esboço de estimativa: considera só o IVA do país escolhido e os impostos lançados ou informados. Alíquotas reduzidas variam por produto em cada país. Fonte: Comissão Europeia.'}

for name, data in [('us.json', US), ('eu.json', EU)]:
    known = {s['id'] for s in data['sources']}
    for g in data['consumption']: assert all(x in known for x in g['source_ids']), g
    for r in data['regions']: assert all(x in known for x in r['source_ids']), r
    json.dump(data, open(os.path.join(out_dir, name), 'w'), ensure_ascii=False, indent=1)
    print(name, len(data['consumption']), 'groups', len(data['regions']), 'regions')
