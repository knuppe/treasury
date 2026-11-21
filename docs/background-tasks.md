# Background tasks

Downloads and calculations that run in the background (quotes, indices, fundamentals, email,
app update) show up as tasks at the top of the interface, with a progress bar when the
total is known and an option to cancel.

## How it works

1. **One place.** The **Background tasks** button in the top bar (activity icon)
   lists the collections together with the agent's jobs (conversations and routines). The counter, the
   Scanner theme's scanner and the animated piece of the other themes count both.
2. **At the top, while it runs.** While a collection is running, a chip appears next to the button
   with the short name of the task ("Fundamentals", "Quotes"), the percentage when there is a total and a
   thin progress bar. Clicking opens the list. The chip disappears when it finishes, because the
   result already shows on the screens.
3. **Honest progress.** The bar is determinate only when the total is known: quotes (one
   request per asset), months of the CVM fund unit price history and the update download. In
   fundamentals (CVM, SEC, B3, IF.data), the number of files depends on what changed since the last
   collection. So the bar stays indeterminate, with the current file and how many are done.
4. **Canceling is cooperative and keeps what arrived.** The cancel request takes effect at the next
   boundary (between one file and another, between one asset and another). Each file is already
   written in its own transaction, so what was already downloaded stays. A canceled automatic collection does not return
   right away: it waits an hour and resumes where it stopped (the ETags avoid downloading again what did not
   change). When requested by hand, it runs right away.
5. **In memory.** The registry lives in memory (`crate::tasks`), with no table in the vault. Finished
   tasks stay 30 minutes in the list, like the agent's.
6. **No extra parameter.** The current task is kept in the thread that runs it (`tasks::current`). This way, the
   loop that downloads the files reports progress and checks for cancellation without receiving a new
   parameter. The update download, which is asynchronous, uses the task handle directly.
7. **Notify without polling constantly.** The core announces the start and end of each task
   (`tasks://changed`). The interface calls `tasks.list` every second only while a task is
   running; when idle, it rereads only when the notice arrives or every 30 s, like the agent's jobs.

## Tasks

| Type (`TaskKind`) | When | Progress | Cancel |
| --- | --- | --- | --- |
| `market_sync`: indices, exchange rates, quotes and announced dividends | every 12 h and on the Refresh button | steps and assets (n/total) | yes |
| `indices_sync`: indices and exchange rates only | button on the indices screen | series and currencies (n/total) | yes |
| `research_sync`: fundamentals (CVM, SEC, B3, IF.data) and the market score | every 24 h and on the Refresh button | files downloaded | yes |
| `mail_sync`: ingestion email of an account | at the account's interval and on manual fetch | indeterminate | no |
| `update_download`: app update | when a new version is found | bytes (n/total) | no |

The CVM unit price history (funds and pension) is part of `market_sync`, with a bar by
month.

## What does not show up in the list

- Quick routines (daily backup, intraday quotes, lock): they take less than a second and
  would only flicker in the list.
- Asset enrichment: it is local, with no network, and runs inside the command that requests it.
