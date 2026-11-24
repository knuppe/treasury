-- Costs a loan charges on day zero, in cents: IOF, one-time fees (registration, appraisal,
-- registry) and single-premium insurance. The CET (Res. CMN 4.881/2020) subtracts them from the
-- amount released; empty means not informed.
ALTER TABLE loan ADD COLUMN upfront_iof INTEGER CHECK (upfront_iof >= 0);
ALTER TABLE loan ADD COLUMN upfront_fees INTEGER CHECK (upfront_fees >= 0);
ALTER TABLE loan ADD COLUMN upfront_insurance INTEGER CHECK (upfront_insurance >= 0);
