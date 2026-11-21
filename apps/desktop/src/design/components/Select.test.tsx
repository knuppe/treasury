import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { Select } from './Select';

const OPTIONS = [
  { value: 'a', label: 'Primeira' },
  { value: 'b', label: 'Segunda' },
];

describe('Select', () => {
  it('sem valor mostra o placeholder e ganhar valor depois não avisa do Radix', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const error = vi.spyOn(console, 'error').mockImplementation(() => {});
    const { rerender } = render(
      <Select
        aria-label="Opção"
        value={undefined}
        onValueChange={() => {}}
        options={OPTIONS}
        placeholder="Escolha"
      />,
    );
    expect(screen.getByRole('combobox', { name: 'Opção' }).textContent).toContain('Escolha');
    rerender(
      <Select
        aria-label="Opção"
        value="b"
        onValueChange={() => {}}
        options={OPTIONS}
        placeholder="Escolha"
      />,
    );
    expect(screen.getByRole('combobox', { name: 'Opção' }).textContent).toContain('Segunda');
    const calls = [...warn.mock.calls, ...error.mock.calls].map((c) => String(c[0]));
    expect(calls.filter((c) => /controlled/i.test(c))).toEqual([]);
    warn.mockRestore();
    error.mockRestore();
  });
});
