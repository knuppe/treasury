-- Closing date of an account (`YYYY-MM-DD`), for a checking or brokerage account the family
-- closed at the bank. A closed account is also archived (`archived = 1`), so every place that
-- leaves archived accounts out of lists and alerts leaves it out too; the date keeps the
-- history readable ("closed on") and blocks new transactions after it. Reopening clears both.
ALTER TABLE account ADD COLUMN closed_on TEXT;
