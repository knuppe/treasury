-- Structured "Rendimentos e Amortizações" notices that listed real estate and agribusiness
-- funds (FII, Fiagro) file on Fundos.NET, the CVM and B3 disclosure system. Technical table,
-- like the other research_* tables: outside the change history and the backups.
CREATE TABLE research_fund_notice (
    document_id   INTEGER NOT NULL,            -- document id on Fundos.NET
    version       INTEGER NOT NULL DEFAULT 1,  -- 2+ = the fund filed it again (corrected)
    cnpj          TEXT NOT NULL,               -- fund CNPJ, digits only
    ticker        TEXT NOT NULL,               -- trading code of the unit or the subscription receipt
    isin          TEXT,
    kind          TEXT NOT NULL CHECK (kind IN ('fii_income', 'amortization')),
    amortization  TEXT,                        -- "Total" or "Parcial", as the notice says
    ex_date       TEXT NOT NULL,               -- record date ("data-base": last day with the right)
    value         TEXT NOT NULL,               -- amount per unit, exact decimal
    pay_date      TEXT,
    period        TEXT,                        -- reference month as the notice writes it
    exempt        INTEGER CHECK (exempt IN (0, 1)),  -- income tax exempt for individuals
    delivered_at  TEXT,                        -- when the fund filed the document
    source        TEXT NOT NULL,               -- fundos_net
    fetched_at    TEXT NOT NULL,
    PRIMARY KEY (document_id, ticker, kind, ex_date)
) STRICT;

CREATE INDEX research_fund_notice_ticker ON research_fund_notice (ticker, ex_date);
CREATE INDEX research_fund_notice_cnpj ON research_fund_notice (cnpj);
