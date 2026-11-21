/**
 * Códigos de barras do produto (docs/gtin-inflation.md §3): os que vieram dos cupons e os que a
 * pessoa digitou ou leu da foto. O núcleo confere o tamanho, o dígito verificador e se o código
 * é de uso interno da loja, e diz o que está errado.
 */
import { useState } from 'react';

import type { FoodProduct } from '@/api/generated/FoodProduct';
import { useAddProductGtin, useRemoveProductGtin } from '@/api/hooks/food';
import { Button, Field, Icon, IconButton, Input } from '@/design/components';
import { canReadBarcode, readBarcode } from '@/features/receipts/barcode';
import { FileInput, InlineError } from '@/features/transactions/ui/controls';
import { t } from '@/i18n';

export function BarcodeField({ product }: { product: FoodProduct }) {
  const add = useAddProductGtin();
  const remove = useRemoveProductGtin();
  const [code, setCode] = useState('');
  const [photoMsg, setPhotoMsg] = useState<string | null>(null);
  const save = () => {
    const c = code.trim();
    if (!c) return;
    add.mutate({ product_id: product.id, code: c }, { onSuccess: () => setCode('') });
  };
  return (
    <Field label={t('alimentacao-codigo-barras')} hint={t('alimentacao-codigo-barras-dica')}>
      <div className="flex flex-col gap-2">
        {product.gtins.length ? (
          <ul className="flex flex-wrap gap-1.5" aria-label={t('alimentacao-codigos-do-produto')}>
            {product.gtins.map((g) => (
              <li
                key={g}
                className="num flex items-center gap-1 rounded-control border border-line py-0.5 pr-0.5 pl-2 text-[12.5px]"
              >
                {g}
                <IconButton
                  size="sm"
                  label={t('alimentacao-codigo-tirar', { codigo: g })}
                  icon={<Icon name="x" size={11} />}
                  onClick={() => remove.mutate({ product_id: product.id, code: g })}
                />
              </li>
            ))}
          </ul>
        ) : null}
        <div className="flex flex-wrap items-center gap-2">
          <Input
            aria-label={t('alimentacao-codigo-novo')}
            className="max-w-48"
            inputMode="numeric"
            placeholder="7891234567895"
            value={code}
            onChange={(e) => setCode(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && save()}
          />
          <Button
            size="sm"
            variant="secondary"
            disabled={!code.trim()}
            loading={add.isPending}
            onClick={save}
          >
            {t('alimentacao-codigo-adicionar')}
          </Button>
          {canReadBarcode() ? (
            <FileInput
              aria-label={t('alimentacao-codigo-foto')}
              accept="image/*"
              onChange={async (e) => {
                const f = e.target.files?.[0];
                e.target.value = '';
                if (!f) return;
                setPhotoMsg(null);
                const found = await readBarcode(f);
                if (found) setCode(found);
                else setPhotoMsg(t('alimentacao-codigo-foto-nada'));
              }}
            />
          ) : null}
        </div>
        {photoMsg ? <p className="text-[12.5px] text-dim">{photoMsg}</p> : null}
        <InlineError error={add.error ?? remove.error} />
      </div>
    </Field>
  );
}
