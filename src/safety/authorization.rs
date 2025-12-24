use argon2::{
    password_hash::PasswordHasher,
    Argon2,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct VoidToken(String);

impl VoidToken {
    pub fn new_token() -> Self {
        let uuid = Uuid::new_v4().to_string();

        let argon2 = Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let hash = argon2
            .hash_password(uuid.as_bytes())
            .unwrap()
            .to_string();
        Self(hash)
    }
}



#[test]
fn test() {
    println!("{:?}", VoidToken::new_token());
}
