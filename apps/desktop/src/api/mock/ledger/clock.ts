/** "Hoje" do mock: o relógio, ou uma data fixa nos testes (`setMockToday`). */
import { todayIso } from '@/lib/format';

let override: string | null = null;

/** Fixa o "hoje" do mock (testes). `null` volta ao relógio. O banco é refeito na próxima chamada. */
export function setMockToday(iso: string | null): void {
  override = iso;
}

export function mockToday(): string {
  return override ?? todayIso();
}
