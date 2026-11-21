import { act, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { Institution } from '@/api/generated/Institution';
import { mockCall } from '@/api/mock';
import { renderApp } from '@/test/renderApp';

import { institutionUrl } from './InstitutionSiteLink';

vi.mock('@/charts/EChart', () => import('@/test/echartsMock'));

describe('institution site', () => {
  it('becomes an http(s) address; the rest is left out', () => {
    expect(institutionUrl('nubank.com.br')).toBe('https://nubank.com.br/');
    expect(institutionUrl(' https://www.Itau.com.br/pf ')).toBe('https://www.itau.com.br/pf');
    expect(institutionUrl('http://banco.com.br')).toBe('http://banco.com.br/');
    expect(institutionUrl('banco.com.br:8443/x')).toBe('https://banco.com.br:8443/x');
    expect(institutionUrl('javascript:alert(1)')).toBeNull();
    expect(institutionUrl('https://user:pw@banco.com.br')).toBeNull();
    expect(institutionUrl('localhost')).toBeNull();
    expect(institutionUrl('')).toBeNull();
    expect(institutionUrl(null)).toBeNull();
  });

  it('the account card leads to the site in the system browser, without leaving the screen', async () => {
    const { router, queryClient } = renderApp('unlocked', {}, '#/contas');
    const institutions = (await mockCall('household.list_institutions', {})) as Institution[];
    const inter = institutions.find((i) => i.name === 'Banco Inter')!;
    await mockCall('household.update_institution', {
      id: inter.id,
      website: 'https://www.bancointer.com.br',
    });
    await act(() => queryClient.invalidateQueries());
    const link = (
      await screen.findAllByRole('link', { name: 'Abrir o site de Banco Inter no navegador' })
    )[0]!;
    expect(link.getAttribute('href')).toBe('https://www.bancointer.com.br/');
    expect(link.getAttribute('target')).toBe('_blank');
    expect(link.getAttribute('rel')).toContain('noopener');
    expect(router.state.location.pathname).toBe('/contas');
  });
});
