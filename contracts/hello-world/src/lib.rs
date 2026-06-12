#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, String,
};

#[contracttype]
#[derive(Clone)]
pub struct Certificate {
    pub student_name: String,
    pub course_name: String,
    pub issue_date: u64,
    pub issuer: Address,
    pub certificate_hash: String,
    pub revoked: bool,
}

#[contracttype]
pub enum DataKey {
    Certificate(u64),
    Admin,
}

#[contract]
pub struct EduCertificateChain;

#[contractimpl]
impl EduCertificateChain {

    // Initialize contract with administrator
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Admin, &admin);
    }

    // Issue a new certificate
    pub fn issue_certificate(
        env: Env,
        cert_id: u64,
        student_name: String,
        course_name: String,
        certificate_hash: String,
    ) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap();

        admin.require_auth();

        if env.storage().persistent().has(&DataKey::Certificate(cert_id)) {
            panic!("Certificate already exists");
        }

        let certificate = Certificate {
            student_name,
            course_name,
            issue_date: env.ledger().timestamp(),
            issuer: admin,
            certificate_hash,
            revoked: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Certificate(cert_id), &certificate);
    }

    // Revoke an existing certificate
    pub fn revoke_certificate(env: Env, cert_id: u64) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap();

        admin.require_auth();

        let mut certificate: Certificate = env
            .storage()
            .persistent()
            .get(&DataKey::Certificate(cert_id))
            .unwrap();

        certificate.revoked = true;

        env.storage()
            .persistent()
            .set(&DataKey::Certificate(cert_id), &certificate);
    }

    // Retrieve certificate information
    pub fn get_certificate(
        env: Env,
        cert_id: u64,
    ) -> Certificate {
        env.storage()
            .persistent()
            .get(&DataKey::Certificate(cert_id))
            .unwrap()
    }

    // Verify certificate validity
    pub fn verify_certificate(
        env: Env,
        cert_id: u64,
    ) -> bool {
        let certificate: Certificate = env
            .storage()
            .persistent()
            .get(&DataKey::Certificate(cert_id))
            .unwrap();

        !certificate.revoked
    }
}