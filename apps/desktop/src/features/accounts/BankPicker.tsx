import { useDeferredValue } from 'react';

import type { Bank } from '@/api/generated/Bank';
import { useCountryInfo } from '@/api/hooks/country';
import { useBanks } from '@/api/hooks/household';
import { Button, Input } from '@/design/components';
import { t } from '@/i18n';

const SMALL = new Set(['de', 'do', 'da', 'dos', 'das', 'e']);
const KEEP = new Set([
  'S.A.',
  'S/A',
  'IP',
  'S.A',
  'LTDA',
  'LTDA.',
  'CFI',
  'SCD',
  'DTVM',
  'CTVM',
  'BTG',
]);

/** "NU PAGAMENTOS S.A. - INSTITUIÇÃO DE PAGAMENTO" → "Nu Pagamentos S.A. - Instituição de Pagamento". */
export function bankName(b: Bank): string {
  const raw = b.full_name || b.short_name;
  if (raw !== raw.toUpperCase()) return raw;
  return raw
    .split(' ')
    .map((w, i) => {
      if (KEEP.has(w)) return w;
      const low = w.toLocaleLowerCase('pt-BR');
      if (i > 0 && SMALL.has(low)) return low;
      return low.charAt(0).toLocaleUpperCase('pt-BR') + low.slice(1);
    })
    .join(' ');
}

/**
 * Busca na lista oficial do Banco Central (nome, código COMPE ou ISPB). Escolher um banco
 * preenche o nome e o código da nova instituição; o texto digitado também serve, sem código,
 * para quem não está na lista (corretora estrangeira, carteira). Fora do Brasil, só o texto
 * livre: a lista é dos bancos daqui (docs/international.md).
 */
export function BankPicker({
  value,
  code,
  onChange,
  onCancel,
}: {
  value: string;
  code: string | null;
  onChange: (name: string, code: string | null) => void;
  onCancel: () => void;
}) {
  const query = useDeferredValue(value.trim());
  const brazil = (useCountryInfo()?.code ?? 'BR') === 'BR';
  const { data: banks = [] } = useBanks(query, brazil && query.length >= 2 && !code);
  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex gap-2">
        <Input
          value={value}
          onChange={(e) => onChange(e.target.value, null)}
          placeholder={brazil ? t('contas-banco-exemplo') : t('contas-banco-exemplo-exterior')}
          aria-label={t('contas-nova-instituicao')}
          suffix={code ? <span className="font-mono text-[11px] text-dim">{code}</span> : null}
          autoFocus
        />
        <Button size="sm" variant="ghost" onClick={onCancel}>
          {t('comum-cancelar')}
        </Button>
      </div>
      {!code && banks.length > 0 ? (
        <ul
          role="listbox"
          aria-label={t('contas-bancos-bc')}
          className="max-h-44 overflow-auto rounded-control border border-line bg-bg"
        >
          {banks.slice(0, 8).map((b) => (
            <li key={b.ispb} role="option" aria-selected={false}>
              <button
                type="button"
                className="flex w-full items-baseline gap-2 px-2.5 py-1.5 text-left text-[13px] hover:bg-surface-2"
                onClick={() => onChange(bankName(b), b.code)}
              >
                <span className="w-8 shrink-0 font-mono text-[11.5px] text-dim">
                  {b.code ?? '—'}
                </span>
                <span className="truncate">{bankName(b)}</span>
              </button>
            </li>
          ))}
        </ul>
      ) : null}
      <p className="text-[11.5px] text-dim">
        {code ? t('contas-banco-oficial') : t('contas-lista-oficial')}
      </p>
    </div>
  );
}
