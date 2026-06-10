#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, BytesN, Env, Symbol, Vec};
use nbbs_shared::{CreditError, CreditType};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Retirement(u64),
    RetirementCount,
    HolderRetirements(Address),
    RetiredCredits(Address),
    Nonce(Address),
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct RetirementRecord {
    pub id: u64,
    pub holder: Address,
    pub bond_id: u64,
    pub amount: i128,
    pub credit_type: CreditType,
    pub retired_at: u64,
    pub certificate_ipfs_hash: BytesN<32>,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct RetirementCertificate {
    pub record_id: u64,
    pub holder: Address,
    pub bond_id: u64,
    pub amount: i128,
    pub credit_type: CreditType,
    pub retired_at: u64,
    pub certificate_hash: BytesN<32>,
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
pub struct CreditRetirement;

#[contractimpl]
impl CreditRetirement {
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn retire_credits(
        env: Env,
        holder: Address,
        bond_id: u64,
        amount: i128,
        credit_type: CreditType,
        certificate_hash: BytesN<32>,
        nonce: u64,
    ) -> Result<u64, CreditError> {
        holder.require_auth();

        let expected_nonce = get_nonce(&env, &holder);
        if nonce != expected_nonce {
            return Err(CreditError::InvalidNonce);
        }
        set_nonce(&env, &holder, expected_nonce + 1);

        if amount <= 0 {
            return Err(CreditError::InsufficientCredits);
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::RetirementCount)
            .unwrap_or(0);
        let retirement_id = count + 1;
        env.storage()
            .instance()
            .set(&DataKey::RetirementCount, &retirement_id);

        let now = env.ledger().timestamp();
        let record = RetirementRecord {
            id: retirement_id,
            holder: holder.clone(),
            bond_id,
            amount,
            credit_type,
            retired_at: now,
            certificate_ipfs_hash: certificate_hash.clone(),
        };

        env.storage()
            .instance()
            .set(&DataKey::Retirement(retirement_id), &record);

        let retired: i128 = env
            .storage()
            .instance()
            .get(&DataKey::RetiredCredits(holder.clone()))
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::RetiredCredits(holder.clone()), &(retired + amount));

        let mut retirements: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::HolderRetirements(holder.clone()))
            .unwrap_or(vec![&env]);
        retirements.push_back(retirement_id);
        env.storage()
            .instance()
            .set(&DataKey::HolderRetirements(holder.clone()), &retirements);

        env.events().publish(
            (Symbol::new(&env, "CreditsRetired"),),
            (holder.clone(), amount, credit_type),
        );

        Ok(retirement_id)
    }

    pub fn get_retirement_record(
        env: Env,
        retirement_id: u64,
    ) -> Result<RetirementRecord, CreditError> {
        env.storage()
            .instance()
            .get(&DataKey::Retirement(retirement_id))
            .ok_or(CreditError::InsufficientCredits)
    }

    pub fn get_retirement_certificate(
        env: Env,
        retirement_id: u64,
    ) -> Result<RetirementCertificate, CreditError> {
        let record: RetirementRecord = env
            .storage()
            .instance()
            .get(&DataKey::Retirement(retirement_id))
            .ok_or(CreditError::InsufficientCredits)?;

        Ok(RetirementCertificate {
            record_id: record.id,
            holder: record.holder,
            bond_id: record.bond_id,
            amount: record.amount,
            credit_type: record.credit_type,
            retired_at: record.retired_at,
            certificate_hash: record.certificate_ipfs_hash,
        })
    }

    pub fn get_holder_retirements(env: Env, holder: Address) -> Vec<u64> {
        env.storage()
            .instance()
            .get(&DataKey::HolderRetirements(holder))
            .unwrap_or(vec![&env])
    }

    pub fn get_total_retired(env: Env, holder: Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::RetiredCredits(holder))
            .unwrap_or(0)
    }

    pub fn total_retirements(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::RetirementCount)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn make_certificate_hash(env: &Env, value: u8) -> BytesN<32> {
        let mut arr = [0u8; 32];
        arr[0] = value;
        BytesN::from_array(env, &arr)
    }

    #[test]
    fn test_retire_credits_and_query() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let holder = Address::generate(&env);

        let contract_id = env.register(CreditRetirement, (admin,));
        let client = CreditRetirementClient::new(&env, &contract_id);

        let hash = make_certificate_hash(&env, 1);
        let id = client.retire_credits(
            &holder,
            &1,
            &1000i128,
            &CreditType::Carbon,
            &hash,
            &0,
        );
        assert_eq!(id, 1);

        let record = client.get_retirement_record(&id);
        assert_eq!(record.holder, holder);
        assert_eq!(record.bond_id, 1);
        assert_eq!(record.amount, 1000);
        assert_eq!(record.credit_type, CreditType::Carbon);
        assert_eq!(record.certificate_ipfs_hash, hash);

        let cert = client.get_retirement_certificate(&id);
        assert_eq!(cert.record_id, id);
        assert_eq!(cert.holder, holder);
        assert_eq!(cert.amount, 1000);
        assert_eq!(cert.certificate_hash, hash);

        assert_eq!(client.total_retirements(), 1);
    }

    #[test]
    fn test_multiple_retirements_and_total() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let holder = Address::generate(&env);

        let contract_id = env.register(CreditRetirement, (admin,));
        let client = CreditRetirementClient::new(&env, &contract_id);

        let hash1 = make_certificate_hash(&env, 1);
        let id1 = client.retire_credits(
            &holder,
            &1,
            &500i128,
            &CreditType::Carbon,
            &hash1,
            &0,
        );

        let hash2 = make_certificate_hash(&env, 2);
        let id2 = client.retire_credits(
            &holder,
            &1,
            &300i128,
            &CreditType::Biodiversity,
            &hash2,
            &1,
        );

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(client.total_retirements(), 2);

        assert_eq!(client.get_total_retired(&holder), 800);

        let retirements = client.get_holder_retirements(&holder);
        assert_eq!(retirements.len(), 2);
    }

    #[test]
    fn test_retire_zero_credits_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let holder = Address::generate(&env);

        let contract_id = env.register(CreditRetirement, (admin,));
        let client = CreditRetirementClient::new(&env, &contract_id);

        let hash = make_certificate_hash(&env, 1);
        let result = client.try_retire_credits(
            &holder,
            &1,
            &0i128,
            &CreditType::Carbon,
            &hash,
            &0,
        );
        assert_eq!(result, Err(Ok(CreditError::InsufficientCredits)));
    }

    #[test]
    fn test_query_nonexistent_retirement() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);

        let contract_id = env.register(CreditRetirement, (admin,));
        let client = CreditRetirementClient::new(&env, &contract_id);

        let result = client.try_get_retirement_record(&999);
        assert_eq!(result, Err(Ok(CreditError::InsufficientCredits)));
    }

    #[test]
    fn test_multiple_holders_tracked_independently() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let holder1 = Address::generate(&env);
        let holder2 = Address::generate(&env);

        let contract_id = env.register(CreditRetirement, (admin,));
        let client = CreditRetirementClient::new(&env, &contract_id);

        let hash = make_certificate_hash(&env, 1);
        client.retire_credits(&holder1, &1, &1000i128, &CreditType::Carbon, &hash, &0);

        let hash2 = make_certificate_hash(&env, 2);
        client.retire_credits(&holder2, &1, &2000i128, &CreditType::Biodiversity, &hash2, &0);

        assert_eq!(client.get_total_retired(&holder1), 1000);
        assert_eq!(client.get_total_retired(&holder2), 2000);
        assert_eq!(client.total_retirements(), 2);

        assert_eq!(client.get_holder_retirements(&holder1).len(), 1);
        assert_eq!(client.get_holder_retirements(&holder2).len(), 1);
    }

    #[test]
    fn test_nonexistent_certificate() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);

        let contract_id = env.register(CreditRetirement, (admin,));
        let client = CreditRetirementClient::new(&env, &contract_id);

        let result = client.try_get_retirement_certificate(&999);
        assert_eq!(result, Err(Ok(CreditError::InsufficientCredits)));
    }

    #[test]
    fn test_empty_total_retirements() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);

        let contract_id = env.register(CreditRetirement, (admin.clone(),));
        let client = CreditRetirementClient::new(&env, &contract_id);

        assert_eq!(client.total_retirements(), 0);
        let empty: Vec<u64> = vec![&env];
        assert_eq!(client.get_holder_retirements(&admin), empty);
        assert_eq!(client.get_total_retired(&admin), 0);
    }

    #[test]
    fn test_invalid_nonce_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let holder = Address::generate(&env);

        let contract_id = env.register(CreditRetirement, (admin,));
        let client = CreditRetirementClient::new(&env, &contract_id);

        let hash = make_certificate_hash(&env, 1);
        let result = client.try_retire_credits(
            &holder,
            &1,
            &1000i128,
            &CreditType::Carbon,
            &hash,
            &1,
        );
        assert_eq!(result, Err(Ok(CreditError::InvalidNonce)));
    }
}
