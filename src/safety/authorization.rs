use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct VoidToken(String);

impl VoidToken {
    pub fn new_token() -> Self {
        let uuid = Uuid::new_v4().to_string();

        let salt = SaltString::generate(); // Note: needs the `getrandom` feature of `argon2` enabled

        let argon2 = Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let hash = argon2
            .hash_password(uuid.as_bytes(), &salt)
            .unwrap()
            .to_string();
        Self(hash)
    }
}



#[test]
fn test() {
    println!("{:?}", VoidToken::new_token());
}
