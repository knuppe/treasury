/** Flash benefit card (VA/VR): benefit-type account and the CSV recognized by its header. */
import { beforeEach, describe, expect, it } from 'vitest';

import type { AccountView } from '@/api/generated/AccountView';
import type { ImportPreview } from '@/api/generated/ImportPreview';
import { mockCall, resetMock } from '@/api/mock';

const CSV = [
  'Data,Hora,Movimentação,Valor,Meio de Pagamento,Saldo',
  '16/10/2026,12:41,"CAPPTA *RESTAURANTE BOA MESA CAMPINAS BRA","-R$ 42,70",Cartão,"R$ 1.885,30"',
  '15/10/2026,19:01,"IFD*BR OSASCO BR","-R$ 0,00",Cartão,"R$ 1.928,00"',
  '28/09/2026,09:14,Depósito transferido,"R$ 1.193,00",Depósito,"R$ 1.928,00"',
  '28/09/2026,09:14,Depósito transferido,"R$ 735,00",Depósito,"R$ 735,00"',
].join('\n');

const b64 = (s: string) => btoa(String.fromCharCode(...new TextEncoder().encode(s)));

beforeEach(() => resetMock('unlocked'));

describe('Flash card', () => {
  it('benefit account and CSV statement with the Flash layout', async () => {
    const acc = (await mockCall('accounts.create', {
      name: 'Flash',
      kind: 'benefit',
    })) as AccountView;
    expect(acc.kind).toBe('benefit');
    const p = (await mockCall('import.preview', {
      account_id: acc.id,
      filename: 'flash_extrato_10-2026.csv',
      content_base64: b64(CSV),
    })) as ImportPreview;
    expect(p.preset_id).toBe('flash');
    expect(p.rows.map((r) => r.amount)).toEqual([-4270, 0, 119300, 73500]);
    expect(p.rows[0]!.payee).toBe('RESTAURANTE BOA MESA CAMPINAS');
    expect(p.rows[1]!.status).toBe('ignored');
    expect(p.rows[2]!.payee).toBeNull();
    expect(new Set(p.rows.map((r) => r.external_id)).size).toBe(4);
  });
});
