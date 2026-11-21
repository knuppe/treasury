/**
 * Mock do catálogo de bolsas (`market.exchanges`, `market.exchange_save`,
 * `market.exchange_delete`): uma parte do seed da V85 do núcleo, com o mesmo formato.
 */
import type { Exchange } from '../generated/Exchange';
import type { ExchangeInput } from '../generated/ExchangeInput';
import { mockFail, onMockReset, registerMockHandlers, requireUnlocked } from '../mockRegistry';

const PTAX = ['AUD', 'CAD', 'CHF', 'DKK', 'EUR', 'GBP', 'JPY', 'NOK', 'SEK', 'USD'];
/** Moedas com câmbio de referência do BCE, que chegam ao real cruzando pelo euro. */
const ECB = [
  ...['CNY', 'CZK', 'HKD', 'HUF', 'IDR', 'ILS', 'INR', 'ISK', 'KRW', 'MXN', 'MYR', 'NZD', 'PHP'],
  ...['PLN', 'RON', 'SGD', 'THB', 'TRY', 'ZAR'],
];
const fxAutomatic = (c: string) => c === 'BRL' || PTAX.includes(c) || ECB.includes(c);

type Row = [
  code: string,
  name: string,
  country: string,
  currency: string,
  suffix: string,
  mic: string,
  tz: string,
  open: string,
  close: string,
  lunch: [string, string] | null,
  aliases: string,
];

const SEED: Row[] = [
  [
    'B3',
    'B3 (Brasil, Bolsa, Balcão)',
    'BR',
    'BRL',
    '.SA',
    'BVMF',
    'America/Sao_Paulo',
    '10:00',
    '18:00',
    null,
    'SAO,BOVESPA',
  ],
  [
    'NYSE',
    'New York Stock Exchange',
    'US',
    'USD',
    '',
    'XNYS',
    'America/New_York',
    '09:30',
    '16:00',
    null,
    'NYQ',
  ],
  [
    'NASDAQ',
    'Nasdaq',
    'US',
    'USD',
    '',
    'XNAS',
    'America/New_York',
    '09:30',
    '16:00',
    null,
    'NMS,NGM,NCM',
  ],
  [
    'AMEX',
    'NYSE American',
    'US',
    'USD',
    '',
    'XASE',
    'America/New_York',
    '09:30',
    '16:00',
    null,
    'ASE',
  ],
  [
    'ARCA',
    'NYSE Arca',
    'US',
    'USD',
    '',
    'ARCX',
    'America/New_York',
    '09:30',
    '16:00',
    null,
    'PCX,NYSE ARCA',
  ],
  [
    'TSX',
    'Toronto Stock Exchange',
    'CA',
    'CAD',
    '.TO',
    'XTSE',
    'America/Toronto',
    '09:30',
    '16:00',
    null,
    'TOR',
  ],
  [
    'BMV',
    'Bolsa Mexicana de Valores',
    'MX',
    'MXN',
    '.MX',
    'XMEX',
    'America/Mexico_City',
    '08:30',
    '15:00',
    null,
    'MEX',
  ],
  [
    'BYMA',
    'Bolsas y Mercados Argentinos',
    'AR',
    'ARS',
    '.BA',
    'XBUE',
    'America/Argentina/Buenos_Aires',
    '11:00',
    '17:00',
    null,
    'BUE',
  ],
  [
    'LSE',
    'London Stock Exchange',
    'GB',
    'GBP',
    '.L',
    'XLON',
    'Europe/London',
    '08:00',
    '16:30',
    null,
    'LON',
  ],
  [
    'XETRA',
    'Xetra (Deutsche Börse)',
    'DE',
    'EUR',
    '.DE',
    'XETR',
    'Europe/Berlin',
    '09:00',
    '17:30',
    null,
    'GER',
  ],
  [
    'EPA',
    'Euronext Paris',
    'FR',
    'EUR',
    '.PA',
    'XPAR',
    'Europe/Paris',
    '09:00',
    '17:30',
    null,
    'PAR',
  ],
  [
    'AMS',
    'Euronext Amsterdã',
    'NL',
    'EUR',
    '.AS',
    'XAMS',
    'Europe/Amsterdam',
    '09:00',
    '17:30',
    null,
    '',
  ],
  [
    'ELI',
    'Euronext Lisboa',
    'PT',
    'EUR',
    '.LS',
    'XLIS',
    'Europe/Lisbon',
    '08:00',
    '16:30',
    null,
    'LIS',
  ],
  [
    'BME',
    'Bolsa de Madri (BME)',
    'ES',
    'EUR',
    '.MC',
    'XMAD',
    'Europe/Madrid',
    '09:00',
    '17:30',
    null,
    'MCE',
  ],
  [
    'MIL',
    'Euronext Milão (Borsa Italiana)',
    'IT',
    'EUR',
    '.MI',
    'XMIL',
    'Europe/Rome',
    '09:00',
    '17:30',
    null,
    '',
  ],
  [
    'SIX',
    'SIX Swiss Exchange',
    'CH',
    'CHF',
    '.SW',
    'XSWX',
    'Europe/Zurich',
    '09:00',
    '17:30',
    null,
    'EBS',
  ],
  [
    'JSE',
    'Johannesburg Stock Exchange',
    'ZA',
    'ZAR',
    '.JO',
    'XJSE',
    'Africa/Johannesburg',
    '09:00',
    '17:00',
    null,
    'JNB',
  ],
  [
    'NSE',
    'National Stock Exchange of India',
    'IN',
    'INR',
    '.NS',
    'XNSE',
    'Asia/Kolkata',
    '09:15',
    '15:30',
    null,
    'NSI',
  ],
  [
    'HKEX',
    'Bolsa de Hong Kong',
    'HK',
    'HKD',
    '.HK',
    'XHKG',
    'Asia/Hong_Kong',
    '09:30',
    '16:00',
    ['12:00', '13:00'],
    'HKG',
  ],
  [
    'TSE',
    'Bolsa de Tóquio (JPX)',
    'JP',
    'JPY',
    '.T',
    'XTKS',
    'Asia/Tokyo',
    '09:00',
    '15:30',
    ['11:30', '12:30'],
    'JPX',
  ],
  [
    'KRX',
    'Korea Exchange (KOSPI)',
    'KR',
    'KRW',
    '.KS',
    'XKRX',
    'Asia/Seoul',
    '09:00',
    '15:30',
    null,
    'KSC',
  ],
  [
    'TWSE',
    'Bolsa de Taiwan',
    'TW',
    'TWD',
    '.TW',
    'XTAI',
    'Asia/Taipei',
    '09:00',
    '13:30',
    null,
    'TAI',
  ],
  [
    'SGX',
    'Singapore Exchange',
    'SG',
    'SGD',
    '.SI',
    'XSES',
    'Asia/Singapore',
    '09:00',
    '17:00',
    ['12:00', '13:00'],
    'SES',
  ],
  [
    'ASX',
    'Australian Securities Exchange',
    'AU',
    'AUD',
    '.AX',
    'XASX',
    'Australia/Sydney',
    '10:00',
    '16:00',
    null,
    '',
  ],
];

function fromRow(r: Row): Exchange {
  return {
    code: r[0],
    name: r[1],
    country: r[2],
    currency: r[3],
    yahoo_suffix: r[4],
    mic: r[5],
    timezone: r[6],
    open_time: r[7],
    close_time: r[8],
    lunch_start: r[9]?.[0] ?? null,
    lunch_end: r[9]?.[1] ?? null,
    trading_days: '12345',
    aliases: r[10] ? r[10].split(',') : [],
    builtin: true,
    notes: r[0] === 'LSE' ? 'O Yahoo cota em pence (GBp); o Treasury converte para libras.' : null,
    fx_automatic: fxAutomatic(r[3]),
  };
}

let exchanges: Exchange[] = SEED.map(fromRow);
onMockReset(() => {
  exchanges = SEED.map(fromRow);
});

export function allExchanges(): Exchange[] {
  return exchanges;
}

/** Bolsa pelo código, nome ou apelido (`market::exchange::resolve`). */
export function resolveExchange(market: string | null | undefined): Exchange | null {
  const m = (market ?? '').trim().toUpperCase();
  if (!m) return null;
  return (
    exchanges.find((e) => e.code === m) ??
    exchanges.find(
      (e) => e.name.toUpperCase() === m || e.aliases.some((a) => a.toUpperCase() === m),
    ) ??
    null
  );
}

/** Bolsa de um símbolo do Yahoo (sufixo; sem sufixo, código da bolsa no Yahoo). */
export function exchangeForYahoo(symbol: string, yahooExchange?: string): Exchange | null {
  const dot = symbol.lastIndexOf('.');
  if (dot > 0) {
    const suffix = symbol.slice(dot).toUpperCase();
    const hit = exchanges.find((e) => e.yahoo_suffix && e.yahoo_suffix === suffix);
    if (hit) return hit;
  }
  return yahooExchange ? resolveExchange(yahooExchange) : null;
}

function text(v: unknown): string {
  return typeof v === 'string' ? v.trim() : '';
}

registerMockHandlers({
  'market.exchanges': () => {
    requireUnlocked();
    return exchanges;
  },

  'market.exchange_save': (raw) => {
    requireUnlocked();
    const i = raw as unknown as ExchangeInput;
    const code = text(i.code).toUpperCase();
    if (!/^[A-Z0-9_]{1,12}$/.test(code)) {
      mockFail('invalid', 'código da bolsa: até 12 letras e números (ex.: LSE)');
    }
    if (!text(i.name)) mockFail('invalid', 'informe o nome da bolsa');
    const country = text(i.country).toUpperCase();
    if (!/^[A-Z]{2}$/.test(country)) mockFail('invalid', 'país: código ISO de 2 letras (ex.: GB)');
    const currency = text(i.currency).toUpperCase();
    if (!/^[A-Z]{3}$/.test(currency)) mockFail('invalid', `moeda inválida: ${currency}`);
    try {
      new Intl.DateTimeFormat('en-US', { timeZone: text(i.timezone) });
    } catch {
      mockFail('invalid', `fuso horário desconhecido: ${text(i.timezone)}`);
    }
    const hhmm = /^\d{1,2}:\d{2}$/;
    if (!hhmm.test(text(i.open_time)) || !hhmm.test(text(i.close_time))) {
      mockFail('invalid', 'horário inválido (use HH:MM)');
    }
    const pad = (t: string) => t.padStart(5, '0');
    if (pad(text(i.open_time)) >= pad(text(i.close_time))) {
      mockFail('invalid', 'a abertura precisa ser antes do fechamento');
    }
    const prev = exchanges.find((e) => e.code === code);
    const next: Exchange = {
      code,
      name: text(i.name),
      country,
      currency,
      yahoo_suffix: text(i.yahoo_suffix).toUpperCase(),
      mic: text(i.mic).toUpperCase() || null,
      timezone: text(i.timezone),
      open_time: pad(text(i.open_time)),
      close_time: pad(text(i.close_time)),
      lunch_start: text(i.lunch_start) || null,
      lunch_end: text(i.lunch_end) || null,
      trading_days: text(i.trading_days) || '12345',
      aliases: (i.aliases ?? []).map((a) => a.trim()).filter(Boolean),
      builtin: prev?.builtin ?? false,
      notes: text(i.notes) || null,
      fx_automatic: fxAutomatic(currency),
    };
    exchanges = prev ? exchanges.map((e) => (e.code === code ? next : e)) : [...exchanges, next];
    return next;
  },

  'market.exchange_delete': (args) => {
    requireUnlocked();
    const code = text(args.code).toUpperCase();
    const e = exchanges.find((x) => x.code === code);
    if (!e) mockFail('not_found', `Não encontrado: bolsa ${code}.`);
    if (e.builtin) {
      mockFail(
        'conflict',
        'bolsas do catálogo não podem ser apagadas (ajuste o horário, se precisar)',
      );
    }
    exchanges = exchanges.filter((x) => x.code !== code);
    return null;
  },
});
