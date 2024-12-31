use argon2::{
    Argon2,
    password_hash::{
        PasswordHash,
        PasswordHasher,
        PasswordVerifier,
        SaltString,
    },
};

static SALT: &str = "sinapis";

pub fn encryption(password: &str) -> String {
    Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::from_b64(SALT).unwrap())
        .unwrap()
        .to_string()
}

pub fn decryption(password: &[u8], password_hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(password_hash).unwrap();
    Argon2::default()
        .verify_password(password, &parsed_hash)
        .is_ok()
}
