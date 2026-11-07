# Security Hickory split — unfinished

Mark split this work from fix/security-advisories. Original branch history remains unchanged. All previously uncommitted adapters were committed first at 2dfcb234. The branch intentionally retains the full original history so no work is lost. The smaller non-Hickory PR is fix/security-tls.

Next: resolve the exact mail-auth 0.11.0 pin through CBC, apply the lockfile and formatter diffs, compile/test DNS and mail users, run security and full gates, then start fresh independent reviews. The lock currently contains mail-auth 0.11.2. No production installation or deployment.

Branch fix/security-hickory; head/source 2dfcb234 (this status commit follows); gate NOT PASSED — unfinished; CBC pass 20261107-204531-0eb24e83-a1, failed compile 20261107-210955-bb0ec842-a1 / 20261107-210956-17bf7a0d-a1 / 20261107-211704-681cf6d0-a1 / 20261107-211705-af40aaa6-a1; diagnostic jobs 20261107-211707-a72fecd8 / 20261107-212635-39b09297 have no accepted final receipt here.
