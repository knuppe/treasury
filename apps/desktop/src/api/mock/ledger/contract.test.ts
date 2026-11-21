/**
 * O mesmo roteiro de contrato do núcleo real (`scripts/contract-ledger.mjs`),
 * contra o mock: formas iguais às dos tipos gerados e as mesmas regras.
 */
import { describe, expect, it } from 'vitest';

import { runLedgerContract } from '@/api/contract/flow';
import { MOCK_PASSWORD, mockCall, resetMock } from '@/api/mock';
import { todayIso } from '@/lib/format';

describe('contrato do livro-caixa no mock', () => {
  it('passa no roteiro inteiro, com as formas dos tipos gerados', async () => {
    resetMock('unlocked', { keychainAvailable: true });
    const results = await runLedgerContract((cmd, args) => mockCall(cmd, args), {
      password: MOCK_PASSWORD,
      exportDir: '/Users/ana/Documents',
      today: todayIso(),
      restore: true,
    });
    const failures = results
      .filter((r) => !r.ok)
      .map(
        (r) =>
          `${r.cmd} (${r.step}): ${r.error ?? r.findings.map((f) => `${f.path} ${f.message}`).join('; ')}`,
      );
    expect(failures).toEqual([]);
    const warnings = results.flatMap((r) =>
      r.findings.filter((f) => f.level === 'aviso').map((f) => `${r.cmd}: ${f.message}`),
    );
    // O mock reproduz os `null` do núcleo em campos opcionais, nada além disso.
    expect(warnings.filter((w) => !w.includes('chega como null'))).toEqual([]);
    expect(results.length).toBeGreaterThan(100);
  });
});
