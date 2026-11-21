import { QueryClient } from '@tanstack/react-query';
import { describe, expect, it, vi } from 'vitest';

import type { DataEvent } from '../generated/DataEvent';
import { makeDataEventHandler } from './dataEvents';
import * as components from '@/design/components';

const ev = (module: string, actor = 'mcp'): DataEvent => ({
  seq: 1,
  at: '2026-10-16T00:00:00Z',
  cmd: `${module}.x`,
  module,
  actor,
  client: actor === 'mcp' ? 'agente' : null,
});

describe('tempo real (data://changed)', () => {
  it('agrupa a rajada, relê só os módulos afetados e ignora a própria interface', () => {
    vi.useFakeTimers();
    const client = new QueryClient();
    const spy = vi.spyOn(client, 'invalidateQueries');
    const on = makeDataEventHandler(client);
    on(ev('transactions', 'user'));
    vi.advanceTimersByTime(500);
    expect(spy).not.toHaveBeenCalled();
    on(ev('transactions'));
    on(ev('transactions'));
    on(ev('research', 'system'));
    vi.advanceTimersByTime(500);
    const keys = spy.mock.calls.map((c) => (c[0] as { queryKey?: string[] }).queryKey?.[0]);
    expect(keys).toContain('transactions');
    expect(keys).toContain('reports');
    expect(keys).toContain('research');
    expect(keys).toContain('ai');
    expect(keys.filter((k) => k === 'transactions')).toHaveLength(1);
    vi.useRealTimers();
  });

  it('cliente MCP novo avisa na hora, em qualquer tela', () => {
    const toast = vi.spyOn(components, 'toast').mockImplementation(() => '');
    const on = makeDataEventHandler(new QueryClient());
    on({ ...ev('mcp', 'system'), cmd: 'mcp.client_new' });
    expect(toast).toHaveBeenCalledWith(expect.objectContaining({ tone: 'warning' }));
    on(ev('transactions'));
    expect(toast).toHaveBeenCalledTimes(1);
  });
});
