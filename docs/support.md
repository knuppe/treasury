# Supporting Treasury

Treasury is free and open source, made by one person. This document describes how the
app asks for support: a discreet, rare reminder on the Dashboard and an always-available page. None
of this uses the network, telemetry or analytics; the links only open in the browser when the user clicks.

- **Core:** `support` (`crates/treasury-core/src/support/`): the channels in `support.json`
  (embedded in the build) and the reminder rules (`decide`,
  a pure function). Commands `support.info`, `support.event`, `support.prompt`, `support.seen`,
  `support.respond` and `support.set_opted_out`; only `info` and `prompt` can be read by the agent
  or by MCP clients, and the agent only talks about support if asked.
- **Interface:** `features/support/` (page, card and texts).

## Where it appears

1. **Card at the end of the Dashboard**, in the style of the insights. Never a modal, never on unlock,
   never in the middle of an import.
2. **"Support Treasury" page** in Settings › About and support, and in the macOS menu
   (Treasury › Support the project…). Always available.
3. Never in notifications, email, the AI agent (only if asked), the demo vault
   or "View as" mode.

## Reminder rules

- **First time:** at least 30 days since installation **and** 10 distinct days of use.
- **Only after a value event:** reviewing an already closed month in the Budget, finishing an
  import (statement, brokerage notes, B3) or viewing the annual IR report. Never by a
  timer or on open. The event leaves the card waiting for the next visit to the Dashboard
  for up to 3 days, and it counts as shown when it appears.
- **Between appearances:** at least 30 days **and** 8 days of use. Three "Not now" in a row: 90
  days. At most 6 per year.
- **Silence:** a negative balance alert in the last 30 days, an overdue unpaid card statement,
  a late debt installment, or the last month with income below 60% of the average of the three
  before. The financial data only silence; they never increase the requests.
- **Buttons:** "How to support" (opens the page), "I already support" (silences for 12 months, without verification;
  later comes back as a thank-you), "Not now" (counts toward fatigue) and "Don't show again"
  (turns it off; turn it back on in Settings › About and support). "I already support" and "Don't show again" are always visible.
- **Per-installation counter**, in `support-state.json` in the app folder (outside the vault):
  `installed_at`, `active_days`, `last_active_day`, `active_days_at_last`, `last_shown_at`,
  `dismiss_streak`, `shown_this_year`, `shown_year`, `snooze_until`, `opted_out`, `supporter` and the
  pending card. No financial data. The day of use is marked by the app's cycle with the vault
  open.

## Texts

Two tones by profile level (Beginner and Intermediate; Advanced), in the app's language
([i18n-design.md](i18n-design.md)), in the catalogs `locales/<language>/apoio.ftl`. No suggested amounts, no guilt, goals, countdown
or comparisons. Signature: Knuppe.

## The page

1. **Why support:** the app is free and made by one person; the money goes to that person (AI,
   research, living); those who cannot owe nothing; it is support, not a contract.
2. **With money:** only GitHub Sponsors, no fee for personal accounts (up to 6% for
   organizations; checked on 2026-10-19). Pix and Liberapay were removed on 2026-11-16, at the
   maintainer's request: a single channel, in a place where the recipient's key and data are not kept in
   the repository or in the app.
3. **Without money:** report a bug and suggest a bank for the import (the agent helps
   write it, without personal data), translate or star the repository, recommend the app.
4. **Where it goes:** one honest sentence and, if there is one, the annual summary with the date
   (`annual_summary` in `support.json`).
5. **Commitments** of the app (no ads, data sales, tracking, affiliates; what exists
   does not become paid) and of the maker (security, security flaws first, updates, advance
   notice if it stops).

## Channels

In a single place, `crates/treasury-core/src/support/support.json`:

```json
{
  "github_sponsors_url": "https://github.com/sponsors/knuppe",
  "repository_url": "https://github.com/knuppe/treasury",
  "signature": "Knuppe",
  "annual_summary": null
}
```

While a value is `<<PLACEHOLDER…>>`, the channel shows as "coming soon" (never a broken
link). Links must start with `https://`. The annual summary is `{"date": "AAAA-MM-DD", "text": "…"}`.
