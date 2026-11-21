# Visual direction

Treasury presents itself as a **measuring instrument**: a near-black background, small uppercase labels with wide spacing, numbered sections and figures, cards with a window header and
key/value spec pairs. The motto is "Medido, nunca inventado" (Measured, never invented). This document describes
the principles, the themes, the signature components and the visualizations; the live values live in
`apps/desktop/src/design/tokens.css`.

## Principles

1. **Instrument panel, not a spreadsheet.** Large, tabular, aligned numbers; small uppercase labels above each measure.
2. **Every measure has an origin.** Each number can be "inspected": where it came from, from which date, from which source. It is "Medido, nunca inventado" applied to finance.
3. **Dark by default, other themes optional.** Classic: Dark, Light and Mono; special: Midnight, Paper and Glass; fun: Scanner, Orbit, Neon, Lo-fi and Phosphor.
4. **Numbering and captions as a language.** Screens and panels have an index (`01 · PATRIMÔNIO`, `FIG. 03 · CARTEIRA × IPCA`).
5. **Density with breathing room.** A lot of information, but a rigid grid, generous spacing and no decoration without a function.
6. **Restrained motion.** Numbers that "arrive" with a short transition (no animated counting, which would invent intermediate values), a `● AO VIVO` indicator when data is updating, an animated mark, cards that arrive in sequence and a faded theme switch; only opacity and transform, and nothing moves with the system's reduced-motion preference (`src/design/motion.css`).
7. **Accessible in every theme.** WCAG AA minimum contrast for text in all themes; gain and loss never depend on color alone (`+`/`−` sign, ▲/▼ icon); full keyboard navigation.
8. **Brazilian format.** `R$ 1.234,56`, `14/10/2026`, `12,5%`; values in another currency show the code (`US$ 1,234.56` becomes `USD 1.234,56` in the pt-BR standard) and the equivalent in reais beside it.

## Tokens

### Dark theme colors

| Token | Value | Use |
|---|---|---|
| `--bg` | `#070b0f` | window background |
| `--surface` | `#0d131a` | cards |
| `--surface-2` | `#121a23` | raised cards, hover |
| `--line` | `#1e2a36` | 1px borders, grid |
| `--text` | `#e6edf3` | main text |
| `--text-dim` | `#8b98a5` | labels and captions |
| `--accent` | `#4cc9f0` | focus, links, main series |
| `--positive` | `#3ddc97` | gain, income |
| `--negative` | `#ff5c5c` | loss, expense |
| `--warning` | `#ffb020` | alerts, inflation |

Mono swaps the whole palette for shades of gray (a single hue), keeping gain/loss distinguishable by icon and sign, not only by color.

### Themes and materials

The live values live in `apps/desktop/src/design/tokens.css`; the table above summarizes the color roles. Each theme has solid colors (text, backgrounds, accent, gain/loss) and a **material**: how the surfaces are painted (`--surface-fill`, `--pop-fill`, `--chrome-fill`, border, glow, shadow, background art).

| Theme | Character | Accent |
|---|---|---|
| Dark (default) | bluish graphite `#070b0f`, cards with a top glow and soft shadow | cyan `#56c7ef` |
| Light | neutral light gray, white cards with a light shadow | blue `#0b69c7` |
| Midnight | deep navy blue | champagne `#e6c27a` |
| Paper | warm light, like paper | blue ink `#2d4f86` |
| Glass | frosted glass (`backdrop-filter: blur() saturate()`) over a still mesh of color (animated, it forced the blurs to be redone every frame) | lavender blue `#a3c4ff` |
| Mono (classic) | shades of gray, a single hue; charts with up to 4 series by luminosity | white |
| Scanner (fun) | glossy car black `#050506` with a diagonal reflection; the scanner (eight lamps with dark tips, sweep lines and a CRT TV grid, light with a trail) sweeps the edge of the top bar while an agent task is running and disappears like a TV turning off when there is none, the dot of the indicators becomes a mini scanner and the agent's voice box pulses while it answers | scanner red `#ff2b1c` |
| Orbit (fun) | blue-black space with stars and the black hole at the top right (shadow, photon ring, a golden disk that turns slowly and lens arcs); opaque cards | disk gold `#f0b25a` |
| Neon (fun) | violet night with haze, orange fire on the horizon, pink and cyan neon, diagonal rain and the top bar edge in neon; opaque cards | neon pink `#ff4fb0` |
| Lo-fi (fun, light) | a calm afternoon in Japan: washi paper `#f1ebe0`, sumi ink with a touch of violet, a low pale sun at the top, lilac hills and maple and ginkgo leaves drifting in the wind through the gaps between the cards (opaque); matcha gain, beni loss, kincha alert | ai-iro indigo `#3a5f8f` |
| Phosphor (fun, dark) | old terminal: pure black, cards `#050a06` with border `#0f2a14`, text `#b8ffc8`, numbers, tables and titles in mono (titles in uppercase) and code rain falling slowly in the top strip; gain in its own green, loss in the red pill `#ff3b3b`, alert `#e5ff00` | phosphor green `#00ff41` |

- **Contrast**: `tokens.test.ts` requires 4.5:1 for all text over all backgrounds of all themes and 3:1 for control borders. In Glass it tests the worst case: the glass composited over the lightest color at each point of the mesh (without blur) and the popover glass over pure white. Without `backdrop-filter` or with reduced transparency in the system, Glass goes back to solid colors.
- **Charts**: each theme has its own categorical palette and sequential ramp, validated with the dataviz skill over the theme's surfaces (`src/charts/palette.ts`).
- **Mark**: the frame sign is drawn when the vault opens and on mouse hover, a reading beam crosses the frame from time to time and the final dot breathes like a live indicator.
- **Animated art of Orbit and Neon**: `design/cinema.css` (body::after; the still art lives in --canvas-art and appears in the thumbnails). With the art in motion, the cards of these themes are opaque and without `backdrop-filter` (the blur would have to be redone every frame).
- **Scanner parts**: `components/ScannerParts.tsx` and `design/scanner.css`. They are always in the DOM and only appear when the closest theme is Scanner (`--scanner-display`/`--scanner-hide` in tokens.css), including in the picker thumbnail; only transform and opacity move and, with reduced motion, the light stays still in the center.
- **Activity**: `state/activity.ts` is the single source of what is running (`start`/`end`, `useActivityTask`, `useSettleFlash`). It receives the agent jobs and the core tasks (`ActivityBridge` in app/BackgroundTasks.tsx) and the interface's slow commands (`ACTIVITY_COMMANDS` in api/client.ts: import, receipt, email, scenario, quotes; those that finish in under 250ms light nothing). The Scanner's scanner, the pieces of Orbit, Neon and Lo-fi and Phosphor react only to it; a new theme turns on its version by reading `useBusy()`.
- **Phosphor parts**: `design/phosphor.css`, `components/PhosphorRain.tsx`, `PhosphorActivity.tsx` and `Decode.tsx`, hidden outside the theme by `--phosphor-display`/`--phosphor-hide`.
  - Rain: a fixed canvas behind everything, up to 30 frames per second, paused when the tab is hidden and destroyed when the theme changes.
  - At rest, the rain stays in the ~180px strip at the top; with activity, it is three times faster and stronger (0.15 → 0.35) and falls to the bottom of the screen, returning in ~600ms.
  - Chip `▮ PROCESSANDO_` (or `▮ N TAREFAS_`, `▮ FALHA_` for 3s), with `aria-live`; the tooltip lists the tasks.
  - Money values arrive "decoding" in 400–800ms (at most 48 at the same time).
  - In the agent, while it thinks, the line `> agente conectado... analisando dados_`; the new text of the answer enters in a lighter green (CSS Custom Highlight API).
  - When an answer or a scenario recalculation finishes, the border flashes green or red.
  - With reduced motion: no rain and no decode, only the still chip.
  - Cost measured in WebKit (Market, Portfolio and Assets, 2026-11-01): the rain costs ~14ms of script per second at rest and ~24ms with activity, without changing the scroll pace of the screens.
  - No easter eggs with phrases or objects from films. A theme's name, description and comment do not cite a film, series, character, story location or third-party brand: the theme describes what is seen (trademark risk; the old ids `kitt`, `interstellar`, `bladerunner` and `matrix` stay because they are the value saved in the settings).
- **Picker**: Settings › Appearance and use shows a real thumbnail of each theme (the `data-theme` attribute applies the theme only inside it); the command bar has one command per theme.

### Typography

- **Numbers, labels and tables**: JetBrains Mono, with `font-variant-numeric: tabular-nums`.
- **Titles and running text**: Inter.
- Both fonts are embedded in the app (`src/design/fonts.css`, OFL-1.1 license).
- Labels: 11px, uppercase, `letter-spacing: 0.12em`, color `--text-dim`.
- Main measures: 32–48px mono, weight 500.

### Signature components

- **Instrument card**: header with `● ● ●`, index (`02`), title in uppercase; body with the measure and a small chart; footer with key/value pairs (`FONTE  BCB/SGS 433`, `ATUALIZADO  14/10/2026`).
- **Inflation gauge**: a nominal × real double bar per asset.
- **Figure**: chart with a numbered caption (`FIG. 04 · PATRIMÔNIO LÍQUIDO, 24 MESES`).
- **Technical table**: thin lines, right-aligned mono columns, selected row with an `--accent` border. Where one can act on several rows (asset and portfolio records, Transactions, statements), Shift+↑/↓ (and Shift+Home/End/PageUp/PageDown) extend the selection from the anchor, Shift+click takes the range, ⌘/Ctrl+click toggles the row, ⌘A/Ctrl+A selects all and Esc goes back to one; selected rows get a background and an `--accent` bar on the left, the border stays on the cursor row and a status line shows the count (and sums).
- **Command bar** (⌘K / Ctrl+K): navigation and questions to the AI agent in the same field.
- **Approval queue**: AI proposals shown in plain language (sentences about what changes and, per row, fields in plain words with names instead of ids, money, dates and types formatted) and, in "View technical details", the exact diff (before/after, line by line, with ids and raw values); approve all, approve selection or reject; counter in the sidebar. The change history uses the same reading.
- **Context menu**: right click, Shift+F10 or the Menu key on cards and table rows open a custom menu (the webview's native one is turned off) with the actions that already exist on the screen, shortcuts beside them and destructive actions in red; arrows, letter, Enter and Esc, with focus returning to whoever opened it. On a row of a multi-row selection, it opens the selection's menu ("3 trades selected": copy, delete…). In tables, the shortcuts shown in the menu (⌘C copies the rows as tab-separated text, ⌘⌫ deletes with confirmation) work with focus on the row, without opening the menu.
- **Origin badge**: a discreet icon beside each number; on hover it shows source, date and whether the value is official or **estimated** (estimated ones in italics with `≈`).
- **Privacy mode**: a shortcut that swaps all values for `•••••`, keeping charts without a scale; useful for sharing the screen. By default it turns on by itself each time the vault is unlocked (Settings › Appearance and use turns it off).
- **Collapsible sidebar**: the button at the top of the menu (or ⌘\\ / Ctrl+\\) leaves only the screen numbers, with the name in the tooltip, to give more area to the screen; the choice is saved.
- **Theme in the top bar**: the theme icon opens a menu with all the themes (classic, special and fun).
- **Alert inbox**: alerts from the agent and the rules, with state (new, seen, resolved).

## Tabs and chips

Tabs must read as tabs, the same on every screen and in every theme.

- **Section tabs** (`TabsList`/`TabsTrigger`, Radix Tabs): a track with a border and background
  (`.tab-list`). The active tab has the accent's background and text and a 2 px bar underneath
  (`.tab-trigger[data-state=active]`). The tabs stay uppercase and spaced and drop to
  a second line when space runs out.
- **Segmented controls** (`Segmented`, radio group): the same track background and the same
  active-tab mark, in compact size, for choices inside a panel (period,
  order, deflator).
- **Chips** (`Chip`/`ChipGroup`): one-touch shortcuts and filters (Rankings presets,
  Consumption segments, class in the asset record). They are rounded, with a border and normal
  text; the checked one gets the accent's border, background and text.
- **Colors:** they come from tokens derived from each theme (`--tab-track`, `--tab-hover`, `--tab-text`,
  `--tab-active`, `--tab-border` in `tokens.css`), mixing the theme's text and accent.
  No theme needs to define its own colors. Checked in all 10 themes in WebKit.

## Visualizations

Charts are the center of the experience, especially for expenses. All use **ECharts** with the theme's color tokens, are interactive (hover shows value and origin, clicking opens the transactions) and switch between **nominal** and **inflation-adjusted**.

### Expenses
| Name | Visualization | Question it answers |
|---|---|---|
| Flow | **Sankey** income → accounts → categories → subcategories | Where did the month's money go? |
| Composition | **Treemap / sunburst** by category and subcategory | What weighs most in the budget? |
| Rhythm | **Calendar heatmap** (one square per day, intensity = spending) | On which days and weeks do we spend most? |
| Trend | **Stacked bars** by category, 12–24 months, with a moving average | Is spending going up? On what? |
| Budget | **Bullet chart** planned × actual by category | Where did we overshoot? |
| Comparison | **Slope / diverging bars** period A × period B | What changed from one month or year to another? |
| Inflation | **Lines** nominal spending × IPCA-adjusted | Did we spend more or did it just get more expensive? |
| Holders | **Grouped bars** by family member | Who spends on what? |
| Recurring | **Timeline** of subscriptions and fixed bills | How much is committed every month? |
| Cards | **Area** of future card statements (installments) | How much is already committed in the coming months? |

### Net worth, debts and portfolio
- Net worth as a stacked area (assets × liabilities).
- Outstanding balance and interest × amortization per installment.
- Portfolio allocation (donut or treemap) and deviation from the target.
- Cumulative return × CDI × IPCA × Ibovespa × S&P 500.
- Nominal × real double bar per asset.
- Price chart with purchases and sales marked and an average price line (Lightweight Charts).

## Screens

The single registry of screens lives in `apps/desktop/src/app/screens.ts` (index, route, group, minimum
level and required country feature). Documents cite screens by name, without the number, because
the index changes when a screen enters.

| Screen | What it has |
|---|---|
| Dashboard | Net worth, the month's cash flow, savings rate, portfolio × inflation, due dates, To solve, next step |
| Family | Members, each one's income (salary, payslips, benefits, informal income), each member's accounts and cards, income forecast |
| Accounts | Balances by account, reconciliation, statement status, FGTS |
| Transactions | Table with filters, split, import, rules, card statements |
| Budget | Planned × actual, goals, projects, emergency fund, retirement, subscriptions, installments, scenarios |
| Consumption | Purchases item by item and price per product ([consumption.md](consumption.md)) |
| Debts | Contracts, schedule, simulators |
| Portfolio (Advanced) | Positions, trades, dividends, nominal and real returns, risk, sales, physical assets |
| Assets (Advanced) | Record and page of each asset, rankings, screener, comparator, dividend calendar, watchlist |
| Market (Advanced, Brazil only) | Explore the universe of B3 assets |
| Reports (Intermediate) | Reports, PDF and IR (monthly calculation, DARF, return) |
| Taxes (Advanced, countries with data) | Estimate of the taxes paid in the year ([taxes.md](taxes.md)) |
| Agent | Chat with the AI, approval queue, alerts and tool history |
| Settings | AI engines, data sources, security, backups, change history, themes, profile, vaults |
| Calculators | Quick math without saving anything ([calculators.md](calculators.md)) |
| Developer (developer mode only) | Agent feedback for whoever develops Treasury |

Below the minimum level, the screen leaves the main menu and goes to "More screens". Shortcuts: ⌘1…⌘9 and
⌘0 open the first ten screens of the menu (a screen the family's country does not have, like Market
outside Brazil, does not open through the shortcut); the Agent also opens in the side panel (⌘J); the command
bar (⌘K) reaches all of them.
