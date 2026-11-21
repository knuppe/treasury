import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { Field } from './Field';
import { MoneyInput } from './MoneyInput';

function Harness({
  initial = null,
  onCents,
  allowNegative,
}: {
  initial?: number | null;
  onCents?: (c: number | null) => void;
  allowNegative?: boolean;
}) {
  const [cents, setCents] = useState<number | null>(initial);
  return (
    <>
      <Field label="Valor">
        <MoneyInput
          value={cents}
          allowNegative={allowNegative}
          onValueChange={(c) => {
            setCents(c);
            onCents?.(c);
          }}
        />
      </Field>
      <output data-testid="cents">{cents === null ? 'null' : String(cents)}</output>
      <button type="button" onClick={() => setCents(990)}>
        definir
      </button>
    </>
  );
}

describe('<MoneyInput>', () => {
  it('aceita 1.234,56 e devolve centavos inteiros', async () => {
    const user = userEvent.setup();
    const onCents = vi.fn();
    render(<Harness onCents={onCents} />);
    await user.type(screen.getByLabelText('Valor'), '1.234,56');
    expect(screen.getByTestId('cents').textContent).toBe('123456');
    expect(onCents).toHaveBeenLastCalledWith(123456);
  });

  it('normaliza o texto ao sair do campo', async () => {
    const user = userEvent.setup();
    render(<Harness />);
    const input = screen.getByLabelText<HTMLInputElement>('Valor');
    await user.type(input, '1234,5');
    await user.tab();
    expect(input.value).toBe('1.234,50');
    expect(screen.getByTestId('cents').textContent).toBe('123450');
  });

  it('mostra o valor inicial formatado, com o prefixo da moeda', () => {
    render(<Harness initial={500050} />);
    expect(screen.getByLabelText<HTMLInputElement>('Valor').value).toBe('5.000,50');
    expect(screen.getByText('R$')).toBeTruthy();
  });

  it('ignora letras e marca texto inválido com aria-invalid', async () => {
    const user = userEvent.setup();
    render(<Harness />);
    const input = screen.getByLabelText<HTMLInputElement>('Valor');
    await user.type(input, '12a,345');
    expect(input.value).toBe('12,345');
    expect(input.getAttribute('aria-invalid')).toBe('true');
    expect(screen.getByTestId('cents').textContent).toBe('null');
  });

  it('não aceita negativos por padrão', async () => {
    const user = userEvent.setup();
    render(<Harness />);
    const input = screen.getByLabelText<HTMLInputElement>('Valor');
    await user.type(input, '-10');
    expect(input.value).toBe('10');
    expect(screen.getByTestId('cents').textContent).toBe('1000');
  });

  it('aceita negativos quando permitido', async () => {
    const user = userEvent.setup();
    render(<Harness allowNegative />);
    await user.type(screen.getByLabelText('Valor'), '-10,5');
    expect(screen.getByTestId('cents').textContent).toBe('-1050');
  });

  it('acompanha mudanças de valor vindas de fora', async () => {
    const user = userEvent.setup();
    render(<Harness initial={100} />);
    await user.click(screen.getByRole('button', { name: 'definir' }));
    expect(screen.getByLabelText<HTMLInputElement>('Valor').value).toBe('9,90');
  });

  it('esvaziar o campo devolve null', async () => {
    const user = userEvent.setup();
    render(<Harness initial={100} />);
    await user.clear(screen.getByLabelText('Valor'));
    expect(screen.getByTestId('cents').textContent).toBe('null');
  });
});
