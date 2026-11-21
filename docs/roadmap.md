# Status and next steps

What Treasury already has, on which platforms, and what is still missing. The limitations of each
feature are at the end of its document; only the larger fronts are here.

## Status

- All the areas described in [vision.md](vision.md) exist: cash flow, cards, debts, assets,
  consumption, portfolio, asset analysis and Market, IR (income tax) and taxes paid, planning,
  financial health, AI agent and MCP server, multiple vaults, demo vault and four languages
  (pt-BR, English, Spanish and partial pt-PT).
- **macOS (Apple Silicon)** is the platform used every day.
- **Linux** (AppImage, `.deb`, `.rpm`) and **Windows** (NSIS installer and `.msi`) build from the
  same code, through Docker or in CI, but have not yet been tested end to end on a real
  installation.
- Signed automatic update is implemented for all packages ([auto-update.md](auto-update.md)), but
  only starts working once the keys are created (below) and has not yet been tested on Linux and
  Windows.

## Before the first public release

- Create the update keys (two YubiKeys, a new minisign, a recovery key) and test a real update on
  macOS, Linux and Windows ([auto-update.md](auto-update.md)). Without them, no release ships and
  no app accepts an update ([trust.md](trust.md)).
- Vulnerabilities in dependencies: whatever has a fix is fixed. Two exceptions remain in
  `deny.toml`, each with its reason; remove them when possible:
  - RUSTSEC-2026-0502 (SQLCipher): wait for a `refinery` that accepts `rusqlite` 0.42, to move up
    to `sqlcipher-src` 420.
  - RUSTSEC-2026-0543 (`chrono` unmaintained, no known flaw): `rusqlite`, `rmcp`, `lopdf`,
    `imap` and `schemars` depend on it. Switch to `jiff` or `time` when those dependencies
    switch.
- Turn on the repository protections on GitHub ([distribution.md](distribution.md)).
- See CI green in the public repository, on the three systems, and open the Linux and Windows
  packages on a real machine ([distribution.md](distribution.md)).
- Fill in what is missing from support and the Terms (donation channels, contact and effective
  date; [support.md](support.md), [terms.md](terms.md)) and generate the full list of
  dependency licenses (Rust and npm) along with the package.

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
