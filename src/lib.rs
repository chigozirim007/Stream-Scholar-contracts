#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, Address, Env, token};

#[contracttype]
#[derive(Clone)]
pub struct Access {
    pub student: Address,
    pub course_id: u64,
    pub expiry_time: u64,
    pub token: Address,
}

#[contracttype]
pub enum DataKey {
    Access(Address, u64),
    Price,
}

#[contract]
pub struct ScholarContract;

#[contractimpl]
impl ScholarContract {
    pub fn init(env: Env, rate: i128) {
        env.storage().instance().set(&DataKey::Price, &rate);
    }

    pub fn buy_access(env: Env, student: Address, course_id: u64, amount: i128, token: Address) {
        student.require_auth();

        let client = token::Client::new(&env, &token);
        client.transfer(&student, &env.current_contract_address(), &amount);

        let rate: i128 = env.storage().instance().get(&DataKey::Price).unwrap_or(1); 
        let seconds_bought = (amount / rate) as u64;
        let current_time = env.ledger().timestamp();

        let mut access = env.storage().instance().get(&DataKey::Access(student.clone(), course_id))
            .unwrap_or(Access {
                student: student.clone(),
                course_id,
                expiry_time: current_time,
                token,
            });

        if access.expiry_time > current_time {
            access.expiry_time += seconds_bought;
        } else {
            access.expiry_time = current_time + seconds_bought;
        }

        env.storage().instance().set(&DataKey::Access(student, course_id), &access);
    }

    pub fn has_access(env: Env, student: Address, course_id: u64) -> bool {
        let access: Access = env.storage().instance().get(&DataKey::Access(student, course_id))
            .unwrap_or(Access {
                student: student.clone(),
                course_id,
                expiry_time: 0,
                token: student.clone(),
            });
            
        env.ledger().timestamp() < access.expiry_time
    }
}
