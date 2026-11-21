import { act, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import { MINUS } from '@/lib/format';
import { UI_DEFAULTS, useUi } from '@/state/ui';

import { Money, Private, PRIVACY_MASK } from './Money';

describe('<Money>', () => {
  beforeEach(() => {
    useUi.setState({ ...UI_DEFAULTS });
  });

  it('formats cents in pt-BR', () => {
    render(<Money cents={123456} />);
    expect(screen.getByText('R$ 1.234,56')).toBeTruthy();
  });

  it('shows the ISO code of other currencies', () => {
    render(<Money cents={9900} currency="USD" />);
    expect(screen.getByText('USD 99,00')).toBeTruthy();
  });

  it('gain and loss do not depend on color alone: sign and arrow', () => {
    const { container } = render(
      <>
        <Money cents={2500} tone="auto" trend />
        <Money cents={-2500} tone="auto" trend />
      </>,
    );
    const [gain, loss] = Array.from(container.querySelectorAll('[data-money]'));
    expect(gain!.textContent).toBe('▲+R$ 25,00');
    expect(gain!.className).toContain('text-positive');
    expect(loss!.textContent).toBe(`▼${MINUS}R$ 25,00`);
    expect(loss!.className).toContain('text-negative');
  });

  it('marks estimated values with ≈ and italics', () => {
    const { container } = render(<Money cents={100} estimated />);
    const el = container.querySelector('[data-money]')!;
    expect(el.textContent).toContain('≈');
    expect(el.className).toContain('italic');
  });

  it('in privacy mode replaces the value with •••••', () => {
    useUi.setState({ privacy: true });
    const { container } = render(<Money cents={123456} />);
    expect(container.textContent).toContain(PRIVACY_MASK);
    expect(container.textContent).not.toMatch(/\d/);
    expect(screen.getByText('valor oculto')).toBeTruthy();
  });

  it('reacts immediately when privacy is toggled', () => {
    const { container } = render(<Money cents={500} />);
    expect(container.textContent).toBe('R$ 5,00');
    act(() => useUi.getState().togglePrivacy());
    expect(container.textContent).not.toContain('5,00');
    act(() => useUi.getState().togglePrivacy());
    expect(container.textContent).toBe('R$ 5,00');
  });

  it('<Private> hides other values in privacy mode', () => {
    const { container } = render(<Private>1.234 cotas</Private>);
    expect(container.textContent).toBe('1.234 cotas');
    act(() => {
      useUi.setState({ privacy: true });
    });
    expect(container.textContent).not.toContain('1.234');
    expect(container.textContent).toContain(PRIVACY_MASK);
  });

  it('`reveal` ignores privacy when the user asked to see', () => {
    useUi.setState({ privacy: true });
    render(<Money cents={500} reveal />);
    expect(screen.getByText('R$ 5,00')).toBeTruthy();
  });
});
