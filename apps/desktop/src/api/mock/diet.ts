/**
 * Mock de `food.diet_list` e `food.diet_set` (docs/gtin-inflation.md §7.2): alergias e restrições
 * alimentares de cada membro, em memória. Mesmas regras do núcleo: só as restrições conhecidas,
 * a lista inteira troca a do membro, e a resposta traz só quem tem alguma, por nome e restrição.
 */
import type { MemberDiet } from '../generated/MemberDiet';
import { onMockReset, registerMockHandlers } from '../mockRegistry';
import { guarded } from './ledger/boot';
import { invalid, notFound } from './ledger/errors';
import { rows } from './ledger/store';

/** As restrições que o núcleo aceita (`food::health::RESTRICTIONS`). */
export const DIET_RESTRICTIONS = [
  'gluten',
  'milk',
  'eggs',
  'peanuts',
  'nuts',
  'soybeans',
  'fish',
  'crustaceans',
  'sesame',
  'vegan',
  'vegetarian',
] as const;

/** Membro → restrições marcadas. */
let diets = new Map<string, Set<string>>();
onMockReset(() => {
  diets = new Map();
});

function list(): MemberDiet[] {
  const names = new Map(rows('member').map((m) => [m.id, m.name]));
  return [...diets.entries()]
    .filter(([id, set]) => set.size > 0 && names.has(id))
    .map(([id, set]) => ({
      member_id: id,
      member_name: names.get(id)!,
      restrictions: [...set].sort(),
    }))
    .sort((a, b) => a.member_name.localeCompare(b.member_name, 'pt-BR'));
}

registerMockHandlers(
  guarded({
    'food.diet_list': () => list(),
    'food.diet_set': (args) => {
      const id = typeof args.member_id === 'string' ? args.member_id : '';
      const raw: unknown[] = Array.isArray(args.restrictions) ? args.restrictions : [];
      const bad = raw.find(
        (r) => typeof r !== 'string' || !(DIET_RESTRICTIONS as readonly string[]).includes(r),
      );
      if (bad !== undefined) invalid(`restrição alimentar desconhecida: ${JSON.stringify(bad)}`);
      if (!rows('member').some((m) => m.id === id)) notFound('membro');
      diets.set(id, new Set(raw as string[]));
      return list();
    },
  }),
);
