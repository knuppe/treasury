# Profile and experience levels

Three levels (Beginner, Intermediate, Advanced) change what shows up first and how the app
explains things, based on a short welcome questionnaire. They live in Settings › **Profile**, in the route
`#/boas-vindas` and in the `profile.*` commands (core in `crates/treasury-core/src/profile`,
interface in `apps/desktop/src/features/profile`).

The goal is for the app to serve both someone who has never tracked an expense and someone who follows a portfolio
and taxes: **simple by default for those who need it, complete one click away for everyone**. No
feature is removed at any level; the level only changes what shows up first and how the app explains it.

## Welcome

A new vault, empty and without a profile: the app opens the welcome before any screen, with two
clear exits:

- **"I want to tailor the app to me"**: 7 multiple-choice questions (one per screen, with a
  progress bar, a back button and "Skip this question").
- **"I know what I want: use the full interface"**: advanced mode right away, with no questions.

Vaults that already had data before the profile existed stay at **Advanced**, with no questionnaire
(`source = padrao`). The questionnaire can be redone in Settings › Profile.

### Questions and scoring

Each option is worth 0 (beginner) to 2 (advanced); the level comes from the average of the answered
questions (for the goals question, the average of the options checked): below 0.8, **Beginner**;
below 1.5, **Intermediate**; otherwise, **Advanced**. With no scored answer,
Intermediate. The raw answers are stored (`answers`).

| Question | Options (points) |
|---|---|
| How comfortable you feel managing money | not at all (0) · so-so (0.7) · quite (1.4) · very (2) |
| Whether you already track spending | never (0) · sometimes (0.5) · spreadsheet or app (1.3) · in detail (2) |
| Goal right now (several) | where it goes (0) · emergency fund (0.4) · debts (0.8) · a goal (1) · get organized (1.5) · invest (2) |
| Emergency fund | don't know what it is (0) · don't have one (0.5) · have a little (1) · have enough (2) |
| Main income | fixed · variable · government benefits · mixed · prefer not to say (no points: it only adjusts the tips) |
| Terms like CDI and compound interest | I get lost (0) · so-so (1) · I understand well (2) |
| How the app should talk | very simple (0) · balanced (1) · technical (2) — becomes the **tone** |

## Each member's profile

Each family member can take their own questionnaire ("By family member" box on the
Dashboard, or Settings › Profile with the member chosen in "View as"). Until they do, the family
profile applies. In a member's view, the interface uses their level and the agent talks to them in
their tone ([members.md](members.md)).

## What changes by level

- **Dashboard**
  - **Beginner:** What came in, what went out and what was left (in sentences), emergency fund goal with a progress bar, where the money went, tip of the day, shortcuts "Record an expense" and "Record income"
  - **Intermediate:** Month summary + cash flow, budget, categories, due dates and financial health
  - **Advanced:** Full dashboard
- **Menu**
  - **Beginner:** The screens with no minimum level (Dashboard, Family, Accounts, Transactions, Budget, Consumption, Debts, Agent, Settings, Calculators); the rest under "More screens"
  - **Intermediate:** + Reports; Portfolio, Assets, Market and Taxes under "More screens"
  - **Advanced:** Everything
- **Agent**
  - **Beginner:** Simple language, terms explained, encouraging tone
  - **Intermediate:** Clear, without unnecessary jargon
  - **Advanced:** Technical

- "See full dashboard" shows the whole dashboard at any level (remembered on the computer);
  "Back to the simple dashboard" undoes it.
- The tone (Settings › Profile › Language style) goes into the agent's instructions (`profile::prompt_line`),
  with the goal and the type of income (variable income and benefits change the tips).
- "What is this?" explanations (`content/glossary.ts`) and tips of the day (`content/tips.ts`) are
  content in a file, filtered by level, goal and income.
- **`?` icons** (`design/help.tsx`): at Beginner, every label that mentions a glossary
  term (cards, measures, key/value pairs, fields and table headers) gets a `?`
  that explains the term on hover or on focus with Tab, in the profile's tone (very simple or
  balanced/technical). At Intermediate, only technical terms (CDI, XIRR, come-cotas…); at
  Advanced, none. Settings › Profile › "Term explanations" switches to Always or Never.
  New term: an entry in `GLOSSARY` with `simples`, `equilibrado` (optional), `upTo` and the
  label fragments that identify it (`match`).

## Ideas not yet done

The Dashboard's "Your next step" and "Close the month" ([to-resolve.md](to-resolve.md)) already
cover the order between debts and the emergency fund and the month's report. These remain as ideas:

1. Micro-lessons of 1 to 2 minutes filtered by level, and tips driven by behavior (a category
   that grew, repeated small expenses).
2. An adapted budget (50/30/20 or 70/20/10 for lower or variable income).
3. A suggestion to move up a level after milestones (30 days of use, emergency fund created).
