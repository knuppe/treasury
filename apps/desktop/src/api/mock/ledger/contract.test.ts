/**
 * The same contract script as the real core (`scripts/contract-ledger.mjs`),
 * against the mock: shapes equal to the generated types and the same rules.
 */
import { describe, expect, it } from 'vitest';

import { runLedgerContract } from '@/api/contract/flow';
import { MOCK_PASSWORD, mockCall, resetMock } from '@/api/mock';
import { todayIso } from '@/lib/format';

describe('ledger contract on the mock', () => {
  it('passes the whole script, with the shapes of the generated types', async () => {
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
    // The mock reproduces the core's `null`s in optional fields, nothing more.
    expect(warnings.filter((w) => !w.includes('chega como null'))).toEqual([]);
    expect(results.length).toBeGreaterThan(100);
  });
});
