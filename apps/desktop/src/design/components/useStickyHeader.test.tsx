import { fireEvent, render } from '@testing-library/react';
import { useRef } from 'react';
import { describe, expect, it } from 'vitest';

import { useStickyHeader } from './useStickyHeader';

function Table() {
  const box = useRef<HTMLDivElement>(null);
  useStickyHeader(box);
  return (
    <div data-testid="scroller" style={{ overflowY: 'auto' }}>
      <div ref={box} data-testid="box">
        <table>
          <thead>
            <tr>
              <th>Provedor</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>BCB</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  );
}

const rect = (top: number, height: number) =>
  ({ top, height, bottom: top + height, left: 0, right: 0, width: 0, x: 0, y: top }) as DOMRect;

describe('sticky table header', () => {
  it('floats with a cover while the table scrolls past; pinned at the table end, without cover', () => {
    const { getByTestId, container } = render(<Table />);
    const scroller = getByTestId('scroller');
    const box = getByTestId('box');
    const head = container.querySelector('thead')!;
    Object.defineProperty(head, 'offsetHeight', { value: 32, configurable: true });
    scroller.getBoundingClientRect = () => rect(100, 600);

    // The table started 300 px above the scrolling area and still has a lot below.
    box.getBoundingClientRect = () => rect(-200, 1000);
    fireEvent.scroll(scroller);
    expect(head.dataset.floating).toBe('');
    expect(container.querySelector('th')!.style.transform).toBe('translateY(300px)');

    // Near the end: the header stops at the last row, with real rows above it.
    box.getBoundingClientRect = () => rect(-2000, 1000);
    fireEvent.scroll(scroller);
    expect(head.dataset.floating).toBe('end');

    // Back in place: no transform.
    box.getBoundingClientRect = () => rect(200, 1000);
    fireEvent.scroll(scroller);
    expect('floating' in head.dataset).toBe(false);
    expect(container.querySelector('th')!.style.transform).toBe('');
  });
});
