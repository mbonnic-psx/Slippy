# Retirement ledger

What `/strangle` has moved out of the code that existed before the delivery method did, one row per capability:
from where, to where, routed by what, pinned by which tests, and where it stands. A capability is *routed* when
the seam can send its traffic either way, *moved* when the new home serves it, *retired* when the old path
serves none of it, and *removed* when the old code is gone. Rows are appended, never rewritten; a status changes
by a new row. The programme is finished when every row is *removed* — and a ledger nobody writes to is the first
sign that a programme has quietly become a second system beside the first.

| Date | Capability | From | To | Routed by | Pinned by | Status |
|---|---|---|---|---|---|---|
