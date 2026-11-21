/**
 * Características como tags combináveis no explorador (catálogo `research.tags`, versionado em
 * methodology/tags): "Nunca deu prejuízo", "Proventos há 10 anos", "Dívida baixa", "Cumpre
 * Bazin"… Marcar várias pede todas. A regra de cada uma aparece ao passar o mouse; nada aqui é
 * recomendação.
 */
import type { AssetClass } from '@/api/generated/AssetClass';
import type { AssetTag } from '@/api/generated/AssetTag';
import { Button, Chip, ChipGroup } from '@/design/components';
import { t } from '@/i18n';

/** Tags que valem para as classes do recorte (sem recorte, todas). */
export function tagsFor(all: AssetTag[], classes: AssetClass[] | undefined): AssetTag[] {
  if (!classes?.length) return all;
  return all.filter((x) => x.classes.some((c) => classes.includes(c)));
}

/** `counts`: contagem facetada do resultado na tela (quantos papéis do recorte têm cada tag);
 * sem resultado, vale a contagem do universo. Tag com 0 no recorte fica desabilitada (sem sumir,
 * para o layout não pular); a marcada nunca, para poder desmarcar. */
export function TagPicker({
  tags,
  value,
  counts,
  onChange,
}: {
  tags: AssetTag[];
  value: string[];
  counts?: Record<string, number>;
  onChange: (next: string[]) => void;
}) {
  if (!tags.length) return null;
  const groups: { key: string; label: string; items: AssetTag[] }[] = [];
  for (const x of tags) {
    const g = groups.find((y) => y.key === x.group);
    if (g) g.items.push(x);
    else groups.push({ key: x.group, label: x.group_label, items: [x] });
  }
  const toggle = (key: string) =>
    onChange(value.includes(key) ? value.filter((k) => k !== key) : [...value, key]);
  return (
    <section aria-label={t('pesquisa-tags')} className="flex flex-col gap-2">
      <div className="flex flex-wrap items-baseline gap-2">
        <h4 className="label-caps text-fg">{t('pesquisa-tags')}</h4>
        <span className="text-[12px] text-dim">{t('pesquisa-tags-ajuda')}</span>
        {value.length ? (
          <Button size="sm" variant="ghost" onClick={() => onChange([])}>
            {t('pesquisa-tags-limpar')}
          </Button>
        ) : null}
      </div>
      <div className="flex flex-col gap-2">
        {groups.map((g) => (
          <div key={g.key} className="flex flex-wrap items-center gap-2">
            <span className="w-28 shrink-0 text-[11.5px] text-dim">{g.label}</span>
            <ChipGroup label={g.label}>
              {g.items.map((x) => {
                const pressed = value.includes(x.key);
                const n = counts ? (counts[x.key] ?? 0) : x.count;
                const empty = !pressed && n === 0;
                return (
                  <Chip
                    key={x.key}
                    pressed={pressed}
                    disabled={empty}
                    title={empty ? t('pesquisa-tag-vazia', { regra: x.rule }) : x.rule}
                    onClick={() => toggle(x.key)}
                  >
                    {t('pesquisa-tag-chip', { tag: x.label, n })}
                  </Chip>
                );
              })}
            </ChipGroup>
          </div>
        ))}
      </div>
    </section>
  );
}

/** Rótulos das tags de um papel (para a coluna do resultado). */
export function TagLabels({ keys, tags }: { keys: string[]; tags: AssetTag[] }) {
  const labels = keys
    .map((k) => tags.find((x) => x.key === k))
    .filter((x): x is AssetTag => !!x && x.group !== 'metodos')
    .map((x) => x.label);
  if (!labels.length) return <span className="text-dim">—</span>;
  const shown = labels.slice(0, 3).join(', ');
  return (
    <span className="truncate font-sans text-[12px]" title={labels.join(', ')}>
      {labels.length > 3 ? t('pesquisa-tags-mais', { tags: shown, n: labels.length - 3 }) : shown}
    </span>
  );
}
