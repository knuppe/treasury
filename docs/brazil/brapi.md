# brapi.dev

brapi.dev is a B3 data API with a free plan and paid plans, optional and used with the person's
own token. In Treasury it is a **fallback for B3 quotes**, a way to **resolve renamed tickers**
and a **secondary source** of asset data; it does not replace any official source. The token
lives in Settings › Data sources › brapi.dev.

## What the app uses

| Use | Plan | How |
|---|---|---|
| Fallback quote | free | When the Yahoo collection fails for a B3 security, brapi gives the day's quote (at most one per security per day). The person can also choose brapi as the quote source of an asset. The quote shows the source and the delay (~30 min on the free plan). |
| Renamed tickers | no token | `/api/v2/tickers/resolve` finds the new code of a security after a merger or code change; the notice in the asset's data suggests the new code, item by item, and never swaps it on its own. |
| Check of indicators and dividends | paid | One query per registered security every 20 h compares the score's indicators and the dividends with the official sources (CVM, B3). On the free plan these modules do not exist: the check is off, with an explanation, without spending quota. |
| Asset data | depends on the plan | A brapi read counts as a secondary source (level 3) in enrichment ([agent-enrichment.md](../agent-enrichment.md)). |

Selic, CDI and IPCA still come from the Banco Central, and FIIs (real estate funds), Tesouro and
funds from the official sources.

## Plan and quota

- The plan and the usage come from the API itself (`/api/v2/user/usage`, read at most every 6 h),
  and the app counts the requests it makes between one read and the next. The person does not
  choose the plan: the app detects it from what the API accepts.
- The app stops before 80% of the month's quota (setting `research.brapi_limit_pct`), leaving the
  rest for other uses of the token.
- A 403 response (outside the plan) turns off only that use, without marking the source as
  broken; 429 respects `Retry-After`.
- When checked (10/22/2026, brapi.dev/pricing), the free plan had: 15,000 requests per month,
  1 asset per request, 3 months of history and only the basic quote. With 1 asset per request
  that is ~500 requests per day: it is not enough to quote a large portfolio every 30 minutes,
  which is why brapi is a fallback and not the main source.

## Privacy

- Only the ticker and the token go to brapi. Nothing says the asset is in a portfolio, nor the
  quantity, nor the value.
- The token lives in the encrypted vault (setting `research.brapi_token`) and goes in the
  `Authorization` header, never in a URL, log or collection report.

## In the code

- `crates/treasury-core/src/research/brapi.rs` (client, plan, quota, ticker resolution) and the
  quote fallback in `crates/treasury-core/src/market/sync.rs`.
- Tests with fixed responses, no network.

## Limitations

- Use with a real token has not yet been checked end to end; plan detection depends on the API's
  responses.
