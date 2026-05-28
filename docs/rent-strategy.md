# Access Broker Rent Strategy

## Critical State

The MVP operator sponsors renewal for Access Broker state that must remain available for clinical access:

- `Record(record_id)` entries with encrypted-record locators and commitments.
- `PatientToken(patient)` entries used by presence proofs.
- Access Broker instance state containing admin and issuer-root configuration.
- Active normal `Grant(grant_id)` entries that are not revoked, not vetoed, and not past `expires_at`.

Break-glass and tokenless-fallback grants remain temporary storage entries. Their authorization semantics come from `expires_at`, `reveal_at`, `revoked`, and `vetoed` fields, never from the storage TTL.

## Renewal Policy

Every write to critical persistent broker state extends that entry's TTL through centralized storage helpers. The operator also runs a sponsored renewal job that calls `renew_critical_state(admin, record_ids, patients, grant_ids)` with currently active IDs from the off-chain indexer.

The renewal job is allowed to preserve availability. It is not allowed to create positive authorization. Missing entries remain absent, expired grants remain expired, revoked/vetoed grants remain denied, and grants before `reveal_at` remain unreleasable.

## Failure Mode

If rent lapses and a critical entry is missing, the contract fails closed: record lookup returns `NoSuchRecord`, patient-token lookup returns `NoTokenRegistered`, and grant lookup returns `NoGrant`. A legitimate emergency read failing because critical state lapsed is an operator correctness defect, not an authorization success.
