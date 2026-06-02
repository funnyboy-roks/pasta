use std::{borrow::Cow, num::NonZeroU32};

use aes_gcm::{
    AeadInOut, Aes256Gcm, Key, KeyInit,
    aead::{Aead, Nonce},
};
use anyhow::Context;
use base64::{DecodeError, DecodeSliceError, encoded_len, prelude::*};
use redact::{Secret, expose_secret};
use ring::pbkdf2::PBKDF2_HMAC_SHA256;
use serde::{Deserialize, Serialize};

pub const ENCRYPTED_TYPE: &str = "application/aes256gcm-encrypted";

fn b64_decode(e: &str) -> Result<Vec<u8>, DecodeError> {
    BASE64_STANDARD.decode(e)
}

fn b64_decode_len<const N: usize>(e: &str) -> Result<[u8; N], DecodeSliceError> {
    let mut buf = [0u8; N];
    let n = BASE64_STANDARD.decode_slice(e, &mut buf)?;
    assert_eq!(n, N);
    Ok(buf)
}

fn b64_encode(buf: &[u8], out: &mut String) {
    BASE64_STANDARD.encode_string(buf, out)
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EncryptedContent<'a> {
    #[serde(serialize_with = "expose_secret")]
    pub content_type: Option<Secret<Cow<'a, str>>>,
    #[serde(serialize_with = "expose_secret")]
    pub content: Secret<Cow<'a, str>>,
}

pub fn encrypt(content: EncryptedContent<'_>, password: Secret<&str>) -> String {
    let salt: [u8; 16] = rand::random();
    let key = get_key(password, salt);
    let iv: [u8; 12] = rand::random();

    let aes = Aes256Gcm::new(&key);
    let iv = Nonce::<Aes256Gcm>::from(iv);
    let content = serde_json::to_string(&content).expect("The structure is always serialisable.");
    let cipher = aes.encrypt(&iv, content.as_bytes()).unwrap();

    let mut out = String::with_capacity(
        24 /* salt */ + ':'.len_utf8() + 16 /* iv */ + ':'.len_utf8() + encoded_len(cipher.len(), true).expect("cipher.len() < usize::MAX * .75"),
    );

    b64_encode(&salt, &mut out);
    out.push(':');
    b64_encode(&iv, &mut out);
    out.push(':');
    b64_encode(&cipher, &mut out);

    out
}

pub fn decrypt(
    encoded: &str,
    password: Secret<&str>,
) -> anyhow::Result<Option<EncryptedContent<'static>>> {
    let (salt, rest) = encoded.split_once(':').context("Invalid content format")?;
    let (iv, cipher) = rest.split_once(':').context("Invalid content format")?;

    let salt = b64_decode_len::<16>(salt).context("Improperly formatted salt string")?;
    let iv = b64_decode_len::<12>(iv).context("Improperly formatted iv string")?;
    let mut cipher = b64_decode(cipher).context("Improperly formatted cipher string")?;

    let key = get_key(password, salt);
    let aes = Aes256Gcm::new(&key);
    let nonce = Nonce::<Aes256Gcm>::from(iv);

    match aes.decrypt_in_place(&nonce, &[], &mut cipher) {
        Ok(()) => {
            let data: EncryptedContent<'static> =
                serde_json::from_slice(&cipher).context("parsing encrypted body")?;
            Ok(Some(data))
        }
        Err(_) => Ok(None),
    }
}

fn get_key(password: Secret<&str>, salt: [u8; 16]) -> Key<Aes256Gcm> {
    let mut out = [0; 32];

    ring::pbkdf2::derive(
        PBKDF2_HMAC_SHA256,
        NonZeroU32::new(100_000).unwrap(),
        &salt,
        password.expose_secret().as_bytes(),
        &mut out,
    );

    Key::<Aes256Gcm>::from(out)
}

#[cfg(test)]
mod test {
    use crate::crypto::{decrypt, encrypt};
    use base64::prelude::*;
    use redact::Secret;

    use super::EncryptedContent;

    // content encoded from the web side
    #[test]
    fn from_web() {
        let password = "password";
        let encrypted = "MKW8K63HT9e2YUixkCR7RA==:HeznX7lggJYjAY9l:/NbVBddqxtnpke4cJAhkzrsTgKVE/Ll4txamNugIq5tSx9j8dk+Qj9rvN2SBzRAYRSNuiPaaG27kkOQjrkz/75DWpdUWy+tb2sp8mmXm6eZJ7T2Yjz9ZmVzePG+uDbIwB+xueD16K1QKEz1FfswDWI5oTd5eBVpVBNuJTiKzioOp";
        let decrypted = decrypt(encrypted, password.into()).unwrap().unwrap();

        let content = BASE64_STANDARD
            .decode(&**decrypted.content.expose_secret())
            .unwrap();
        let content = String::from_utf8(content).unwrap();

        assert_eq!(
            decrypted.content_type.unwrap().expose_secret(),
            "text/plain; charset=utf-8"
        );
        assert_eq!(content, "This is a formerly-encrypted message! :D");
    }

    #[test]
    fn roundtrip() {
        let password = "my_password_123!";
        let content = "some very secret string that nobody is allowed to know (except me)";

        let original = EncryptedContent {
            content_type: Some(Secret::new("text/plain".into())),
            content: Secret::new(content.into()),
        };

        let encrypted = encrypt(original.clone(), password.into());

        let decrypted = decrypt(&encrypted, password.into()).unwrap().unwrap();

        assert_eq!(original, decrypted);
    }

    #[test]
    fn roundtrip_invalid_password() {
        let password = "my_password_123!";
        let content = "some very secret string that nobody is allowed to know (except me)";

        let original = EncryptedContent {
            content_type: Some(Secret::new("text/plain".into())),
            content: Secret::new(content.into()),
        };

        let encrypted = encrypt(original, password.into());

        let decrypted = decrypt(&encrypted, "my_password_124!".into()).unwrap();

        assert_eq!(decrypted, None);
    }
}
