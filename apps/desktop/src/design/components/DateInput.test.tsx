import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it } from 'vitest';

import { DateInput, maskDate } from './DateInput';
import { Field } from './Field';

function Harness({ initial = null }: { initial?: string | null }) {
  const [iso, setIso] = useState<string | null>(initial);
  return (
    <>
      <Field label="Data">
        <DateInput value={iso} onValueChange={setIso} />
      </Field>
      <output data-testid="iso">{iso ?? 'null'}</output>
    </>
  );
}

describe('maskDate', () => {
  it('insere as barras enquanto se digita', () => {
    expect(maskDate('1')).toBe('1');
    expect(maskDate('14')).toBe('14/');
    expect(maskDate('1410')).toBe('14/10/');
    expect(maskDate('14102026')).toBe('14/10/2026');
    expect(maskDate('141020261')).toBe('14/10/2026');
  });

  it('completa dia e mês de um dígito quando a barra é digitada', () => {
    expect(maskDate('1/')).toBe('01/');
    expect(maskDate('1/2/2026')).toBe('01/02/2026');
  });

  it('converte ISO colado', () => {
    expect(maskDate('2026-10-14')).toBe('14/10/2026');
  });
});

describe('<DateInput>', () => {
  it('converte dd/mm/aaaa em ISO', async () => {
    const user = userEvent.setup();
    render(<Harness />);
    const input = screen.getByLabelText<HTMLInputElement>('Data');
    await user.type(input, '14102026');
    expect(input.value).toBe('14/10/2026');
    expect(screen.getByTestId('iso').textContent).toBe('2026-10-14');
  });

  it('mostra a data inicial no formato brasileiro', () => {
    render(<Harness initial="2026-01-05" />);
    expect(screen.getByLabelText<HTMLInputElement>('Data').value).toBe('05/01/2026');
  });

  it('marca datas inexistentes como inválidas', async () => {
    const user = userEvent.setup();
    render(<Harness />);
    const input = screen.getByLabelText<HTMLInputElement>('Data');
    await user.type(input, '31022026');
    expect(input.getAttribute('aria-invalid')).toBe('true');
    expect(screen.getByTestId('iso').textContent).toBe('null');
  });

  it('deixa apagar a barra com backspace', async () => {
    const user = userEvent.setup();
    render(<Harness />);
    const input = screen.getByLabelText<HTMLInputElement>('Data');
    await user.type(input, '14');
    expect(input.value).toBe('14/');
    await user.type(input, '{Backspace}');
    expect(input.value).toBe('14');
  });
});
