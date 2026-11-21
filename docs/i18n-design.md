# Languages

Treasury speaks pt-BR (the source language), English and Spanish (neutral Latin American), with
pt-PT as a partial variant that falls back to pt-BR for whatever is missing. The language belongs to
the installation (to the computer's user), separate from the family's country: it is chosen on the
first screen, before the terms, and can be changed in Settings without restarting. The interface and
the core use the same catalog format, Fluent (`.ftl`), in the `locales/` folder; the tone guide for
each language is in [`locales/GUIDE.md`](../locales/GUIDE.md).

## Principles

1. **Language is not country.** Country decides currency, taxes, data sources and visible screens ([countries.md](countries.md)). Language decides only the words and the format of numbers and dates.
2. **No mixed text.** If a sentence has no translation, it falls back whole to the fallback language, never half a sentence.
3. **Plain language in every language.** The audience includes non-technical families: the translation follows the same clear tone as pt-BR, not the banking jargon of the other country.
4. **No network and no tracking.** The catalogs ship inside the app. No downloading translations and no measuring which language people use.
5. **User data is not translated automatically.** Account name, a category the person created or renamed, agent memory: they stay as they were written.
6. **Official terms stay in the original.** DARF, IRRF, INSS, Pix, FGTS, 13º, IRS, FICA, NIF keep the official name in any language, with a short explanation in the language's glossary (e.g., in English, "DARF (Brazilian federal tax payment slip)").

## Choosing the language

### Where it is stored

The language is stored in the installation's `config.toml` (the `locale` field, which already exists), outside the vault. Reasons:

- The lock screen, the vault creation and the terms appear before any vault is open.
- On one computer, the vaults belong to the same person; family members have no login of their own (D6).
- The MCP server and background notifications read the same file.

The `locale` setting inside the vault (`AppSettings`) ceases to exist. Since it was never really used, it can simply be ignored; no data migration is needed.

### First launch

1. The app reads the system language (`navigator.language` in the WebView, with the macOS language as a fallback).
2. The first screen, before the terms, shows the language picker already set to the guess: "Português (Brasil)", "English", "Español" (and "Português (Portugal)" as a variant). Each language is written in its own language, in a dropdown where typing searches by name, so that future languages fit. Each language also shows the flag of the variant's country: Brazil for pt-BR, Portugal for pt-PT, United Kingdom for en and Spain for es (the countries where the language was born). The flags are embedded SVGs (flag-icons, MIT), because Windows does not draw emoji flags.
3. The terms appear in the chosen language.
4. When the vault is created, the country is suggested by the system, independent of the language.

### Afterward

- Settings › Appearance and use › Language, with immediate switching: the interface redraws, the core starts answering in the new language, and the agent's next reply already comes out in the new language.
- When the language changes, if the vault still has default categories the person did not rename, the app offers once: "Translate the default categories too?" (see "Seeded data").

### Fallback rule

| Chosen language | If a text is missing, use |
|---|---|
| pt-BR | (it is the source language, always complete) |
| pt-PT | pt-BR |
| en | pt-BR (only for safety; CI requires complete English) |
| es | en, then pt-BR (only for safety; CI requires complete Spanish) |
| pt (short code) | same as pt-BR |
| future ones | en, then pt-BR |

## Catalog format: Fluent

### Why Fluent

| Option | In favor | Against |
|---|---|---|
| **Fluent (`.ftl`), on both sides** (recommended) | A single format for React and Rust; plural, gender and per-language variations written by the translator, without touching code; readable by non-programmers; mature libraries on both sides (`fluent-bundle` in Rust, `@fluent/bundle` in TypeScript); supported by Weblate and Pontoon | Less popular in React than i18next; we need a small generator to get typed keys |
| i18next (JSON) in the interface + rust-i18n (YAML) in the core | Very well known in React | Two formats, two plural rules, two workflows for translators |
| ICU MessageFormat (FormatJS) on both sides | Industry standard | Support in Rust is still weak for complete messages |

An example of how it looks:

```ftl
# locales/pt-BR/contas.ftl
contas-titulo = Contas
contas-vazias = Nenhuma conta ainda. Comece pela conta onde seu salário cai.
contas-saldo-negativo = { $nome } ficou no vermelho { $dias ->
    [one] 1 dia
   *[other] { $dias } dias
}
```

```ftl
# locales/en/contas.ftl
contas-titulo = Accounts
contas-vazias = No accounts yet. Start with the account your salary goes into.
contas-saldo-negativo = { $nome } was overdrawn for { $dias ->
    [one] 1 day
   *[other] { $dias } days
}
```

### Organization

```
locales/
  pt-BR/   comum.ftl  casca.ftl  contas.ftl  lancamentos.ftl ... erros.ftl  insights.ftl  notificacoes.ftl
  en/      (mesmos arquivos)
  pt-PT/   (mesmos arquivos)
  conteudo/
    pt-BR/ termos.md  glossario.md  dicas.md
    en/ ...
    pt-PT/ ...
```

- One file per area, with the same name in every language, so that two threads working on different areas do not conflict in the same file.
- Keys in pt-BR and without accents (`contas-titulo`), prefixed by the area. pt-BR is the source language: whoever writes code writes the text in pt-BR and the key; the translation comes afterward.
- Long texts (terms, glossary, tips, playbooks) do not become keys: they stay as Markdown per language in `locales/conteudo/`.
- The interface and the core use the same folder. The core embeds the `.ftl` files at compile time (`build.rs` generates the list and `fluent-bundle` assembles the messages); the interface imports with Vite and loads only the active language.

### In the interface (React)

- An `I18nProvider` at the top of the app and a `useT()` hook that returns `t('contas-titulo', { nome })`.
- A script (in the same spirit as `ts-rs`, which already generates the types) generates `src/i18n/keys.ts` from pt-BR (`npm run i18n:keys`): a wrong key or a missing argument becomes a `tsc -b` error, which already runs in `release.sh`.
- ECharts receives the language through `registerLocale` (month names on the axes, zoom buttons).
- `<html lang>` follows the language (`index.html` starts with `pt-BR`), which helps screen readers and hyphenation.

### In the core (Rust)

- An `i18n` module with `t(loc, "chave", args)` and the current language kept in the app state (read from `config.toml` and changed by the `system.set_locale` command).
- Every call that arrives through the interface, the MCP or the routines uses the installation's current language.
- A core test checks that every key used in the code exists in pt-BR (a source code scan, like the tests that already check the list of MCP tools).

## Numbers, dates and currency

`lib/format.ts` does the formatting by hand, without `Intl`, to give the same result in any WebView (WebKit on macOS, WebKitGTK on Linux, WebView2 on Windows). The proposal keeps that decision and only parameterizes:

| Language | Value | Date | Month | Own currency | Other currency |
|---|---|---|---|---|---|
| pt-BR | `1.234,56` | `14/10/2026` | `outubro de 2026`, `out/26` | `R$ 1.234,56` | `USD 1.234,56` |
| pt-PT (variant) | `1234,56` (thin space as thousands separator from 5 digits) | `14/10/2026` | `outubro de 2026` | `1234,56 €` | `USD 1234,56` |
| en | `1,234.56` | `Oct 14, 2026` (short `10/14/2026`) | `October 2026`, `Oct 26` | `$1,234.56` | `USD 1,234.56` |
| es | `1.234,56` | `14/10/2026` | `octubre de 2026`, `oct/26` | `$1.234,56` (ISO code when the currency is not the family's) | `USD 1.234,56` |

- A small table per language (separators, symbol position, date patterns, month names coming from the catalog) replaces the fixed constants. Money stays an integer in cents and arithmetic stays in integers and text.
- The minus sign U+2212 stays in all languages.
- `parseMoney` accepts the language's format and keeps accepting the pasted decimal point from an American source when there is no ambiguity. In English, `1,234` becomes one thousand two hundred thirty-four (not 1.234 reais): the ambiguity rule changes per language and gets its own tests.
- The currency symbol follows the language, not the country: `R$` in pt-BR, `$`/`US$` in English depending on the family's currency, `€` in pt-PT. When the currency is not the language's "home" one, it shows the ISO code, as today.
- In the database, nothing changes: dates stay ISO and values stay in cents.
- The core gets the same table for the texts it assembles itself (insights, notifications, explanations), today with `money(total, cur)` in pt-BR.

## Core texts

### Errors

Each error has a stable `code` and the message comes from the core catalog
(`locales/<language>/nucleo-<area>.ftl`, with `i18n::t` and `i18n::t_args`), in the installation's language.
The fixed errors (`NoVault`, `Locked`, `WrongPassword` etc.) have the key equal to the `code`
(`erro-no_vault`). Numbers in the arguments go as text, because Fluent would group the thousands.

### Insights, alerts, notifications and explanations

- Texts generated on the spot (insights, report explanations, alert warnings) use the catalog.
- What is stored in the database to be shown later (alerts) stores the key and the arguments
  (`title_key`, V128), and the text is assembled when displayed. Alerts older than V128 keep the
  stored text.
- System notifications go out in the current language at send time.

### What does not change

- Reading of statements, brokerage notes, payslips, purchase e-mails and NFC-e QR codes: the file's language is that of the bank or the issuing body, not the user's. The importers stay as they are.
- Internal names, code comments and logs stay in pt-BR. Commit messages are in English
  ([development.md](development.md#commits)).

## AI agent and MCP

| Part | How it works |
|---|---|
| Agent instructions (`ai/prompt.rs`) | In English, with the line "Always answer in {user's language}" and the language's number and date conventions. A single version for all languages; the language rule is repeated at the end of the instructions, and a prompt test checks the line. |
| MCP tool descriptions | In English: external clients and contributors read English, and the model answers the user in their language anyway. |
| Tool and field names | In English (`list_accounts`) |
| Tool responses | Neutral data; explanatory texts in the installation's language |
| Agent proposals (diff to approve) | In the user's language (they come from the core) |
| Manager memory, conversations | What was written; nothing is translated |

## Seeded data (ready-made categories and rules)

The initial categories separate what exists from the name (V127):

- **Which** categories exist is still decided by the country (Brazil has "Vale-refeição", the US has "401(k)").
- **The name** comes from the language: each seed gets a Fluent key instead of the fixed name.
- Each seeded category stores the origin key (`seed_key` column, a small migration). That way the app knows which ones still have the default name and can offer the translation when the language changes. A category renamed by the person is never touched.
- The ready-made categorization rules are still decided by the country, because they match the bank statement's text, which is in the bank's language.

## Screens for one country only

Screens that only appear in Brazil (IR with DARF, B3, FIPE, Fundamentus, portability) also go through the catalog, but their translation to English and Spanish has lower priority, because almost everyone who sees them reads Portuguese. A screen not yet translated appears entirely in pt-BR (the "no mixed text" rule applied per screen, with a discreet notice "This screen has not been translated yet"). Likewise, the US-specific screens are born in English and those of Portugal in pt-PT, and their other translations come later.

## Translation quality

- **AI draft, human review.** The first English and Spanish version can be made by AI from pt-BR, following a tone guide per language (`locales/GUIDE.md`: plain language, "você"/"you" address, in Spanish "tú", neutral Latin American; in pt-PT the "você" becomes a formal third person as it already is in the donations). Each file says at the top whether it was reviewed by a native speaker.
- **Pseudo-language to test layout.** An `xx` language only in developer mode, which lengthens the texts by ~35% and swaps letters for accented variants (`[Ćöñţåš ~~~]`). It serves to find forgotten fixed text and buttons that overflow when the text grows (German and Spanish tend to be 20–35% longer than Portuguese).
- **Automatic tests in CI and in `release.sh`:**
  - every pt-BR key exists in en and es (pt-PT can be partial), with the same arguments (`{ $nome }`);
  - no leftover key that the code does not use;
  - a lint rule (`eslint-plugin-i18next`, `no-literal-string`) that flags fixed text in JSX; it starts as a warning and becomes an error in each folder as soon as it is migrated;
  - the current tests (Vitest, Playwright, core tests) keep running in pt-BR, so none of them breaks because of the switch.

## Translation contributions (open source)

- A simple start: translations come in by pull request on GitHub, in the `.ftl` files.
- With contributors in other languages, a translation platform that understands Fluent can open the pull requests on its own.
- A new language enters the picker only when the app shell, the errors and the main screens (Overview, Accounts, Transactions, Budget) are 100% translated.

## Platform

- macOS: declare `CFBundleLocalizations` (pt-BR, en, es, pt-PT) in Tauri's `Info.plist`, so the system dialogs (open file, Touch ID, Keychain) come out in the right language. The native menu ("Support the project…") starts using the core's catalog.
- Linux and Windows: the app's texts already resolve everything; Windows notifications (toast) use the same catalog.
- Right-to-left writing (Arabic, Hebrew): out of scope. It is not worth restructuring the CSS now.

## Repository documentation

All documentation is in English, which is the reference version. A document may have an optional
copy in another language next to it, named `<name>.<language>.md`: `README.pt-BR.md`,
`CONTRIBUTING.pt-BR.md`, `SECURITY.pt-BR.md`, `CODE_OF_CONDUCT.pt-BR.md` and `docs/terms.pt-BR.md`
(the Portuguese terms are the legal source text). The copies are not kept in sync automatically:
when they differ, the English file is the one that counts, except for the terms.

## Limitations

- pt-PT is partial (about 15% of the keys); the rest appears in pt-BR.
- Of the health score methodology (`methodology/`), the `nucleo-metodologia.ftl` catalog translates
  what applies outside Brazil: foreign stocks, ETFs listed abroad and crypto assets (metrics with
  the formula, profiles, segments, dimensions, red flags and event types), plus the bands. What
  exists only in Brazil (B3 stocks, FIIs, CVM funds and local fixed income) stays in Portuguese
  in any language.
- Left out of the catalog, because they go only to the model or to whoever calls through the MCP:
  the actions refused outside the interface ("only through the app's interface"), the validations of
  the procedure memory, the texts that go to the model along with the conversation and the
  `treasury mcp` messages in the terminal.
- Right-to-left writing (Arabic, Hebrew) is not supported.
