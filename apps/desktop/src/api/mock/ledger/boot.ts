/**
 * Liga o banco do livro-caixa ao ciclo de vida do mock: semeia na primeira
 * chamada, de novo a cada `resetMock`, quando o "hoje" do mock muda e quando muda o
 * país da família de exemplo (`mockHomeCountry`). Todos os mocks de módulo usam
 * `ready()`; as visões derivadas usam `ensureSeeded()`.
 */
import { mockHomeCountry, onMockReset, requireUnlocked } from '../../mockRegistry';
import { today } from './dates';
import { seedLedger } from './seed';

let seededFor: string | null = null;

onMockReset(() => {
  seededFor = null;
});

/** Banco semeado para o "hoje" atual (sem conferir o cofre). */
export function ensureSeeded(): void {
  const key = `${today()}|${mockHomeCountry()}`;
  if (seededFor !== key) {
    seededFor = key;
    seedLedger();
  }
}

/** Cofre aberto e banco semeado. Chame no começo de todo comando. */
export function ready(): void {
  requireUnlocked();
  ensureSeeded();
}

/** Semeia de novo (restauração de backup). */
export function reseed(): void {
  seededFor = `${today()}|${mockHomeCountry()}`;
  seedLedger();
}

type Handler = (args: Record<string, unknown>) => unknown;

/** Envolve cada comando com `ready()`. */
export function guarded(map: Record<string, Handler>): Record<string, Handler> {
  return Object.fromEntries(
    Object.entries(map).map(([cmd, fn]) => [
      cmd,
      (args: Record<string, unknown>) => {
        ready();
        return fn(args);
      },
    ]),
  );
}
