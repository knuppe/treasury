-- B3 import: which account each custodian of the B3 reports goes to, remembered between
-- imports. `key` is the custodian's normalized name (aliases of a renamed broker share it),
-- plus the B3 account number when the position report brings one ("<name>#<account>").
-- An archived or deleted account invalidates the row (the import falls back to the
-- suggestion).
CREATE TABLE import_institution_map (
    source      TEXT NOT NULL,                 -- 'b3'
    key         TEXT NOT NULL,
    account_id  TEXT NOT NULL REFERENCES account (id) ON DELETE CASCADE,
    updated_at  TEXT NOT NULL,
    PRIMARY KEY (source, key)
) STRICT;

-- CNPJ of the institution (digits only), to recognize a custodian by it.
ALTER TABLE institution ADD COLUMN cnpj TEXT;

-- Bonus shares credited in one account when there is no position before the event (the
-- factor cannot be derived): the event carries the credited quantity instead of a factor.
ALTER TABLE corporate_action ADD COLUMN account_id TEXT REFERENCES account (id);
ALTER TABLE corporate_action ADD COLUMN quantity TEXT;
