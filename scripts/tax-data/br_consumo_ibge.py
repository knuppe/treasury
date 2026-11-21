"""Dados oficiais do Brasil para a estimativa de impostos: percentuais de consumo pelas Contas
Nacionais do IBGE e despesa do governo geral por função (Tesouro Nacional, COFOG).

Lê `docs/tax-data/research/br_ibge_tru_2021.json` (valores copiados das Tabelas de Recursos e
Usos de 2021) e regrava, em `crates/treasury-core/src/tax_burden/data/br.json`, os grupos de
consumo, as fontes, a referência dos dias de trabalho, a despesa por função
(`docs/tax-data/research/br_cofog_2024.json`) e a metodologia.

Percentual de um produto = impostos sobre o produto (importação, IPI, ICMS e outros, menos
subsídios) ÷ oferta total a preço de consumidor. Um grupo com vários produtos usa a média
ponderada pelo consumo das famílias. Gasolina, diesel e gás de cozinha ficam com a composição de
preço da Petrobras; seguros e tarifas bancárias, com a alíquota da lei.

Uso: python3 scripts/tax-data/br_consumo_ibge.py
"""
import json, os

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TRU = os.path.join(ROOT, 'docs/tax-data/research/br_ibge_tru_2021.json')
OUT = os.path.join(ROOT, 'crates/treasury-core/src/tax_burden/data/br.json')
COFOG = os.path.join(ROOT, 'docs/tax-data/research/br_cofog_2024.json')
TODAY = '2026-11-18'
IBGE = 'ibge_scn_tru_2021'

# Grupo de consumo do app → produtos das Contas Nacionais (códigos da TRU).
MAP = {
    'alimentos_basicos': ['01911', '01916', '01917', '01919', '01924', '02802', '10911', '10912', '10913',
                          '10915', '10921', '10932', '10933', '10934', '10935'],
    'alimentos_industrializados': ['10914', '10916', '10931', '10937'],
    'bebidas_nao_alcoolicas': ['11001'],
    'bebidas_alcoolicas': ['11001'],
    'restaurantes': ['56001'],
    'etanol': ['19921'],
    'energia_eletrica': ['35001'],
    'agua_esgoto': ['36801'],
    'telefonia_internet': ['61001'],
    'tv_assinatura_streaming': ['61001'],
    'eletronicos_nacionais': ['26002', '26003'],
    'eletronicos_importados': ['26002', '26003'],
    'eletrodomesticos': ['27002'],
    'moveis': ['31801'],
    'vestuario': ['14001'],
    'calcados': ['15001'],
    'medicamentos': ['21001'],
    'higiene_pessoal': ['20931'],
    'cosmeticos_perfumes': ['20931'],
    'limpeza': ['20931'],
    'servicos_pessoais': ['94803'],
    'educacao_mensalidade': ['85921'],
    'educacao_livros': ['58001'],
    'transporte_publico': ['49002'],
    'transporte_aplicativo': ['49002'],
    'passagens_aereas': ['51001'],
    'hotel_turismo': ['55001'],
    'automovel_novo': ['29911'],
    'pecas_manutencao_veiculo': ['29921', '45001'],
    'brinquedos': ['31802'],
    'optica': ['31802'],
    'material_escolar': ['17002', '31802'],
    'pet': ['10936'],
    'materiais_construcao': ['20923', '23001', '23002', '23003'],
    'cigarros': ['12001'],
    'lazer': ['59801', '90801'],
    'aluguel': ['68001'],
    'condominio': ['78801'],
    'estacionamento': ['52801'],
}
# Mercado: tudo o que é alimento, básico ou industrializado.
MAP['mercado'] = MAP['alimentos_basicos'] + MAP['alimentos_industrializados']

# Ficam como estão: composição de preço da Petrobras e alíquotas da lei.
KEEP = {'gasolina', 'diesel', 'glp', 'seguros', 'tarifas_bancarias'}
# Sem dado oficial que sirva ao grupo: o gasto aparece, fora da estimativa.
NO_DATA = {
    'gnv': 'As Contas Nacionais juntam o gás natural à eletricidade e a outras utilidades.',
    'motocicleta': 'As Contas Nacionais juntam motocicletas a aeronaves e embarcações.',
    'saude_plano_consulta': 'Plano de saúde e consultas caem em produtos diferentes das Contas Nacionais.',
    'servicos_domesticos': 'Não há imposto sobre o produto; os encargos da folha não entram nesta estimativa.',
    'presentes': 'Itens variados, sem um produto das Contas Nacionais que os represente.',
    'compras_diversas': 'Compras sem detalhe, sem um produto das Contas Nacionais que as represente.',
    'contas_diversas': 'Contas sem detalhe, sem um produto das Contas Nacionais que as represente.',
}
# Trechos antigos das explicações que citavam a lista de produtos abandonada.
OLD_NOTES = {
    'gasolina': [' A lista do IBPT traz 51,63% para gasolina, valor sem data e anterior, ao que tudo indica, à LC 192/194 de 2022 (ICMS ad rem e teto); por isso não foi usado no cenário.'],
    'diesel': [' A lista do IBPT traz 44,07% para diesel (sem data; não usada pelo mesmo motivo da gasolina).'],
}
REWRITE = {
    'glp': 'Os três cenários usam o ICMS de 16,7% da composição de preço da Petrobras (tributos federais zerados). Data de referência: coleta 2026-10-04 a 2026-10-10. A Petrobras informa que CIDE, PIS/Pasep e Cofins do GLP estão zerados.',
}
# Fontes que não são oficiais e deixam de ser usadas.
DROP_SOURCES = {'ibpt_impostometro_produtos', 'ibpt_dias_2026', 'eshoje_2025_05_07', 'cnn_abradee_2025',
                'acende_pwc_2021', 'teletime_conexis_2024', 'prt_conexis_2024', 'sindusfarma_perfil_2022'}


def fmt(v):
    return ('%.2f' % v).replace('.', ',')


def main():
    tru = json.load(open(TRU, encoding='utf-8'))
    prod = {p['code']: p for p in tru['products']}
    d = json.load(open(OUT, encoding='utf-8'))

    def share(code):
        p = prod[code]
        return 100.0 * p['taxes_total'] / p['supply_consumer_price']

    for g in d['consumption']:
        key = g['key']
        g['icms_modal'] = False
        if key in KEEP:
            g['source_ids'] = [s for s in g['source_ids'] if s not in DROP_SOURCES]
            assert g['source_ids'], key
            # Sai da explicação o que comparava com a lista de produtos que deixou de ser usada.
            for cut in OLD_NOTES.get(key, []):
                g['basis'] = g['basis'].replace(cut, '').strip()
            if key in REWRITE:
                g['basis'] = REWRITE[key]
            assert 'IBPT' not in g['basis'], key
            continue
        if key in NO_DATA:
            g.update(low=None, central=None, high=None, source_ids=[],
                     basis='Sem dado oficial para o grupo: o gasto aparece na tela, mas não entra na estimativa. ' + NO_DATA[key])
            continue
        codes = MAP[key]
        weight = sum(prod[c]['household_consumption'] for c in codes)
        assert weight > 0, key
        value = round(sum(share(c) * prod[c]['household_consumption'] for c in codes) / weight, 2)
        parts = '; '.join(f"{prod[c]['name']} {fmt(share(c))}%" for c in codes)
        how = ('Média ponderada pelo consumo das famílias. ' if len(codes) > 1 else '')
        g.update(low=value, central=value, high=value, source_ids=[IBGE],
                 basis=f'Contas Nacionais do IBGE, 2021, impostos sobre o produto ÷ oferta a preço de consumidor: {parts}. {how}'
                       'O mesmo valor nos três cenários.')
    keys = {g['key'] for g in d['consumption']}
    assert keys == set(MAP) | KEEP | set(NO_DATA), keys ^ (set(MAP) | KEEP | set(NO_DATA))

    # Estado: os percentuais do IBGE são nacionais; o ajuste pelo ICMS de cada estado (e as
    # fontes dele, em parte compilações privadas) deixa de existir. Fica o IPVA.
    icms_only = set()
    for r in d['regions']:
        icms_only.update(x for x in r['source_ids'] if not x.startswith('ipva_'))
        r['source_ids'] = [x for x in r['source_ids'] if x.startswith('ipva_')]
        r['icms_modal'] = None
    d['reference_icms'] = None
    still = set()
    for g in d['consumption']: still.update(g['source_ids'])
    for p in d['payroll']: still.update(p['source_ids'])
    drop = DROP_SOURCES | (icms_only - still)
    d['sources'] = [s for s in d['sources'] if s['id'] not in drop and s['id'] != IBGE]
    d['sources'].insert(0, {
        'id': IBGE, 'publisher': 'IBGE (Instituto Brasileiro de Geografia e Estatística)',
        'title': 'Sistema de Contas Nacionais: Tabelas de Recursos e Usos de 2021, nível de 68 atividades e 128 produtos (impostos sobre produtos e consumo das famílias)',
        'url': tru['url'], 'published': tru['published'], 'accessed': TODAY})

    # Dias de trabalho: a carga tributária bruta do Tesouro, em dias do ano.
    ctb = d['context']
    days = round(365 * ctb['value'] / 100)
    month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    m, left = 0, days
    while left > month_days[m]:
        left -= month_days[m]; m += 1
    d['work_days'] = {'year': ctb['year'], 'days': days, 'until': '%d-%02d-%02d' % (ctb['year'], m + 1, left), 'share': ctb['value'],
                      'label': 'Carga tributária bruta do Brasil em dias do ano (Tesouro Nacional)',
                      'source_ids': ctb['source_ids']}

    # Para onde vai o imposto: despesa do governo geral por função.
    cofog = json.load(open(COFOG, encoding='utf-8'))
    assert abs(sum(f['amount'] for f in cofog['functions']) - cofog['total']['amount']) <= 5
    d['sources'] = [s for s in d['sources'] if s['id'] != cofog['source_id']]
    d['sources'].insert(1, {k: cofog[k if k != 'id' else 'source_id'] for k in ('id', 'publisher', 'title', 'url', 'published', 'accessed')})
    d['spending'] = {'year': cofog['year'],
                     'label': 'Despesa do governo geral por função (União, estados e municípios)',
                     'functions': [{'key': f['key'], 'label': f['label'], 'share': f['share'], 'gdp_share': f['gdp_share']}
                                   for f in cofog['functions']],
                     'source_ids': [cofog['source_id']]}

    d['methodology'] = [
        'Consumo: o gasto de cada categoria vezes a fração do preço que é imposto sobre o produto. Os percentuais vêm das Contas Nacionais do IBGE (Tabelas de Recursos e Usos de 2021, o último ano publicado com 128 produtos): impostos sobre o produto (imposto de importação, IPI, ICMS e outros impostos menos subsídios) divididos pela oferta a preço de consumidor. Para gasolina, diesel e gás de cozinha vale a composição de preço publicada pela Petrobras; para seguros e tarifas bancárias, a alíquota da lei.',
        'Cenários: os grupos calculados pelas Contas Nacionais têm o mesmo valor nos três cenários. Quando um grupo junta vários produtos, o valor é a média ponderada pelo consumo das famílias. Os cenários só diferem onde a fonte dá uma faixa (combustíveis, seguros e tarifas bancárias).',
        'Estado: os percentuais de consumo são médias nacionais e não mudam com o estado escolhido. O estado entra no IPVA.',
        d['methodology'][3],
        d['methodology'][4],
        d['methodology'][5],
        'O que o número do IBGE mede: só os impostos cobrados sobre o produto. Impostos pagos antes na cadeia e não recuperados, encargos sobre a folha e o imposto de renda das empresas não entram. Por isso os percentuais são menores que os de estimativas que somam esses tributos, e o resultado deve ser lido como um piso.',
        'Não entram: encargos pagos pelo empregador (INSS patronal, FGTS), grupos sem dado oficial (plano de saúde, serviços domésticos, gás veicular, motocicletas, compras sem detalhe) e o teste da reforma tributária de 2026 (CBS 0,9% e IBS 0,1%, compensados com PIS/Cofins).',
        'Atualização: as Contas Nacionais com esse detalhe saem com cerca de dois anos de atraso. Mudanças de alíquota depois de 2021, como o teto do ICMS de energia e telecomunicações de 2022, só aparecem quando o IBGE publicar o ano correspondente.',
    ]
    d['disclaimer'] = ('Esta é uma estimativa baseada em médias nacionais de imposto por categoria de consumo e nas regras de imposto de renda vigentes. '
                       'Os valores reais variam conforme o produto, o estado, o regime tributário das empresas e benefícios fiscais específicos. '
                       'Fontes: IBGE (Contas Nacionais), Receita Federal, Previdência Social, Tesouro Nacional e Petrobras.')
    d['version'] = '2026-11'
    d['updated'] = TODAY

    used = set()
    for g in d['consumption']: used.update(g['source_ids'])
    for r in d['regions']: used.update(r['source_ids'])
    for p in d['payroll']: used.update(p['source_ids'])
    used.update(d['context']['source_ids']); used.update(d['work_days']['source_ids'])
    used.update(d['spending']['source_ids'])
    known = {s['id'] for s in d['sources']}
    assert used <= known, used - known
    open(OUT, 'w', encoding='utf-8').write(json.dumps(d, ensure_ascii=False, indent=1) + '\n')
    for g in d['consumption']:
        print(f"{g['key']:28} {g['low']} {g['central']} {g['high']}")
    print('dias', d['work_days'])


main()
