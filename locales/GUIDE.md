# Language catalog guide

Treasury's texts live here, in Fluent (`.ftl`), one file per area with the same name in every
language (docs/i18n-design.md). The interface and the core read the same folder.

## Rules

- **pt-BR is the source.** Whoever writes code writes the text in pt-BR and the key; en and es
  come afterward. The core test `i18n::tests` requires every pt-BR key to exist in en and es, with
  the same arguments (`{ $nome }`), and en and es must not have extra keys. pt-PT can be partial:
  whatever is missing falls back entirely to pt-BR.
- **Keys** are in pt-BR, without accents, prefixed by the area: `contas-titulo`, `casca-menu-painel`.
- **After changing pt-BR**, run `npm run i18n:keys` in `apps/desktop` (it generates the key types;
  `tsc -b` reports a wrong key or a missing argument).
- **Long texts** (such as the full terms of use) do not become keys: they stay as Markdown per
  language in `conteudo/<language>/`. Short paragraphs, such as the glossary ones and the tips of
  the day, stay in the area's `.ftl` (`glossario.ftl`, `dicas.ftl`).
- **Glossary:** each term has a per-language search (`glossario-<id>-busca`, regular expressions
  separated by `|`, tested against the label in lowercase and without accents) that decides where
  the `?` appears. When translating a label, check that the language's search still matches it.
- **One key, one file:** the same key in two `.ftl` files of the same language breaks the catalog
  load; the area prefix avoids this, and a core test checks it.
- **Official terms stay in the original** in any language: DARF, IRRF, INSS, Pix, FGTS, 13º, IRS,
  FICA, NIF. A short explanation goes in the language's glossary.
- **User data is not translated** (account name, created or renamed category).

## Tone per language

- **pt-BR:** simple language, "você", short sentences. No banking jargon.
- **en:** plain English, "you", sentence case in titles.
- **es:** neutral Latin American Spanish, "tú", no regionalisms ("computadora", "celular").
- **pt-PT:** formal third person (as in the donations), vocabulary from Portugal ("ecrã",
  "palavra-passe", "guardar").

## Review

Each file says in the comment at the top whether a native speaker reviewed it. The first version of
en and es is an AI draft from the pt-BR.
