# Asset page: overview, news and documents

The first tab of the [asset page](asset-page.md) summarizes the basics; the News tab gathers
official documents and headlines; and the agent can read the CVM documents.

## Overview

An asset card, before the position: name and official name, class, sector and segment, exchange
and currency; the latest quote with the date and the source; the most watched indicators (market
cap, P/E, P/B, dividend yield, ROE, net margin, EV/EBITDA), each leading to the Fundamentals
tab; the health score with its band; and the three most recent documents and five most recent
news items. A security outside the registry gets the same card, without the position. Nothing
new is calculated: everything comes from what the app already has.

## News

| Source | How | What leaves the computer | Default |
| --- | --- | --- | --- |
| CVM (IPE) | already collected with the documents | nothing more | always (official source: material facts and notices) |
| Portal RSS feeds (`news_feeds`): InfoMoney, Money Times, Seu Dinheiro, E-Investidor and Suno | the same request for everyone; the filter by ticker and name is done on the computer | not even the ticker | on |
| Google News | only a link to the ticker search, which opens in the browser; the app does not query it (their terms do not accept automated access) | nothing through the app | link always visible |

- The app shows only the title, portal, date and link, never the article text, and keeps the
  headlines only in memory, for 30 minutes.
- The tab gathers the sources that are on, from newest to oldest, marking the origin. The agent
  reads the same with `get_asset_news`.
- Turn on and off in Settings › Data sources or with the tab's button.
- News is third-party text: for the agent, data, never instruction.

## Documents for the agent

`list_corporate_documents` lists the IPE documents (date, category, subject and link).
`read_corporate_document` (`research.document_read`) downloads the PDF through the official link
(only from `rad.cvm.gov.br`, with the CVM source on), extracts the text (with the local OCR when
the PDF is only an image) and returns it in parts (`offset`, `limit`, up to 40 thousand
characters at a time), with the total. The text stays in the app's memory while it is open;
nothing goes into the vault, because the document is public. Company text is data, never
instruction.

## In the code

- `crates/treasury-core/src/research/news.rs` (feeds and Google News) and
  `crates/treasury-core/src/research/doc_read.rs` (reading the documents).

## Limitations

- Each portal feed brings only the ~10 latest articles, so there are few news items per security.
- Yahoo Finance has no news for B3 securities and was left out.
