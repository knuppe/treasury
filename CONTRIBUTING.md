# Contributing

[Português](CONTRIBUTING.pt-BR.md)

Thanks for your interest in Treasury. Bug reports, ideas, translations and code are welcome.
Issues and pull requests can be written in English, Portuguese or Spanish.

## Who builds and publishes Treasury

Treasury versions are built and published **exclusively by Knuppe**, the project owner. No
version is published without the owner's signature, made with a hardware key only the owner
holds, and an installed Treasury refuses any update without that signature.

The reason is security. People who use Treasury put their family's financial life in it and trust
that the software is free of malicious code. That trust is placed in an identified person, and
Knuppe commits to doing everything possible so that every version comes out clean: built from the
reviewed code in this repository, following the project's rules, and checked before it is
signed. How this works and how anyone can check it: [docs/trust.md](docs/trust.md) (in
Portuguese).

In practice, for contributors:

- an accepted contribution goes into the code, but only reaches users in the next version built
  and signed by Knuppe;
- a build made by someone else, including from a fork, is not a Treasury version: it does not
  receive updates and must not be distributed under that name;
- changes to the files that decide what the app accepts as an update and how a version is made
  (listed in [.github/CODEOWNERS](.github/CODEOWNERS)) only go in with the owner's review.

## Contributions and the project's vision

Treasury follows Knuppe's vision for the software ([docs/vision.md](docs/vision.md), in
Portuguese): what goes in, what stays out and how things work. A contribution may be declined and
closed without discussion when it does not fit that vision, even if it is correct and well
written. This is not a judgment of the person who contributed: it is what keeps the project
coherent and maintainable by one person. To avoid wasted work, open an issue before starting
anything large and wait for an answer.

## First of all: never post financial data

Treasury holds a family's financial life. An issue or pull request is **public and permanent**,
even if it is deleted later.

- **Do not attach** bank or card statements, payslips, brokerage notes, tax documents,
  spreadsheets or screenshots showing amounts, names, ID numbers, account or card numbers.
- To show a problem, use the **demo vault**: when creating a vault, choose "Demo vault" (the
  shortcut is also under Vaults). It holds only made-up data and opens without a password.
- If the bug only happens with one of your files (a bank statement, for example), describe its
  **format** (bank, file type, what differs from what was expected), never its content. If needed,
  make up a file with the same format and fake values.
- Security issues do **not** go in a public issue: follow [SECURITY.md](SECURITY.md).

## Reporting a bug or suggesting an idea

Open an [issue](../../issues/new/choose) and pick a template: bug, idea or translation. Include the
Treasury version (Settings › About and support), your system (macOS, Linux or Windows) and the
steps to reproduce it in the demo vault.

## Setting up

- Stable Rust (1.89 or newer) and Node.js 22.
- macOS: Xcode Command Line Tools. Linux: the WebKitGTK libraries listed in
  [.github/workflows/ci.yml](.github/workflows/ci.yml).
- Turn on the hook that rejects documents and personal data in commits:

  ```bash
  git config core.hooksPath .githooks
  ```

The full guide (code layout, running the UI against the real core, end-to-end tests and
conventions) is in [docs/development.md](docs/development.md).

## Before opening a pull request

1. For large changes (a new screen, a database migration, a new dependency), open an issue first
   and wait for an answer: what does not fit the project's vision is declined.
2. Run what CI runs:

   ```bash
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cd apps/desktop && npm run lint && npm run typecheck && npm test && npm run format:check
   ```

3. Tests and fixtures use **made-up data**: copy the format of a real document, never its content.
4. New text in the UI or the core becomes a key in the `locales/` catalogs: write it in pt-BR and
   add en and es (a test checks that all three have the same keys). The rules are in
   [locales/GUIDE.md](locales/GUIDE.md).
5. Database migrations follow the rules in [docs/development.md](docs/development.md): forward
   only, a number higher than the last released one, never edit a released one.
6. Commit messages in the imperative, with the module prefix (`core:`, `ui:`, `mcp:`, `docs:`,
   `build:`).

## Translations

Texts live in `locales/<language>/*.ftl` (Fluent). The English and Spanish versions started as AI
drafts, and a review by a native speaker is one of the most useful contributions. Follow the tone
for each language described in [locales/GUIDE.md](locales/GUIDE.md) and keep the arguments
(`{ $name }`) identical to pt-BR. For a new language, open an issue with the translation template
first.

## License and DCO

Treasury is distributed under the [AGPL-3.0-only](LICENSE). By contributing, you agree that your
contribution is distributed under the same license.

There is no CLA. We ask for the [Developer Certificate of Origin](https://developercertificate.org/)
(DCO): each commit carries a `Signed-off-by` line with your name and email, stating that you have
the right to submit that code. Git adds the line with `-s`:

```bash
git commit -s -m "ui: fix the statement total in the dark theme"
```

Commit messages are in English: a first line with a short summary in the imperative and the
module prefix (`core:`, `ui:`, `mcp:`, `docs:`, `build:`), up to about 72 characters, and, if
needed, a blank line and at most two short paragraphs on what changed and why
([docs/development.md](docs/development.md#commits)).

AI-assisted contributions are welcome; whoever signs off the DCO is responsible for the submitted
code. Do not paste code, text, icons or fonts from another project without checking that its
license is compatible with the AGPL-3.0 (the accepted list is in `deny.toml`). A new dependency
is the exception: the policy is in [docs/development.md](docs/development.md#dependencies), CI
blocks any name that is not in `scripts/deps-approved.txt`, and it only comes in with the owner's
approval and with its license in `THIRD_PARTY_LICENSES.md`
(`python3 -I scripts/third-party-licenses.py`).

## What the project does not do

- **No opinions on assets.** Issues, discussions and examples do not say whether a stock, fund or
  crypto asset is good, cheap or expensive, nor suggest buying or selling. The app calculates with
  the user's own data and criteria; questions about what to buy belong elsewhere.
- **No film, TV or brand names** in themes, texts or easter eggs.
- **No comparisons with competing products by name.**

## What to expect

Treasury is maintained by one person, Knuppe, who decides what goes in. Reviews may take a few
days, and small, focused pull requests get reviewed faster. All project spaces follow the [Code of Conduct](CODE_OF_CONDUCT.md).
