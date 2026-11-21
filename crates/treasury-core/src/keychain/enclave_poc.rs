//! Prova de conceito do caminho B do item 5 de docs/local-security.md: a chave do banco
//! embrulhada por uma chave P-256 que só existe dentro da Secure Enclave, sem item no
//! Keychain, sem entitlements e sem Developer ID.
//!
//! Resultado (2026-10-23, macOS 27.0.1, Apple Silicon), igual no binário só com a assinatura
//! do linker (dev) e no ad hoc com hardened runtime (release):
//! - a chave nasce na Secure Enclave (`kSecAttrIsPermanent = false`) e o sistema devolve o
//!   blob dela (`toid`, ~324 bytes, cifrado pela própria Secure Enclave);
//! - o embrulho ECIES (cofator, X9.63 SHA-256, AES-GCM) de 32 bytes tem 113 bytes;
//! - `SecKeyCreateWithData` recria a mesma chave a partir do blob **desde que o `toid` também
//!   vá nos parâmetros**; sem ele, o sistema cria outra chave e o embrulho não abre;
//! - outro binário, com outra assinatura, recria a chave e abre o embrulho: o blob não fica
//!   preso ao app. A proteção vem só do controle de acesso (`biometryCurrentSet` ou
//!   `userPresence`), que a própria Secure Enclave cobra a cada uso.
//!
//! Por isso os testes ficam fora da suíte (`--ignored`): precisam da Secure Enclave e o de
//! biometria abre o Touch ID.

use core_foundation::base::{CFType, TCFType};
use core_foundation::data::CFData;
use core_foundation::dictionary::CFMutableDictionary;
use core_foundation::string::CFString;
use security_framework::access_control::{ProtectionMode, SecAccessControl};
use security_framework::key::{Algorithm, GenerateKeyOptions, KeyType, SecKey, Token};
use security_framework_sys::access_control::{
    kSecAccessControlBiometryCurrentSet, kSecAccessControlPrivateKeyUsage,
};
use security_framework_sys::item::{
    kSecAttrKeyClass, kSecAttrKeyClassPrivate, kSecAttrKeyType, kSecAttrKeyTypeECSECPrimeRandom,
    kSecAttrTokenID, kSecAttrTokenIDSecureEnclave,
};

const ALG: Algorithm = Algorithm::ECIESEncryptionCofactorVariableIVX963SHA256AESGCM;

/// Chave nova na Secure Enclave e o blob para guardar em `~/.treasury`.
fn create(biometry: bool) -> (SecKey, Vec<u8>) {
    let flags = kSecAccessControlPrivateKeyUsage
        | if biometry {
            kSecAccessControlBiometryCurrentSet
        } else {
            0
        };
    let ac = SecAccessControl::create_with_protection(
        Some(ProtectionMode::AccessibleWhenUnlockedThisDeviceOnly),
        flags,
    )
    .unwrap();
    let mut o = GenerateKeyOptions::default();
    o.set_key_type(KeyType::ec_sec_prime_random())
        .set_size_in_bits(256)
        .set_token(Token::SecureEnclave)
        .set_access_control(ac);
    let key = SecKey::new(&o).unwrap();
    let attrs = key.attributes();
    let toid_key = CFString::from_static_string("toid");
    let raw = attrs.find(toid_key.as_concrete_TypeRef().cast()).unwrap();
    // SAFETY: o valor de `toid` é um CFData do dicionário, que segue vivo aqui.
    let blob = unsafe { CFData::wrap_under_get_rule((*raw).cast()) };
    (key, blob.bytes().to_vec())
}

/// Recria a chave pelo blob; o `toid` precisa ir também nos parâmetros.
fn restore(blob: &[u8]) -> SecKey {
    let data = CFData::from_buffer(blob);
    let mut params = CFMutableDictionary::<CFType, CFType>::new();
    // SAFETY: constantes do Security; o dicionário e o dado vivem até o fim da função.
    unsafe {
        let s = |c| CFString::wrap_under_get_rule(c).as_CFType();
        params.set(s(kSecAttrTokenID), s(kSecAttrTokenIDSecureEnclave));
        params.set(s(kSecAttrKeyType), s(kSecAttrKeyTypeECSECPrimeRandom));
        params.set(s(kSecAttrKeyClass), s(kSecAttrKeyClassPrivate));
        params.set(
            CFString::from_static_string("toid").as_CFType(),
            data.as_CFType(),
        );
        let mut err = std::ptr::null_mut();
        let k = security_framework_sys::key::SecKeyCreateWithData(
            data.as_concrete_TypeRef(),
            params.as_concrete_TypeRef().cast(),
            &mut err,
        );
        assert!(!k.is_null(), "SecKeyCreateWithData falhou");
        SecKey::wrap_under_create_rule(k)
    }
}

fn round_trip(biometry: bool) {
    let db_key = [0x5au8; 32];
    let (key, blob) = create(biometry);
    let public = key.public_key().unwrap();
    let wrapped = public.encrypt_data(ALG, &db_key).unwrap();
    drop(key);
    let again = restore(&blob);
    assert_eq!(
        again.public_key().unwrap().external_representation(),
        public.external_representation(),
        "a chave recriada é outra"
    );
    assert_eq!(again.decrypt_data(ALG, &wrapped).unwrap(), db_key);
}

#[test]
#[ignore = "precisa da Secure Enclave (cargo test enclave_poc -- --ignored)"]
fn wraps_and_unwraps_from_the_blob() {
    round_trip(false);
}

#[test]
#[ignore = "abre o Touch ID a cada uso da chave"]
fn wraps_and_unwraps_with_touch_id() {
    round_trip(true);
}
