#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    symbol_short, Address, Env, String, Vec,
};

#[contracttype]
#[derive(Clone)]
pub enum CertificateType {
    Degree,
    Diploma,
    CourseCompletion,
    ProfessionalLicense,
    AchievementBadge,
}

#[contracttype]
#[derive(Clone)]
pub struct Certificate {
    pub cert_id: u64,
    pub student_wallet: Address,
    pub student_name: String,
    pub institution_name: String,
    pub course_name: String,
    pub cert_type: CertificateType,
    pub issue_date: u64,
    pub expiration_date: u64,
    pub metadata_uri: String,
    pub issuer: Address,
    pub revoked: bool,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Issuer(Address),
    Certificate(u64),
    StudentCertificates(Address),
}

#[contract]
pub struct EduCertificateChain;

#[contractimpl]
impl EduCertificateChain {
    // Initialize contract
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    // Add issuer
    pub fn add_issuer(env: Env, issuer: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap();

        admin.require_auth();

        env.storage()
            .persistent()
            .set(&DataKey::Issuer(issuer.clone()), &true);

        env.events().publish(
            (symbol_short!("issuer"), symbol_short!("add")),
            issuer,
        );
    }

    // Remove issuer
    pub fn remove_issuer(env: Env, issuer: Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap();

        admin.require_auth();

        env.storage()
            .persistent()
            .remove(&DataKey::Issuer(issuer.clone()));

        env.events().publish(
            (symbol_short!("issuer"), symbol_short!("remove")),
            issuer,
        );
    }

    // Check issuer
    pub fn is_issuer(env: Env, issuer: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Issuer(issuer))
            .unwrap_or(false)
    }

    // Issue certificate
    #[allow(clippy::too_many_arguments)]
    pub fn issue_certificate(
        env: Env,
        cert_id: u64,
        student_wallet: Address,
        student_name: String,
        institution_name: String,
        course_name: String,
        cert_type: CertificateType,
        expiration_date: u64,
        metadata_uri: String,
        issuer: Address,
    ) {
        issuer.require_auth();

        let authorized: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Issuer(issuer.clone()))
            .unwrap_or(false);

        if !authorized {
            panic!("Unauthorized issuer");
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::Certificate(cert_id))
        {
            panic!("Certificate already exists");
        }

        let certificate = Certificate {
            cert_id,
            student_wallet: student_wallet.clone(),
            student_name,
            institution_name,
            course_name,
            cert_type,
            issue_date: env.ledger().timestamp(),
            expiration_date,
            metadata_uri,
            issuer,
            revoked: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Certificate(cert_id), &certificate);

        let mut student_certs: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::StudentCertificates(student_wallet.clone()))
            .unwrap_or(Vec::new(&env));

        student_certs.push_back(cert_id);

        env.storage()
            .persistent()
            .set(
                &DataKey::StudentCertificates(student_wallet),
                &student_certs,
            );

        env.events().publish(
            (symbol_short!("cert"), symbol_short!("issue")),
            cert_id,
        );
    }

    // Revoke certificate
    pub fn revoke_certificate(
        env: Env,
        cert_id: u64,
        issuer: Address,
    ) {
        issuer.require_auth();

        let mut cert: Certificate = env
            .storage()
            .persistent()
            .get(&DataKey::Certificate(cert_id))
            .unwrap();

        if cert.issuer != issuer {
            panic!("Not certificate issuer");
        }

        cert.revoked = true;

        env.storage()
            .persistent()
            .set(&DataKey::Certificate(cert_id), &cert);

        env.events().publish(
            (symbol_short!("cert"), symbol_short!("revoke")),
            cert_id,
        );
    }

    // Get certificate
    pub fn get_certificate(
        env: Env,
        cert_id: u64,
    ) -> Certificate {
        env.storage()
            .persistent()
            .get(&DataKey::Certificate(cert_id))
            .unwrap()
    }

    // Get all certificates of a student
    pub fn get_student_certificates(
        env: Env,
        student: Address,
    ) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::StudentCertificates(student))
            .unwrap_or(Vec::new(&env))
    }

    // Verify certificate
    pub fn verify_certificate(
        env: Env,
        cert_id: u64,
    ) -> bool {
        let cert: Certificate = env
            .storage()
            .persistent()
            .get(&DataKey::Certificate(cert_id))
            .unwrap();

        let now = env.ledger().timestamp();

        if cert.revoked {
            return false;
        }

        if cert.expiration_date != 0
            && now > cert.expiration_date
        {
            return false;
        }

        true
    }
}