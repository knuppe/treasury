# Status and next steps

What Treasury already has, on which platforms, and what is still missing. The limitations of each
feature are at the end of its document; only the larger fronts are here.

## Status

- All the areas described in [vision.md](vision.md) exist: cash flow, cards, debts, assets,
  consumption, portfolio, asset analysis and Market, IR (income tax) and taxes paid, planning,
  financial health, AI agent and MCP server, multiple vaults, demo vault and four languages
  (pt-BR, English, Spanish and partial pt-PT).
- **macOS (Apple Silicon)** is the platform used every day.
- **Linux** (AppImage, `.deb`, `.rpm`): installation and first launch checked on Ubuntu
  22.04/24.04, Debian 12/13 and Fedora 42. Automatic update not yet tested.
- **Windows** (NSIS installer and `.msi`): built by CI, preview; not yet opened on a real
  installation and without code signing.
- Public releases since 1.0.0, with signed automatic update for all packages
  ([auto-update.md](auto-update.md), [trust.md](trust.md)).

## Open items

- Vulnerabilities in dependencies: whatever has a fix is fixed. Two exceptions remain in
  `deny.toml`, each with its reason; remove them when possible:
  - RUSTSEC-2026-0502 (SQLCipher): wait for a `refinery` that accepts `rusqlite` 0.42, to move up
    to `sqlcipher-src` 420.
  - RUSTSEC-2026-0543 (`chrono` unmaintained, no known flaw): `rusqlite`, `rmcp`, `lopdf`,
    `imap` and `schemars` depend on it. Switch to `jiff` or `time` when those dependencies
    switch.
- Test automatic update on Linux and Windows, and open the Windows packages on a real machine
  ([distribution.md](distribution.md)).

## Next fronts

| Front | What is missing | Document |
|---|---|---|
| Distribution on macOS | Developer ID and notarization (removes the Gatekeeper block and the Keychain prompt on every version) | [local-security.md](local-security.md), item 16 |
| Local security | Vault key wrapped by the Secure Enclave (item 5); lock on sleep on Linux and Windows | [local-security.md](local-security.md) |
| Linux and Windows | Real test of automatic update (already set up for all packages) | [auto-update.md](auto-update.md) |
| Languages | Complete pt-PT | [i18n-design.md](i18n-design.md) |
| Other countries | Local indexers (Euribor, SOFR), local exchanges as a reference, public retirement beyond Brazil, the US and Portugal | [international.md](international.md) |
| Market | Backtest of the methods and US universe | [design/explorador-de-mercado.md](design/market-explorer.md) |
| Enrichment | AI enrichment of asset data | [agent-enrichment.md](agent-enrichment.md) |
| Cards and installments | Split the card statement as a contract in Debts; convert detected installments | [design/cartoes.md](design/cards.md), [design/parcelas-detectadas.md](design/detected-installments.md) |
| Email | OAuth and other providers | [email-ingest.md](email-ingest.md) |

## Out of scope

Mobile app, hosted web version, cloud sync, brokerage orders and automatic login to banks
([vision.md](vision.md)).
