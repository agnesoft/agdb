use crate::error_code::ErrorCode;
use crate::server_error::ServerResult;
use ring::digest;
use ring::pbkdf2;
use ring::rand::SecureRandom;
use ring::rand::SystemRandom;
use serde::Deserialize;
use serde::Serialize;
use std::num::NonZeroU32;

pub(crate) const PASSWORD_LEN: usize = digest::SHA256_OUTPUT_LEN;
pub(crate) const SALT_LEN: usize = 16;
static ALGORITHM: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
static PEPPER: &[u8; SALT_LEN] = std::include_bytes!("../pepper");
static DB_SALT: [u8; SALT_LEN] = [
    198, 78, 119, 143, 114, 32, 22, 184, 167, 93, 196, 63, 154, 18, 14, 79,
];

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Password {
    pub(crate) username: String,
    pub(crate) password: [u8; PASSWORD_LEN],
    pub(crate) user_salt: [u8; SALT_LEN],
}

impl Password {
    pub(crate) fn create(user: &str, pswd: &str) -> Self {
        let rng = SystemRandom::new();
        let mut user_salt = [0_u8; SALT_LEN];
        let _ = rng.fill(&mut user_salt);
        let password = Self::password(user, user_salt, pswd);

        Self {
            username: user.to_string(),
            user_salt,
            password,
        }
    }

    pub(crate) fn new(user: &str, pswd: &[u8], salt: &[u8]) -> ServerResult<Self> {
        Ok(Self {
            username: user.to_string(),
            password: pswd.try_into()?,
            user_salt: salt.try_into()?,
        })
    }

    pub(crate) fn verify_password(&self, attempted_password: &str) -> bool {
        let salt = Self::salt(&self.username, self.user_salt);

        pbkdf2::verify(
            ALGORITHM,
            NonZeroU32::new(123_456).unwrap(),
            &salt,
            attempted_password.as_bytes(),
            &self.password,
        )
        .is_ok()
    }

    fn password(user: &str, user_salt: [u8; SALT_LEN], pswd: &str) -> [u8; PASSWORD_LEN] {
        let salt = Self::salt(user, user_salt);
        let mut out_pswd = [0u8; PASSWORD_LEN];

        pbkdf2::derive(
            ALGORITHM,
            NonZeroU32::new(123_456).unwrap(),
            &salt,
            pswd.as_bytes(),
            &mut out_pswd,
        );

        out_pswd
    }

    fn salt(user: &str, user_salt: [u8; SALT_LEN]) -> Vec<u8> {
        let mut salt = Vec::with_capacity(
            user.as_bytes().len() + user_salt.len() + DB_SALT.len() + PEPPER.len(),
        );

        salt.extend(DB_SALT);
        salt.extend(user.as_bytes());
        salt.extend(PEPPER);
        salt.extend(user_salt);

        salt
    }
}

pub(crate) fn validate_password(password: &str) -> ServerResult {
    if password.len() < 8 {
        Err(ErrorCode::PasswordTooShort.into())
    } else {
        Ok(())
    }
}

pub(crate) fn validate_username(name: &str) -> ServerResult {
    if name.len() < 3 {
        Err(ErrorCode::NameTooShort.into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_password() -> ServerResult {
        let password = Password::create("alice", "MyPassword123");
        assert_ne!(password.password, [0_u8; PASSWORD_LEN]);
        assert_ne!(password.user_salt, [0_u8; SALT_LEN]);

        assert!(password.verify_password("MyPassword123"));
        assert!(!password.verify_password("MyPassword"));

        let other = Password::new(&password.username, &password.password, &password.user_salt)?;

        assert!(other.verify_password("MyPassword123"));
        assert!(!other.verify_password("MyPassword"));
        Ok(())
    }
}
