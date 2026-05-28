# Governance and Bootstrap Policy

## MVP Trust Model

The MVP uses one operator/admin key per contract. That key is a documented trust point for setup and future upgrade authority while the product proves the clinical access predicate, prescription bridge, and supply-chain workflows.

The admin key is not allowed to bypass patient access controls, grant expiry, revocation, veto, credential checks, or key-release predicates. Those predicates remain real from day one. The admin key only configures contract dependencies and authority registries.

## Bootstrap Rules

- Identity records its admin during `initialize(admin)`. Issuer registration is admin-only after initialization.
- Access Broker records its admin during `initialize(admin)`. The issuer root is configured by `configure_issuer_root(admin, issuer_root)`.
- Prescription records its admin during `initialize(admin)`. Identity, Access Broker, and Supply Chain contract IDs are configured by `configure_dependencies(...)`.
- Supply Chain records its admin during `initialize(admin)`. Cold-chain oracles and opposing-interest attesters are registered by admin-only functions.

All admin setup functions call `require_auth()` on the admin argument and return typed contract errors for uninitialized or unauthorized calls.

## Upgrade Path

The named successor path is:

1. Replace the single operator key with a multisig-controlled admin address.
2. Add a timelock for non-emergency administrative changes.
3. Move long-term protocol changes to stakeholder governance after external audit and pilot completion.

Until that migration, deployments must treat admin-key custody as a production security control and keep the operator key isolated from application hot paths.
