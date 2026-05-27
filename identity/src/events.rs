use soroban_sdk::{symbol_short, Address, BytesN, Env, Symbol};

pub fn publish_issuer_registered(env: &Env, admin: &Address, issuer: &Address) {
    env.events()
        .publish((symbol_short!("issuer"), admin.clone()), issuer.clone());
}

pub fn publish_credential_issued(
    env: &Env,
    issuer: &Address,
    cred_id: &BytesN<32>,
    subject: &Address,
    role_code: u32,
) {
    env.events().publish(
        (Symbol::new(env, "cred_issue"), issuer.clone()),
        (cred_id.clone(), subject.clone(), role_code),
    );
}

pub fn publish_credential_revoked(env: &Env, issuer_or_admin: &Address, cred_id: &BytesN<32>) {
    env.events().publish(
        (symbol_short!("cred_rev"), issuer_or_admin.clone()),
        cred_id.clone(),
    );
}
