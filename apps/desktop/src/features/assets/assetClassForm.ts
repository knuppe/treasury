/**
 * O que o cadastro de ativo pede para cada tipo: só os campos que fazem sentido para ele
 * (a bolsa para o que é negociado em bolsa, o CNPJ para fundos e previdência, o título para
 * o Tesouro, os termos para a renda fixa, o plano para a previdência). O resto fica de fora
 * ou em "Cotação e identificadores", recolhido.
 */
import type { AssetClass } from '@/api/generated/AssetClass';
import { t } from '@/i18n';

export interface ClassForm {
  /** Uma linha sobre o tipo, abaixo da escolha. */
  about: string;
  /** Busca de tickers (B3, EUA, outras bolsas, cripto) que preenche o formulário. */
  search: boolean;
  ticker: { label: string; hint: string; placeholder?: string } | null;
  name: { label: string; placeholder?: string };
  /** Bolsa do catálogo (dá moeda, país e símbolo da cotação). */
  exchange: boolean;
  /** Mercado em texto livre (só "Outros"). */
  market: boolean;
  /** Moeda e país: no formulário, em "Cotação e identificadores" ou fora. */
  currency: 'main' | 'advanced' | null;
  country: 'main' | 'advanced' | null;
  isin: boolean;
  cnpj: { label: string; hint: string } | null;
  treasuryCode: boolean;
  sector: { label: string; placeholder?: string } | null;
  segment: { label: string; placeholder?: string } | null;
  /** Termos de renda fixa: obrigatórios, opcionais (caixa para informar) ou fora. */
  terms: 'required' | 'optional' | null;
  /** Plano de previdência (VGBL/PGBL) no cadastro. */
  pension: boolean;
  /** Opções de fonte de cotação (`auto` = a da classe). */
  providers: readonly { value: string; label: string }[];
}

// Os textos vêm do catálogo na hora (`classForm()`): o app remonta quando o idioma muda.
const manual = () => ({ value: 'manual', label: t('ativos-fonte-manual') });
const none = () => ({ value: 'none', label: t('ativos-fonte-nenhuma') });
const yahoo = () => [{ value: 'auto', label: t('ativos-fonte-auto-yahoo') }, manual(), none()];
const cvm = () => [{ value: 'auto', label: t('ativos-fonte-auto-cvm') }, manual(), none()];

const sector = () => ({
  label: t('ativos-campo-setor'),
  placeholder: t('ativos-campo-setor-exemplo'),
});
const segment = (placeholder = t('ativos-opcional')) => ({
  label: t('ativos-campo-segmento'),
  placeholder,
});

function listed(about: string, ticker: string, extra: Partial<ClassForm> = {}): ClassForm {
  return {
    about,
    search: true,
    ticker: {
      label: t('ativos-col-ticker'),
      hint: t('ativos-exemplo', { exemplo: ticker }),
      placeholder: ticker,
    },
    name: { label: t('ativos-col-nome') },
    exchange: true,
    market: false,
    currency: 'advanced',
    country: 'advanced',
    isin: true,
    cnpj: null,
    treasuryCode: false,
    sector: sector(),
    segment: segment(),
    terms: null,
    pension: false,
    providers: yahoo(),
    ...extra,
  };
}

const NOT_LISTED: Omit<ClassForm, 'about' | 'name' | 'providers'> = {
  search: false,
  ticker: null,
  exchange: false,
  market: false,
  currency: null,
  country: null,
  isin: false,
  cnpj: null,
  treasuryCode: false,
  sector: null,
  segment: null,
  terms: null,
  pension: false,
};

const NO_SECTOR = { sector: null, segment: null };

const B3: readonly AssetClass[] = ['stock_br', 'fii', 'etf_br', 'bdr'];
const US: readonly AssetClass[] = ['stock_us', 'etf_us', 'reit'];
const INTL: readonly AssetClass[] = ['stock_intl', 'etf_intl'];

/** O que o cadastro pede para o tipo, no idioma atual. */
export function classForm(cls: AssetClass): ClassForm {
  const f = baseForm(cls);
  const [auto, ...rest] = f.providers;
  const withExtra = (extra: { value: string; label: string }[]): ClassForm => ({
    ...f,
    providers: [...(auto ? [auto] : []), ...extra, ...rest],
  });
  const yahooChoice = { value: 'yahoo', label: t('ativos-fonte-yahoo') };
  const eodhd = { value: 'eodhd', label: t('ativos-fonte-eodhd') };
  // Papéis da B3: a brapi.dev (com token) também pode ser a fonte da cotação.
  if (B3.includes(cls))
    return withExtra([
      { value: 'b3_cotahist', label: t('ativos-fonte-cotahist') },
      { value: 'brapi', label: t('ativos-fonte-brapi') },
      yahooChoice,
    ]);
  // Estados Unidos e outras bolsas: as fontes com a chave da própria pessoa, ou o Yahoo.
  if (US.includes(cls))
    return withExtra([{ value: 'tiingo', label: t('ativos-fonte-tiingo') }, eodhd, yahooChoice]);
  if (INTL.includes(cls)) return withExtra([eodhd, yahooChoice]);
  // Cripto: a automática é a DefiLlama; o Yahoo fica para quem preferir.
  if (cls === 'crypto') return withExtra([yahooChoice]);
  return f;
}

function baseForm(cls: AssetClass): ClassForm {
  switch (cls) {
    case 'stock_br':
      return listed(t('ativos-tipo-stock-br'), 'PETR4');
    case 'fii':
      return listed(t('ativos-tipo-fii'), 'HGLG11', {
        sector: null,
        segment: segment(t('ativos-segmento-fii-exemplo')),
      });
    case 'etf_br':
      return listed(t('ativos-tipo-etf-br'), 'BOVA11', NO_SECTOR);
    case 'bdr':
      return listed(t('ativos-tipo-bdr'), 'AAPL34');
    case 'stock_us':
      return listed(t('ativos-tipo-stock-us'), 'AAPL');
    case 'etf_us':
      return listed(t('ativos-tipo-etf-us'), 'VOO', NO_SECTOR);
    case 'reit':
      return listed(t('ativos-tipo-reit'), 'O', {
        sector: null,
        segment: segment(t('ativos-segmento-reit-exemplo')),
      });
    case 'stock_intl':
      return listed(t('ativos-tipo-stock-intl'), 'VOD');
    case 'etf_intl':
      return listed(t('ativos-tipo-etf-intl'), 'VWRL', NO_SECTOR);
    case 'crypto':
      return {
        ...NOT_LISTED,
        about: t('ativos-tipo-crypto'),
        search: true,
        ticker: {
          label: t('ativos-col-ticker'),
          hint: t('ativos-exemplo', { exemplo: 'BTC, ETH, SOL' }),
          placeholder: 'BTC',
        },
        name: { label: t('ativos-col-nome'), placeholder: 'Bitcoin' },
        currency: 'main',
        providers: [{ value: 'auto', label: t('ativos-fonte-auto-cripto') }, manual(), none()],
      };
    case 'treasury_br':
      return {
        ...NOT_LISTED,
        about: t('ativos-tipo-treasury-br'),
        name: { label: t('ativos-col-nome'), placeholder: 'Tesouro IPCA+ 2035' },
        treasuryCode: true,
        terms: 'optional',
        providers: [{ value: 'auto', label: t('ativos-fonte-auto-tesouro') }, manual(), none()],
      };
    case 'fixed_income':
      return {
        ...NOT_LISTED,
        about: t('ativos-tipo-fixed-income'),
        name: { label: t('ativos-col-nome'), placeholder: t('ativos-nome-cdb-exemplo') },
        cnpj: {
          label: t('ativos-cnpj-emissor'),
          hint: t('ativos-cnpj-emissor-dica'),
        },
        terms: 'required',
        providers: [{ value: 'auto', label: t('ativos-fonte-auto-curva') }, manual()],
      };
    case 'fund':
      return {
        ...NOT_LISTED,
        about: t('ativos-tipo-fund'),
        name: { label: t('ativos-nome-fundo') },
        cnpj: { label: t('ativos-cnpj-fundo'), hint: t('ativos-cnpj-fundo-dica') },
        segment: { label: t('ativos-categoria'), placeholder: t('ativos-categoria-fundo-exemplo') },
        providers: cvm(),
      };
    case 'pension':
      return {
        ...NOT_LISTED,
        about: t('ativos-tipo-pension'),
        name: { label: t('ativos-nome-plano'), placeholder: 'VGBL Brasilprev RV 30' },
        cnpj: {
          label: t('ativos-cnpj-fie'),
          hint: t('ativos-cnpj-fie-dica'),
        },
        pension: true,
        providers: cvm(),
      };
    case 'other':
      return {
        ...NOT_LISTED,
        about: t('ativos-tipo-other'),
        ticker: { label: t('ativos-codigo'), hint: t('ativos-opcional-ponto') },
        name: { label: t('ativos-col-nome') },
        market: true,
        currency: 'main',
        country: 'main',
        isin: true,
        providers: [
          { value: 'auto', label: t('ativos-fonte-auto-nenhuma') },
          { value: 'yahoo', label: t('ativos-fonte-yahoo-simbolo') },
          manual(),
          none(),
        ],
      };
  }
}

/** Tipos agrupados como aparecem na escolha, no idioma atual. */
export function typeGroups(): readonly { label: string; classes: readonly AssetClass[] }[] {
  return [
    { label: t('ativos-grupo-bolsa-br'), classes: ['stock_br', 'fii', 'etf_br', 'bdr'] },
    { label: t('ativos-grupo-renda-fixa'), classes: ['treasury_br', 'fixed_income'] },
    { label: t('ativos-grupo-fundos'), classes: ['fund', 'pension'] },
    {
      label: t('ativos-grupo-exterior'),
      classes: ['stock_us', 'etf_us', 'reit', 'stock_intl', 'etf_intl'],
    },
    { label: t('ativos-grupo-outros'), classes: ['crypto', 'other'] },
  ];
}

/** Opções de fonte de cotação da classe, mantendo a gravada no ativo. */
export function providerOptions(cls: AssetClass, current: string | null | undefined) {
  const base = classForm(cls).providers;
  if (!current || base.some((p) => p.value === current)) return base;
  const known: Record<string, string> = {
    yahoo: 'Yahoo Finance',
    b3_cotahist: 'B3 COTAHIST',
    tiingo: 'Tiingo',
    eodhd: 'EODHD',
    defillama: 'DefiLlama',
    tesouro: 'Tesouro Transparente',
    cvm: t('ativos-fonte-cota-cvm'),
    brapi: t('ativos-fonte-brapi'),
  };
  return [...base, { value: current, label: known[current] ?? current }];
}
