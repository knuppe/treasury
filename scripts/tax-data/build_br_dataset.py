"""Atualiza as regras de renda do trabalho (IRPF e INSS) de `data/br.json` a partir da pesquisa.

Uso: python3 scripts/tax-data/build_br_dataset.py docs/tax-data/research \
         crates/treasury-core/src/tax_burden/data/br.json

O arquivo de saída precisa existir: só `payroll` (e as fontes que ele cita) é regravado. Os
percentuais de consumo vêm das Contas Nacionais do IBGE por `br_consumo_ibge.py`; os estados
(IPVA), o contexto e a metodologia são mantidos como estão no arquivo.
"""
import json, sys, os
src_dir, out = sys.argv[1], sys.argv[2]
d = json.load(open(os.path.join(src_dir, 'br.json')))
states = None
sp = os.path.join(src_dir, 'br_icms.json')
if os.path.exists(sp):
    states = json.load(open(sp))

def cents(v): return int(round(v * 100))

sources = []
seen = set()
def add_sources(lst):
    for s in lst:
        if s['id'] in seen: continue
        seen.add(s['id'])
        sources.append({k: s[k] for k in ('id','publisher','title','url') } | {
            'published': s.get('published'), 'accessed': s.get('accessed','2026-10-17')})
add_sources(d['sources'])
if states: add_sources(states['sources'])

inc = d['income']['irpf_monthly']
ins = d['payroll']['inss_employee']
def ir_rules(key, frm, inss_year):
    r = inc[key]
    rule = {'from': frm, 'legal_basis': r['legal_basis'] + ' · INSS: ' + ins[inss_year]['legal_basis'],
            'ir_monthly': [{'up_to': cents(b['up_to']) if b['up_to'] is not None else None,
                            'rate': b['rate'], 'deduction': cents(b['deduction'])} for b in r['brackets']],
            'dependent_deduction': cents(r['dependent_deduction']),
            'simplified_discount': cents(r['simplified_discount_monthly']),
            'inss_bands': [{'up_to': cents(b['up_to']), 'rate': b['rate']} for b in ins[inss_year]['bands']],
            'source_ids': [s for s in r['source_ids'] + ins[inss_year]['source_ids'] if s in seen]}
    if 'reduction' in r:
        rule['reducer'] = {'exempt_up_to': 500000, 'max_reduction': 31289, 'phase_out_up_to': 735000,
                           'a': 97862, 'b': 0.133145}
    return rule
payroll = [ir_rules('2025_jan_apr', '2025-01', '2025'), ir_rules('2025_from_may', '2025-05', '2025'),
           ir_rules('2026', '2026-01', '2026')]

base = json.load(open(out, encoding='utf-8'))
have = {x['id'] for x in base['sources']}
need = set()
for p in payroll: need.update(p['source_ids'])
missing = need - seen
assert not missing, missing
base['sources'] += [x for x in sources if x['id'] in need and x['id'] not in have]
base['payroll'] = payroll
open(out, 'w', encoding='utf-8').write(json.dumps(base, ensure_ascii=False, indent=1) + '\n')
print(len(payroll), 'regras de renda regravadas em', out)
