import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { renderApp } from '@/test/renderApp';
import { mockCall } from '@/api/mock';

import {
  resetDownloadOffers,
  simulateDownloadOffer,
  simulatedTrash,
  useDownloadOffers,
  type DownloadOffer,
} from './downloadsWatch';
import { useImportHub } from './importHub';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

const OFX = btoa('OFXHEADER:100\n<OFX></OFX>');

const offer = (name: string, label = 'Extrato OFX'): DownloadOffer => ({
  path: `/Users/teste/Downloads/${name}`,
  name,
  size: 2048,
  kind: 'statement',
  label,
  format: 'ofx',
});

afterEach(() => {
  resetDownloadOffers();
  act(() => useImportHub.setState({ open: false, pending: [] }));
});

describe('Downloads folder notice', () => {
  it('groups the files into one card and opens the import with them loaded', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('heading', { level: 1, name: /Lançamentos/ });
    expect(screen.queryByRole('region', { name: 'Arquivos novos em Downloads' })).toBeNull();

    act(() => {
      simulateDownloadOffer(offer('extrato.ofx'), OFX);
      simulateDownloadOffer(offer('fatura.ofx', 'Fatura de cartão OFX'), OFX);
      // Unknown format does not raise a notice.
      simulateDownloadOffer({ ...offer('foto.pdf'), kind: 'unknown' }, OFX);
    });
    const card = await screen.findByRole('region', { name: 'Arquivos novos em Downloads' });
    expect(within(card).getByText('2 arquivos novos em Downloads')).toBeTruthy();
    expect(within(card).getByText('extrato.ofx')).toBeTruthy();
    expect(within(card).getByText(/Fatura de cartão OFX/)).toBeTruthy();
    expect(within(card).queryByText('foto.pdf')).toBeNull();

    await user.click(within(card).getByRole('button', { name: 'Importar 2' }));
    const dialog = await screen.findByRole('dialog', { name: 'Importar arquivos' });
    const statements = await within(dialog).findByRole('region', { name: 'Extratos e faturas' });
    expect(within(statements).getByText('extrato.ofx')).toBeTruthy();
    expect(within(statements).getByText('fatura.ofx')).toBeTruthy();
    expect(screen.queryByRole('region', { name: 'Arquivos novos em Downloads' })).toBeNull();

    // The same file is not offered again in this session.
    act(() => simulateDownloadOffer(offer('extrato.ofx'), OFX));
    expect(useDownloadOffers.getState().offers).toHaveLength(0);
  });

  it('"Agora não" dismisses until the app is closed', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/lancamentos');
    await screen.findByRole('heading', { level: 1, name: /Lançamentos/ });
    act(() => simulateDownloadOffer(offer('extrato.ofx'), OFX));
    const card = await screen.findByRole('region', { name: 'Arquivos novos em Downloads' });
    expect(within(card).getByText('Novo arquivo em Downloads')).toBeTruthy();
    expect(within(card).getByText(/Extrato OFX/)).toBeTruthy();
    await user.click(within(card).getByRole('button', { name: 'Agora não' }));
    await waitFor(() =>
      expect(screen.queryByRole('region', { name: 'Arquivos novos em Downloads' })).toBeNull(),
    );
    expect(useImportHub.getState().open).toBe(false);
    act(() => simulateDownloadOffer(offer('extrato.ofx'), OFX));
    expect(screen.queryByRole('region', { name: 'Arquivos novos em Downloads' })).toBeNull();
    // A different file shows up normally.
    act(() => simulateDownloadOffer(offer('outro.ofx'), OFX));
    expect(await screen.findByRole('region', { name: 'Arquivos novos em Downloads' })).toBeTruthy();
  });

  it('with "Apagar de Downloads" on, the imported receipt goes to the Trash', async () => {
    const user = userEvent.setup();
    renderApp('unlocked', {}, '#/carteira');
    await screen.findByRole('heading', { level: 1 });
    await mockCall('settings.set', { watch_downloads: true });
    const note: DownloadOffer = {
      ...offer('nota-777.pdf', 'Nota de corretagem (SINACOR)'),
      kind: 'brokerage_note',
      format: 'pdf',
    };
    act(() => simulateDownloadOffer(note, btoa('%PDF-1.4 nota')));
    const card = await screen.findByRole('region', { name: 'Arquivos novos em Downloads' });
    const del = await within(card).findByRole('checkbox', {
      name: 'Apagar de Downloads depois de importar',
    });
    await user.click(del);
    await waitFor(async () =>
      expect(
        ((await mockCall('settings.get', {})) as { downloads_delete_after_import: boolean })
          .downloads_delete_after_import,
      ).toBe(true),
    );
    await user.click(within(card).getByRole('button', { name: 'Importar' }));
    const hub = await screen.findByRole('dialog', { name: 'Importar arquivos' });
    await user.click(await within(hub).findByRole('button', { name: /Importar nota/ }));
    const notes = await screen.findByRole('dialog', { name: 'Importar notas de corretagem' });
    await user.click(within(notes).getByRole('button', { name: 'Ler notas' }));
    const gravar = await within(notes).findByRole('button', { name: /^Gravar \d+ operação/ });
    expect(simulatedTrash).toEqual([]);
    await user.click(gravar);
    await waitFor(() => expect(simulatedTrash).toEqual([note.path]));
    expect(await screen.findByText(/foram para a Lixeira/)).toBeTruthy();
  });
});
