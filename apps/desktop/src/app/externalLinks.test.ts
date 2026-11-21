import { describe, expect, it } from 'vitest';

import { externalUrl } from './externalLinks';

describe('external links', () => {
  const base = 'tauri://localhost/#/ajustes?aba=fontes';
  it('only http(s) to another address goes to the system browser', () => {
    expect(externalUrl('https://brapi.dev/dashboard', base)).toBe('https://brapi.dev/dashboard');
    expect(externalUrl('http://www.bcb.gov.br', base)).toBe('http://www.bcb.gov.br/');
    expect(externalUrl('#/lancamentos', base)).toBeNull();
    expect(externalUrl('/carteira', base)).toBeNull();
    expect(externalUrl('mailto:ana@example.com', base)).toBeNull();
    expect(externalUrl('javascript:alert(1)', base)).toBeNull();
    expect(externalUrl(null, base)).toBeNull();
    expect(externalUrl('http://localhost:5179/x', 'http://localhost:5179/')).toBeNull();
  });
});
