#![no_std]
#![allow(deprecated)]
use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, BytesN, Env, Symbol, Vec};
use nbbs_shared::{OracleError, ReportStatus};

pub const CHALLENGE_WINDOW_SECONDS: u64 = 259200;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Provider(Address),
    ProviderList,
    Report(u64),
    ReportCount,
    ProjectReports(BytesN<32>),
    Challenge(u64),
    SignatureThreshold,
    ChallengeWindow,
    Nonce(Address),
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct OracleProvider {
    pub address: Address,
    pub methodology: Symbol,
    pub stake: i128,
    pub active: bool,
    pub registered_at: u64,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct Report {
    pub id: u64,
    pub provider: Address,
    pub project_id: BytesN<32>,
    pub period_start: u64,
    pub period_end: u64,
    pub carbon_sequestered: i128,
    pub methodology: Symbol,
    pub ipfs_evidence_hash: BytesN<32>,
    pub status: ReportStatus,
    pub submitted_at: u64,
    pub verified_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct Challenge {
    pub report_id: u64,
    pub challenger: Address,
    pub counter_evidence_hash: BytesN<32>,
    pub submitted_at: u64,
    pub resolved: bool,
    pub resolution: u32,
}

fn require_admin(env: &Env, caller: &Address) -> Result<(), OracleError> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(OracleError::NotInitialized)?;
    if caller != &admin {
        return Err(OracleError::Unauthorized);
    }
    Ok(())
}

fn get_nonce(env: &Env, addr: &Address) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::Nonce(addr.clone()))
        .unwrap_or(0)
}

fn set_nonce(env: &Env, addr: &Address, nonce: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::Nonce(addr.clone()), &nonce);
}

#[contract]
pub struct OracleConsumer;

#[allow(clippy::too_many_arguments)]
#[contractimpl]
impl OracleConsumer {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ChallengeWindow, &CHALLENGE_WINDOW_SECONDS);
        env.storage()
            .instance()
            .set(&DataKey::SignatureThreshold, &1u32);
    }

    pub fn register_provider(
        env: Env,
        caller: Address,
        provider: Address,
        methodology: Symbol,
        nonce: u64,
    ) -> Result<(), OracleError> {
        caller.require_auth();

        let expected_nonce = get_nonce(&env, &caller);
        if nonce != expected_nonce {
            return Err(OracleError::InvalidNonce);
        }
        set_nonce(&env, &caller, expected_nonce + 1);

        require_admin(&env, &caller)?;

        if env
            .storage()
            .instance()
            .has(&DataKey::Provider(provider.clone()))
        {
            return Err(OracleError::ProviderAlreadyExists);
        }

        let oracle_provider = OracleProvider {
            address: provider.clone(),
            methodology,
            stake: 0,
            active: true,
            registered_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&DataKey::Provider(provider.clone()), &oracle_provider);

        let mut providers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::ProviderList)
            .unwrap_or(vec![&env]);
        providers.push_back(provider.clone());
        env.storage()
            .instance()
            .set(&DataKey::ProviderList, &providers);

        env.events().publish(
            (Symbol::new(&env, "provider_registered"),),
            (provider,),
        );

        Ok(())
    }

    pub fn remove_provider(
        env: Env,
        caller: Address,
        provider: Address,
        nonce: u64,
    ) -> Result<(), OracleError> {
        caller.require_auth();

        let expected_nonce = get_nonce(&env, &caller);
        if nonce != expected_nonce {
            return Err(OracleError::InvalidNonce);
        }
        set_nonce(&env, &caller, expected_nonce + 1);

        require_admin(&env, &caller)?;

        let mut p: OracleProvider = env
            .storage()
            .instance()
            .get(&DataKey::Provider(provider.clone()))
            .ok_or(OracleError::ProviderNotFound)?;

        p.active = false;
        env.storage()
            .instance()
            .set(&DataKey::Provider(provider.clone()), &p);

        env.events().publish(
            (Symbol::new(&env, "provider_removed"),),
            (provider,),
        );

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn submit_report(
        env: Env,
        provider: Address,
        project_id: BytesN<32>,
        period_start: u64,
        period_end: u64,
        carbon_sequestered: i128,
        methodology: Symbol,
        ipfs_evidence_hash: BytesN<32>,
        nonce: u64,
    ) -> Result<u64, OracleError> {
        provider.require_auth();

        let expected_nonce = get_nonce(&env, &provider);
        if nonce != expected_nonce {
            return Err(OracleError::InvalidNonce);
        }
        set_nonce(&env, &provider, expected_nonce + 1);

        let p: OracleProvider = env
            .storage()
            .instance()
            .get(&DataKey::Provider(provider.clone()))
            .ok_or(OracleError::ProviderNotFound)?;

        if !p.active {
            return Err(OracleError::Unauthorized);
        }

        if period_end <= period_start || carbon_sequestered < 0 {
            return Err(OracleError::InvalidSignature);
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ReportCount)
            .unwrap_or(0);
        let report_id = count + 1;
        env.storage()
            .instance()
            .set(&DataKey::ReportCount, &report_id);

        let now = env.ledger().timestamp();
        let report = Report {
            id: report_id,
            provider: provider.clone(),
            project_id: project_id.clone(),
            period_start,
            period_end,
            carbon_sequestered,
            methodology,
            ipfs_evidence_hash,
            status: ReportStatus::Pending,
            submitted_at: now,
            verified_at: 0,
        };

        env.storage()
            .instance()
            .set(&DataKey::Report(report_id), &report);

        let mut project_reports: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::ProjectReports(project_id.clone()))
            .unwrap_or(vec![&env]);
        project_reports.push_back(report_id);
        env.storage()
            .instance()
            .set(&DataKey::ProjectReports(project_id.clone()), &project_reports);

        env.events().publish(
            (Symbol::new(&env, "report_submitted"),),
            (report_id, provider, project_id),
        );

        Ok(report_id)
    }

    pub fn verify_report(
        env: Env,
        caller: Address,
        report_id: u64,
        nonce: u64,
    ) -> Result<(), OracleError> {
        caller.require_auth();

        let expected_nonce = get_nonce(&env, &caller);
        if nonce != expected_nonce {
            return Err(OracleError::InvalidNonce);
        }
        set_nonce(&env, &caller, expected_nonce + 1);

        let mut report: Report = env
            .storage()
            .instance()
            .get(&DataKey::Report(report_id))
            .ok_or(OracleError::ReportNotFound)?;

        if report.status != ReportStatus::Pending {
            return Err(OracleError::ReportAlreadyVerified);
        }

        if env
            .storage()
            .instance()
            .has(&DataKey::Challenge(report_id))
        {
            return Err(OracleError::ReportAlreadyVerified);
        }

        report.status = ReportStatus::Verified;
        report.verified_at = env.ledger().timestamp();
        env.storage()
            .instance()
            .set(&DataKey::Report(report_id), &report);

        env.events().publish(
            (Symbol::new(&env, "report_verified"),),
            (report_id,),
        );

        Ok(())
    }

    pub fn challenge_report(
        env: Env,
        challenger: Address,
        report_id: u64,
        counter_evidence_hash: BytesN<32>,
        nonce: u64,
    ) -> Result<(), OracleError> {
        challenger.require_auth();

        let expected_nonce = get_nonce(&env, &challenger);
        if nonce != expected_nonce {
            return Err(OracleError::InvalidNonce);
        }
        set_nonce(&env, &challenger, expected_nonce + 1);

        let report: Report = env
            .storage()
            .instance()
            .get(&DataKey::Report(report_id))
            .ok_or(OracleError::ReportNotFound)?;

        if report.status != ReportStatus::Pending {
            return Err(OracleError::ReportAlreadyVerified);
        }

        let now = env.ledger().timestamp();
        let window: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ChallengeWindow)
            .unwrap_or(CHALLENGE_WINDOW_SECONDS);
        if now - report.submitted_at > window {
            return Err(OracleError::ChallengeWindowExpired);
        }

        if env
            .storage()
            .instance()
            .has(&DataKey::Challenge(report_id))
        {
            return Err(OracleError::ProviderAlreadyExists);
        }

        let challenge = Challenge {
            report_id,
            challenger: challenger.clone(),
            counter_evidence_hash,
            submitted_at: now,
            resolved: false,
            resolution: 0,
        };
        env.storage()
            .instance()
            .set(&DataKey::Challenge(report_id), &challenge);

        let mut report_mut: Report = env
            .storage()
            .instance()
            .get(&DataKey::Report(report_id))
            .unwrap();
        report_mut.status = ReportStatus::Challenged;
        env.storage()
            .instance()
            .set(&DataKey::Report(report_id), &report_mut);

        env.events().publish(
            (Symbol::new(&env, "report_challenged"),),
            (report_id, challenger),
        );

        Ok(())
    }

    pub fn resolve_challenge(
        env: Env,
        caller: Address,
        report_id: u64,
        resolution: ReportStatus,
        nonce: u64,
    ) -> Result<(), OracleError> {
        caller.require_auth();

        let expected_nonce = get_nonce(&env, &caller);
        if nonce != expected_nonce {
            return Err(OracleError::InvalidNonce);
        }
        set_nonce(&env, &caller, expected_nonce + 1);

        require_admin(&env, &caller)?;

        let mut challenge: Challenge = env
            .storage()
            .instance()
            .get(&DataKey::Challenge(report_id))
            .ok_or(OracleError::ReportNotFound)?;

        if challenge.resolved {
            return Ok(());
        }

        challenge.resolved = true;
        challenge.resolution = resolution as u32;
        env.storage()
            .instance()
            .set(&DataKey::Challenge(report_id), &challenge);

        let mut report: Report = env
            .storage()
            .instance()
            .get(&DataKey::Report(report_id))
            .ok_or(OracleError::ReportNotFound)?;
        report.status = resolution;
        env.storage()
            .instance()
            .set(&DataKey::Report(report_id), &report);

        env.events().publish(
            (Symbol::new(&env, "challenge_resolved"),),
            (report_id,),
        );

        Ok(())
    }

    pub fn get_provider(env: Env, provider: Address) -> Result<OracleProvider, OracleError> {
        env.storage()
            .instance()
            .get(&DataKey::Provider(provider))
            .ok_or(OracleError::ProviderNotFound)
    }

    pub fn get_report(env: Env, report_id: u64) -> Result<Report, OracleError> {
        env.storage()
            .instance()
            .get(&DataKey::Report(report_id))
            .ok_or(OracleError::ReportNotFound)
    }

    pub fn list_providers(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::ProviderList)
            .unwrap_or(vec![&env])
    }

    pub fn get_project_reports(env: Env, project_id: BytesN<32>) -> Vec<u64> {
        env.storage()
            .instance()
            .get(&DataKey::ProjectReports(project_id))
            .unwrap_or(vec![&env])
    }

    pub fn get_challenge(env: Env, report_id: u64) -> Result<Challenge, OracleError> {
        env.storage()
            .instance()
            .get(&DataKey::Challenge(report_id))
            .ok_or(OracleError::ReportNotFound)
    }

    pub fn set_signature_threshold(
        env: Env,
        caller: Address,
        threshold: u32,
        nonce: u64,
    ) -> Result<(), OracleError> {
        caller.require_auth();

        let expected_nonce = get_nonce(&env, &caller);
        if nonce != expected_nonce {
            return Err(OracleError::InvalidNonce);
        }
        set_nonce(&env, &caller, expected_nonce + 1);

        require_admin(&env, &caller)?;

        env.storage()
            .instance()
            .set(&DataKey::SignatureThreshold, &threshold);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        BytesN, Env, Symbol,
    };

    fn create_project_id(env: &Env, value: u8) -> BytesN<32> {
        let mut arr = [0u8; 32];
        arr[31] = value;
        BytesN::from_array(env, &arr)
    }

    fn make_ipfs_hash(env: &Env, value: u8) -> BytesN<32> {
        let mut arr = [0u8; 32];
        arr[0] = value;
        BytesN::from_array(env, &arr)
    }

    #[test]
    fn test_register_provider_and_submit_report() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let provider = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let contract_id = env.register(OracleConsumer, (admin.clone(),));
        let client = OracleConsumerClient::new(&env, &contract_id);

        env.ledger().set_timestamp(1_000_000);
        client.register_provider(&admin, &provider, &Symbol::new(&env, "verra_vcs"), &0);

        let report_id = client.submit_report(
            &provider,
            &project_id,
            &1000u64,
            &2000u64,
            &100_000i128,
            &Symbol::new(&env, "verra_vcs"),
            &make_ipfs_hash(&env, 1),
            &0,
        );
        assert_eq!(report_id, 1);

        let stored = client.get_report(&report_id);
        assert_eq!(stored.status, ReportStatus::Pending);
        assert_eq!(stored.provider, provider);
        assert_eq!(stored.carbon_sequestered, 100_000);

        env.ledger().set_timestamp(1_000_001);
        client.verify_report(&admin, &report_id, &1);

        let verified = client.get_report(&report_id);
        assert_eq!(verified.status, ReportStatus::Verified);
        assert_eq!(verified.verified_at, 1_000_001);

        let providers = client.list_providers();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers.get(0).unwrap(), provider);

        let project_reports = client.get_project_reports(&project_id);
        assert_eq!(project_reports.len(), 1);
        assert_eq!(project_reports.get(0).unwrap(), report_id);
    }

    #[test]
    fn test_submit_challenge_and_resolve() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let provider = Address::generate(&env);
        let challenger = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let contract_id = env.register(OracleConsumer, (admin.clone(),));
        let client = OracleConsumerClient::new(&env, &contract_id);

        client.register_provider(&admin, &provider, &Symbol::new(&env, "verra_vcs"), &0);

        let report_id = client.submit_report(
            &provider,
            &project_id,
            &1000u64,
            &2000u64,
            &100_000i128,
            &Symbol::new(&env, "verra_vcs"),
            &make_ipfs_hash(&env, 1),
            &0,
        );

        client.challenge_report(
            &challenger,
            &report_id,
            &make_ipfs_hash(&env, 2),
            &0,
        );

        let challenged = client.get_report(&report_id);
        assert_eq!(challenged.status, ReportStatus::Challenged);

        let challenge = client.get_challenge(&report_id);
        assert_eq!(challenge.report_id, report_id);
        assert_eq!(challenge.challenger, challenger);
        assert!(!challenge.resolved);

        client.resolve_challenge(
            &admin,
            &report_id,
            &ReportStatus::Verified,
            &1,
        );

        let resolved = client.get_report(&report_id);
        assert_eq!(resolved.status, ReportStatus::Verified);

        let stored_challenge = client.get_challenge(&report_id);
        assert!(stored_challenge.resolved);
        assert_eq!(stored_challenge.resolution, ReportStatus::Verified as u32);
    }

    #[test]
    fn test_late_challenge() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let provider = Address::generate(&env);
        let challenger = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let contract_id = env.register(OracleConsumer, (admin.clone(),));
        let client = OracleConsumerClient::new(&env, &contract_id);

        env.ledger().set_timestamp(1_000_000);

        client.register_provider(&admin, &provider, &Symbol::new(&env, "verra_vcs"), &0);

        let report_id = client.submit_report(
            &provider,
            &project_id,
            &1_000_100u64,
            &1_000_200u64,
            &100_000i128,
            &Symbol::new(&env, "verra_vcs"),
            &make_ipfs_hash(&env, 1),
            &0,
        );

        env.ledger().set_timestamp(1_000_000 + CHALLENGE_WINDOW_SECONDS + 1);

        let result = client.try_challenge_report(
            &challenger,
            &report_id,
            &make_ipfs_hash(&env, 2),
            &0,
        );
        assert_eq!(result, Err(Ok(OracleError::ChallengeWindowExpired)));
    }

    #[test]
    fn test_submit_from_non_registered() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let rogue = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let contract_id = env.register(OracleConsumer, (admin.clone(),));
        let client = OracleConsumerClient::new(&env, &contract_id);

        let result = client.try_submit_report(
            &rogue,
            &project_id,
            &1000u64,
            &2000u64,
            &100_000i128,
            &Symbol::new(&env, "verra_vcs"),
            &make_ipfs_hash(&env, 1),
            &0,
        );
        assert_eq!(result, Err(Ok(OracleError::ProviderNotFound)));
    }

    #[test]
    fn test_duplicate_provider() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let provider = Address::generate(&env);

        let contract_id = env.register(OracleConsumer, (admin.clone(),));
        let client = OracleConsumerClient::new(&env, &contract_id);

        client.register_provider(&admin, &provider, &Symbol::new(&env, "verra_vcs"), &0);

        let result = client.try_register_provider(
            &admin,
            &provider,
            &Symbol::new(&env, "verra_vcs"),
            &1,
        );
        assert_eq!(result, Err(Ok(OracleError::ProviderAlreadyExists)));
    }

    #[test]
    fn test_double_verify() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let provider = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let contract_id = env.register(OracleConsumer, (admin.clone(),));
        let client = OracleConsumerClient::new(&env, &contract_id);

        client.register_provider(&admin, &provider, &Symbol::new(&env, "verra_vcs"), &0);

        let report_id = client.submit_report(
            &provider,
            &project_id,
            &1000u64,
            &2000u64,
            &100_000i128,
            &Symbol::new(&env, "verra_vcs"),
            &make_ipfs_hash(&env, 1),
            &0,
        );

        client.verify_report(&admin, &report_id, &1);

        let result = client.try_verify_report(&provider, &report_id, &1);
        assert_eq!(result, Err(Ok(OracleError::ReportAlreadyVerified)));
    }

    #[test]
    fn test_challenge_verified_report() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let provider = Address::generate(&env);
        let challenger = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let contract_id = env.register(OracleConsumer, (admin.clone(),));
        let client = OracleConsumerClient::new(&env, &contract_id);

        client.register_provider(&admin, &provider, &Symbol::new(&env, "verra_vcs"), &0);

        let report_id = client.submit_report(
            &provider,
            &project_id,
            &1000u64,
            &2000u64,
            &100_000i128,
            &Symbol::new(&env, "verra_vcs"),
            &make_ipfs_hash(&env, 1),
            &0,
        );

        client.verify_report(&admin, &report_id, &1);

        let result = client.try_challenge_report(
            &challenger,
            &report_id,
            &make_ipfs_hash(&env, 2),
            &0,
        );
        assert_eq!(result, Err(Ok(OracleError::ReportAlreadyVerified)));
    }

    #[test]
    fn test_inactive_provider_submission() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let provider = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let contract_id = env.register(OracleConsumer, (admin.clone(),));
        let client = OracleConsumerClient::new(&env, &contract_id);

        client.register_provider(&admin, &provider, &Symbol::new(&env, "verra_vcs"), &0);
        client.remove_provider(&admin, &provider, &1);

        let result = client.try_submit_report(
            &provider,
            &project_id,
            &1000u64,
            &2000u64,
            &100_000i128,
            &Symbol::new(&env, "verra_vcs"),
            &make_ipfs_hash(&env, 1),
            &0,
        );
        assert_eq!(result, Err(Ok(OracleError::Unauthorized)));
    }

    #[test]
    fn test_resolve_challenge_to_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let provider = Address::generate(&env);
        let challenger = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let contract_id = env.register(OracleConsumer, (admin.clone(),));
        let client = OracleConsumerClient::new(&env, &contract_id);

        client.register_provider(&admin, &provider, &Symbol::new(&env, "verra_vcs"), &0);

        let report_id = client.submit_report(
            &provider,
            &project_id,
            &1000u64,
            &2000u64,
            &100_000i128,
            &Symbol::new(&env, "verra_vcs"),
            &make_ipfs_hash(&env, 1),
            &0,
        );

        client.challenge_report(
            &challenger,
            &report_id,
            &make_ipfs_hash(&env, 2),
            &0,
        );

        client.resolve_challenge(
            &admin,
            &report_id,
            &ReportStatus::Rejected,
            &1,
        );

        let report = client.get_report(&report_id);
        assert_eq!(report.status, ReportStatus::Rejected);
    }

    #[test]
    fn test_resolve_already_resolved_challenge() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let provider = Address::generate(&env);
        let challenger = Address::generate(&env);
        let project_id = create_project_id(&env, 1);

        let contract_id = env.register(OracleConsumer, (admin.clone(),));
        let client = OracleConsumerClient::new(&env, &contract_id);

        client.register_provider(&admin, &provider, &Symbol::new(&env, "verra_vcs"), &0);

        let report_id = client.submit_report(
            &provider,
            &project_id,
            &1000u64,
            &2000u64,
            &100_000i128,
            &Symbol::new(&env, "verra_vcs"),
            &make_ipfs_hash(&env, 1),
            &0,
        );

        client.challenge_report(
            &challenger,
            &report_id,
            &make_ipfs_hash(&env, 2),
            &0,
        );

        client.resolve_challenge(
            &admin,
            &report_id,
            &ReportStatus::Verified,
            &1,
        );

        let result = client.try_resolve_challenge(
            &admin,
            &report_id,
            &ReportStatus::Rejected,
            &2,
        );
        assert_eq!(result, Ok(Ok(())));
    }

    #[test]
    fn test_get_nonexistent_report() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);

        let contract_id = env.register(OracleConsumer, (admin.clone(),));
        let client = OracleConsumerClient::new(&env, &contract_id);

        let result = client.try_get_report(&999);
        assert_eq!(result, Err(Ok(OracleError::ReportNotFound)));
    }

    #[test]
    fn test_get_nonexistent_provider() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let stranger = Address::generate(&env);

        let contract_id = env.register(OracleConsumer, (admin.clone(),));
        let client = OracleConsumerClient::new(&env, &contract_id);

        let result = client.try_get_provider(&stranger);
        assert_eq!(result, Err(Ok(OracleError::ProviderNotFound)));
    }

    #[test]
    fn test_set_signature_threshold() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);

        let contract_id = env.register(OracleConsumer, (admin.clone(),));
        let client = OracleConsumerClient::new(&env, &contract_id);

        client.set_signature_threshold(&admin, &3u32, &0);
        client.set_signature_threshold(&admin, &5u32, &1);
    }

    #[test]
    fn test_query_empty_project_reports() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let project_id = create_project_id(&env, 42);

        let contract_id = env.register(OracleConsumer, (admin.clone(),));
        let client = OracleConsumerClient::new(&env, &contract_id);

        let reports = client.get_project_reports(&project_id);
        assert_eq!(reports.len(), 0);
    }
}
