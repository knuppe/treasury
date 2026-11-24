# Treasury

> Family finances and investments, local-first, encrypted, with an optional AI agent.
> *Measured, never invented: every number has a traceable source.*

[Português](README.pt-BR.md)

**[Download the latest version](https://github.com/knuppe/treasury/releases/latest)** · [Step-by-step
install guide](https://knuppe.github.io/treasury-site/installation/) · [How to install](#install)

Treasury is a desktop app for a family to keep all of its money in one place: bank accounts,
credit cards, budget, debts, property, investments and taxes. The data stays on your computer, in
an encrypted database, and an optional AI agent helps analyze and record things, always asking for
approval first.

![Reports > Flow: where a sample family's income goes, from salary through the budget to each expense category](docs/images/sankey-en.webp)

## What it does

- **Cash flow**: accounts, credit cards (statements, additional cards, limits, fees, cashback and
  points), transactions with categories, splits, rules and recurrences, budget, goals and spending
  groups, per family member.
- **Imports**: OFX/CSV statements, card statements and brokerage notes in PDF, B3 investor reports,
  payslips, receipts (Brazilian NFC-e, photos read by local OCR), e-mail ingestion and a watched
  inbox folder. Everything is previewed and deduplicated before it is saved.
- **Debts and property**: loans and mortgages (fixed rate, CDI/Selic, inflation-indexed),
  amortization simulations, payoff plans, real estate and vehicles with valuations.
- **Investments**: portfolio with average cost, realized and unrealized results, TWR and XIRR,
  **real** returns (deflated by the country's inflation, CPI or M2), risk, concentration, sales
  ledger, fundamentals, rankings, screener, a market explorer and a health score per asset.
- **Taxes**: Brazilian income tax on investments (monthly assessment, DARF, annual report) and an
  estimate of all the taxes the family paid in the year.
- **Planning**: balance forecast, "how much can I spend today", subscriptions, what-if scenarios,
  retirement (Monte Carlo, with the public pension rules of Brazil, the US and Portugal) and a
  "next step" guide for people who are just starting.
- **AI agent and MCP**: the app exposes all its features as an **MCP server** (`treasury mcp`).
  The built-in agent and external MCP clients use the same tools; writes become proposals that you
  approve, with undo and an audit log. Engines: APIs, local agent CLIs that use your own
  subscription, or local models through an endpoint.
- **Countries and languages**: complete for Brazil, adapted for the US, Portugal, Spain, the UK,
  Canada, France, Germany, Argentina and Chile, generic elsewhere. Interface in Brazilian
  Portuguese, English and Spanish (European Portuguese partially).

## Security and privacy

- One SQLite + SQLCipher (AES-256) file per vault, with a key derived from your master password by
  Argon2id. Touch ID, PIN, FIDO2 keys and 2-of-3 split recovery are optional.
- Managed, encrypted and verified backups, with rotation and recovery when the vault does not open.
- No telemetry. The internet is used only for market data, indices and the AI engine you choose,
  and each data source can be turned off.
- The MCP server always requires an API key you create, and external clients only read or propose.

- **Only the maintainer publishes versions.** Packages are built by GitHub Actions with public
  build provenance, then signed on the maintainer's computer with a hardware key (YubiKey, touch
  and PIN). Every installed Treasury refuses an update without that signature, so a hijacked
  GitHub account or CI cannot ship a version. Why and how to check: [docs/trust.md](docs/trust.md).
- Known vulnerabilities in dependencies (RustSec and npm) block every release.

Details: [docs/security.md](docs/security.md) and [docs/local-security.md](docs/local-security.md).
To report a vulnerability, see [SECURITY.md](SECURITY.md).

## Install

You don't need to know how to program. It takes about five minutes.

1. Open the [latest release](https://github.com/knuppe/treasury/releases/latest) page.
2. Scroll down to the **Assets** section at the bottom of the page. If you only see two "Source
   code" files, click **Assets** to show the full list.
3. Download **only one** file, the one for your computer:

| Your computer | File to download |
|---|---|
| Mac with an Apple chip (M1 or newer) | `Treasury_<version>_aarch64.dmg` |
| Ubuntu, Debian, Linux Mint, Pop!_OS | `Treasury_<version>_amd64.deb` |
| Fedora, openSUSE | `Treasury-<version>-1.x86_64.rpm` |
| Any other Linux | `Treasury_<version>_amd64.AppImage` |
| Windows 10 or 11 | `Treasury_<version>_x64-setup.exe` |

Not sure which Mac you have? Open the Apple menu › **About This Mac**: if "Chip" says Apple M1,
M2, M3 or newer, this version is for you. Macs with an Intel processor are not supported.

You can ignore every other file on the page. The `.sig` files, `latest.json`, `allowed_signers`
and `.app.tar.gz` are used by the automatic updates and by people who check signatures, the
`.msi` is for companies that install software on many computers, and `SHA256SUMS.txt` is for the
[optional check](#check-the-download-optional) below.

### macOS

The Mac version is not notarized by Apple yet, so macOS blocks the first launch. Once per
installation:

1. Open the downloaded `.dmg` and drag **Treasury** onto the **Applications** folder.
2. Open Treasury from Applications. macOS says it "cannot be verified"; click **Done** (not "Move
   to Trash").
3. Open System Settings › Privacy & Security, scroll to the message about Treasury and click
   **Open Anyway**, then confirm with your password or Touch ID.
4. Treasury opens. From then on it opens normally, and updates install by themselves.

### Linux

- **`.deb`**: double-click the file and click **Install** in the software center, or run
  `sudo apt install ./Treasury_*_amd64.deb` in the Downloads folder. Then open **Treasury** from the
  applications menu.
- **`.rpm`**: double-click the file and click **Install**, or run
  `sudo dnf install ./Treasury-*-1.x86_64.rpm` (on openSUSE, `sudo zypper install ...`).
- **AppImage**: right-click the file › **Properties** › **Permissions** and turn on **Allow
  executing file as program** (or run `chmod +x Treasury_*_amd64.AppImage`), then double-click it.
  It needs FUSE (`fuse3`) and the system EGL/GLES libraries (`libegl1`, `libgles2`), already
  present on most desktops. If it doesn't open, run
  `sudo apt install fuse3 libegl1 libgles2` and try again. Keep the file in a permanent folder,
  such as `~/Applications`.

### Windows

1. Double-click `Treasury_<version>_x64-setup.exe`.
2. The installer is not code-signed yet, so SmartScreen shows "Windows protected your PC": click
   **More info** and then **Run anyway**.
3. Follow the installer and open **Treasury** from the Start menu.

### Did it work?

The first time, Treasury asks for your language, shows the terms of use and then offers a **demo**
vault (a fictional family, to explore without typing anything) or a **real** vault protected by a
master password. The installed version is in Settings › Updates. Next steps:
[getting started](https://knuppe.github.io/treasury-site/getting-started/).

### Check the download (optional)

Each release has a `SHA256SUMS.txt` with the fingerprint of every file. Put it in the same folder
as the file you downloaded and run `shasum -a 256 -c --ignore-missing SHA256SUMS.txt` on macOS or
`sha256sum -c --ignore-missing SHA256SUMS.txt` on Linux; your file's line must end in `OK`. On
Windows, compare the output of `Get-FileHash .\Treasury_*_x64-setup.exe` in PowerShell with the
file's line. How to check the maintainer's signature too: [docs/distribution.md](docs/distribution.md).

### Platform status

| System | Package | Status |
|---|---|---|
| macOS (Apple Silicon) | `.dmg` | Used daily; automatic updates |
| Linux (x86_64) | AppImage, `.deb`, `.rpm` | Built by CI; automatic updates; install checked on Ubuntu 22.04/24.04, Debian 12/13 and Fedora 42 (update not yet tested) |
| Windows (x64) | NSIS installer, `.msi` | Built by CI; automatic updates; preview, no code signing yet |

Your data lives in the `.treasury` folder in your home folder, outside the app, so updating or
uninstalling Treasury doesn't delete it. Uninstalling and other details:
[installation guide](https://knuppe.github.io/treasury-site/installation/).

## Build from source

Requirements: Rust 1.92 or newer, Node.js 22 and, on Linux, the WebKitGTK libraries listed in
[.github/workflows/ci.yml](.github/workflows/ci.yml).

```bash
cargo test -p treasury-core
```

```bash
cd apps/desktop && npm ci && npm run tauri dev
```

The full guide (repository layout, running the UI against the real core, end-to-end tests,
release builds for macOS, Linux and Windows) is in [docs/development.md](docs/development.md).

## Documentation

The documents in [docs/](docs/README.md) are in Brazilian Portuguese: product vision, decisions,
architecture, each feature, data sources and validation records.

## Contributing

Bug reports, ideas, translations and code are welcome, in Portuguese, English or Spanish. Never
attach real statements or screenshots with values: use the demo vault.

Releases are built and signed exclusively by the project owner, Knuppe, because people trust
Treasury with their financial life. Contributions that do not fit the project's vision may be
declined without discussion; open an issue before starting anything large. See
[CONTRIBUTING.md](CONTRIBUTING.md) and the [Code of Conduct](CODE_OF_CONDUCT.md).

## Disclaimer

Treasury is a tool to organize and study your own data. Its analyses, scores and simulations are
not investment, tax or legal advice. See the terms of use shown on first launch
([docs/terms.md](docs/terms.md)).

Treasury is a personal open-source project. It is not affiliated with, endorsed by or connected to
any company, bank, government body or other product that uses the word "treasury" in its name.

## License

[GNU AGPL-3.0-only](LICENSE). Contributions are accepted under the same license with a DCO
sign-off (`git commit -s`). Third-party components and embedded data: [NOTICE](NOTICE) and
[DATA_LICENSES.md](DATA_LICENSES.md).
