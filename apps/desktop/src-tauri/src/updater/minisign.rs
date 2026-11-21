//! Assinatura minisign dos pacotes de atualização (a primeira das duas; a segunda é a da
//! YubiKey, em `update_sig.rs`). O formato é o do minisign: a assinatura Ed25519 do pacote, ou
//! do BLAKE2b-512 dele (`ED`, o que o `tauri signer sign` grava), e a assinatura global, que
//! cobre o comentário confiável. É no comentário confiável que vem a versão assinada
//! (`version:X.Y.Z`): sem ela, ou com outra versão, o pacote não vale.
//!
//! As primitivas são as do `ring` (Ed25519) e do `blake2`, as mesmas que o resto do app já
//! usa; aqui fica só a leitura dos dois arquivos de texto do formato.

use base64::Engine;
use blake2::{Blake2b512, Digest};
use ring::signature::{ED25519, UnparsedPublicKey};

/// Chave pública minisign aceita, embutida na compilação (a privada fica fora do repositório).
const KEY: &str = include_str!("../../update-minisign.pub");

const B64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKey {
    id: [u8; 8],
    key: [u8; 32],
}

/// A chave embutida.
pub fn embedded() -> Result<PublicKey, String> {
    PublicKey::parse(KEY)
}

/// A linha de dados de um arquivo minisign (a que não é comentário).
fn data_line<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Option<&'a str> {
    lines.find(|l| !l.trim().is_empty() && !l.starts_with("untrusted comment:"))
}

impl PublicKey {
    /// O `.pub` do minisign: comentário e a linha base64 com `Ed`, o id e a chave.
    pub fn parse(text: &str) -> Result<Self, String> {
        let line = data_line(&mut text.lines()).ok_or("chave minisign vazia")?;
        let raw = B64
            .decode(line.trim())
            .map_err(|_| "chave minisign ilegível")?;
        if raw.len() != 42 || &raw[..2] != b"Ed" {
            return Err("chave minisign em formato desconhecido".into());
        }
        let mut id = [0u8; 8];
        let mut key = [0u8; 32];
        id.copy_from_slice(&raw[2..10]);
        key.copy_from_slice(&raw[10..]);
        Ok(Self { id, key })
    }

    fn verify_ed25519(&self, message: &[u8], signature: &[u8]) -> bool {
        UnparsedPublicKey::new(&ED25519, self.key)
            .verify(message, signature)
            .is_ok()
    }

    /// Confere a assinatura (o texto do `.sig`) de `data` e devolve o comentário confiável,
    /// que só vale depois daqui: é a assinatura global que garante que ele não foi trocado.
    pub fn verify(&self, signature: &str, data: &[u8]) -> Result<String, String> {
        let mut lines = signature.lines();
        let sig_line = data_line(&mut lines).ok_or("assinatura vazia")?;
        let trusted = lines
            .next()
            .and_then(|l| l.strip_prefix("trusted comment: "))
            .ok_or("assinatura sem comentário confiável")?;
        let global = lines.next().ok_or("assinatura sem a parte global")?;
        let raw = B64
            .decode(sig_line.trim())
            .map_err(|_| "assinatura ilegível")?;
        if raw.len() != 74 {
            return Err("assinatura em formato desconhecido".into());
        }
        let (alg, rest) = raw.split_at(2);
        let (id, sig) = rest.split_at(8);
        if id != self.id {
            return Err("assinada por outra chave".into());
        }
        let ok = match alg {
            b"ED" => self.verify_ed25519(&Blake2b512::digest(data), sig),
            b"Ed" => self.verify_ed25519(data, sig),
            _ => return Err("algoritmo de assinatura desconhecido".into()),
        };
        if !ok {
            return Err("assinatura do pacote não confere".into());
        }
        let global = B64
            .decode(global.trim())
            .map_err(|_| "assinatura global ilegível")?;
        let mut covered = sig.to_vec();
        covered.extend_from_slice(trusted.as_bytes());
        if !self.verify_ed25519(&covered, &global) {
            return Err("assinatura global não confere".into());
        }
        Ok(trusted.to_string())
    }
}

/// A versão assinada no comentário confiável (`timestamp:…\tfile:…\tversion:X.Y.Z`).
pub fn signed_version(trusted_comment: &str) -> Option<&str> {
    trusted_comment
        .split('\t')
        .find_map(|f| f.strip_prefix("version:"))
}

/// O campo `signature` do `latest.json` é o `.sig` inteiro em base64.
pub fn decode_field(field: &str) -> Result<String, String> {
    let raw = B64
        .decode(field.trim())
        .map_err(|_| "assinatura minisign ilegível no latest.json")?;
    String::from_utf8(raw).map_err(|_| "assinatura minisign ilegível no latest.json".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Chave e assinatura de teste, feitas com `tauri signer generate` e `tauri signer sign
    // --app-version 9.9.9` (uma chave descartável, só para este teste).
    const TEST_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEQxRUU5Mjk2MkMyNDgxNzUKUldSMWdTUXNscEx1MFVXTWtBRHc0L2tWY21xK1gzTEMvaWxhdFQ3Ykp5Wm5hc1lrSm9heUNVSTEK";
    const TEST_SIG: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVSMWdTUXNscEx1MFVVQkRwWmxDNVBSNUh2YVB5ODg0dmRhK2NPMldhbm1YSUx4ZW1NS1BKUTA3dkllUGxzTDl0ejdzWkd0T2pNb0E0bDZRYXVYY1hJaTNnVWJxcTZNN1FrPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzk1MTg2NjI0CWZpbGU6cGFjb3RlLmJpbgl2ZXJzaW9uOjkuOS45Clljd0FhZUxsTVpqVjk3dm5uN3BVQnQ0cWpBcENLZDRMT1M4NW9CMlpUMjkyZGxYR2lUT0JYU2ZlZ2ExYktjNXFFdnp3RVBzL3h4TGlwTDBuYjhMYkNRPT0K";
    const DATA: &[u8] = b"pacote de teste do Treasury\n";

    fn key() -> PublicKey {
        PublicKey::parse(&decode_field(TEST_KEY).unwrap()).unwrap()
    }

    #[test]
    fn accepts_what_the_tauri_cli_signs() {
        let sig = decode_field(TEST_SIG).unwrap();
        let trusted = key().verify(&sig, DATA).unwrap();
        assert_eq!(signed_version(&trusted), Some("9.9.9"));
    }

    #[test]
    fn rejects_other_bytes_other_keys_and_a_changed_comment() {
        let sig = decode_field(TEST_SIG).unwrap();
        assert!(key().verify(&sig, b"outro pacote").is_err());
        // A chave do app (outro id) não aceita a assinatura de teste.
        assert_eq!(
            embedded().unwrap().verify(&sig, DATA),
            Err("assinada por outra chave".into())
        );
        // Trocar a versão do comentário confiável quebra a assinatura global.
        let forged = sig.replace("version:9.9.9", "version:9.9.10");
        assert_eq!(
            key().verify(&forged, DATA),
            Err("assinatura global não confere".into())
        );
        assert!(key().verify("", DATA).is_err());
        assert!(key().verify("untrusted comment: x\nAAAA\n", DATA).is_err());
    }

    #[test]
    fn the_embedded_key_is_readable() {
        let k = embedded().unwrap();
        // EA2245E4D9C5F3C5, o id que o comentário da chave mostra (gravado ao contrário).
        assert_eq!(
            hex::encode_upper(k.id.iter().rev().copied().collect::<Vec<_>>()),
            "EA2245E4D9C5F3C5"
        );
    }

    #[test]
    fn reads_the_signed_version() {
        assert_eq!(
            signed_version("timestamp:1\tfile:a\tversion:1.2.3"),
            Some("1.2.3")
        );
        assert_eq!(signed_version("timestamp:1\tfile:a"), None);
    }
}
