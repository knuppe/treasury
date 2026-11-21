//! Segunda assinatura das atualizações, feita na YubiKey (docs/auto-update.md).
//!
//! A assinatura minisign (`updater/minisign.rs`) usa uma chave guardada em arquivo. Por
//! cima dela, o pacote precisa de uma assinatura SSH (`ssh-keygen -Y sign`, formato SSHSIG) de
//! uma chave FIDO2 que só existe dentro da YubiKey: cada assinatura pede o toque e o PIN na
//! chave física. Mesmo com a chave minisign vazada, nenhuma atualização entra sem a YubiKey.
//!
//! Dois tipos de chave são aceitos, sempre de hardware: `sk-ssh-ed25519` e
//! `sk-ecdsa-sha2-nistp256` (ECDSA P-256). Chave comum, em arquivo, nunca.
//!
//! Uma chave P-256 pode ser a do Secure Enclave do Mac do mantenedor, criada pelo macOS como
//! não exportável e com a digital exigida em cada assinatura (`sc_auth create-ctk-identity
//! -k p-256-ne -t bio`). O macOS não marca "usuário verificado" na assinatura, e uma chave
//! criada sem a digital assina igual: quem exige a digital é o chip, pela forma como a chave
//! foi criada, e o app não tem como conferir. Por isso essa chave é declarada com a palavra
//! `secure-enclave` no começo da linha em `update-keys.pub`; só para ela vale a marca de
//! presença sem a de verificação. Os scripts de release conferem a proteção `bio` da chave
//! antes de assinar (docs/auto-update.md).
//!
//! As chaves públicas aceitas ficam em `update-keys.pub` (uma por YubiKey, no formato do
//! `.pub` do OpenSSH). Com o arquivo sem chaves, o app recusa toda atualização
//! (`updater::hardware_check`).

use base64::Engine;
use sha2::{Digest, Sha256, Sha512};

/// Chaves aceitas, embutidas na compilação.
const KEYS: &str = include_str!("../update-keys.pub");
/// Namespace do `ssh-keygen -Y sign -n`, com a versão: a assinatura de um pacote não vale
/// para outra coisa nem para outra versão. Sem a versão, quem tivesse só a chave minisign
/// poderia republicar um pacote antigo, com a assinatura da YubiKey dele, como versão nova.
pub fn namespace(version: &str) -> String {
    format!("treasury-update@{version}")
}
const SK_ED25519: &str = "sk-ssh-ed25519@openssh.com";
const SK_ECDSA_P256: &str = "sk-ecdsa-sha2-nistp256@openssh.com";
const NISTP256: &str = "nistp256";
/// Palavra que declara, em `update-keys.pub`, uma chave do Secure Enclave do Mac.
const SECURE_ENCLAVE: &str = "secure-enclave";
const MAGIC: &[u8] = b"SSHSIG";
/// Flags do autenticador: presença (toque) e verificação do usuário (PIN).
const USER_PRESENT: u8 = 0x01;
const USER_VERIFIED: u8 = 0x04;

/// Tipo da chave FIDO2: o nome que o OpenSSH dá na chave e na assinatura.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Ed25519,
    EcdsaP256,
}

impl Kind {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            SK_ED25519 => Some(Self::Ed25519),
            SK_ECDSA_P256 => Some(Self::EcdsaP256),
            _ => None,
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Ed25519 => SK_ED25519,
            Self::EcdsaP256 => SK_ECDSA_P256,
        }
    }
}

/// O que a assinatura precisa provar, conforme onde a chave mora.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Policy {
    /// YubiKey: toque e PIN marcados na assinatura pelo próprio aparelho.
    TouchAndPin,
    /// Secure Enclave do Mac: a digital é exigida pelo chip e não aparece na assinatura.
    SecureEnclave,
}

/// Chave pública de hardware (YubiKey ou Secure Enclave do Mac).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareKey {
    /// Blob da chave pública como o OpenSSH grava (tipo, chave e `application`).
    blob: Vec<u8>,
    kind: Kind,
    policy: Policy,
    /// Ed25519: os 32 bytes da chave. ECDSA: o ponto P-256 sem compressão (65 bytes, 0x04 ‖ X ‖ Y).
    public: Vec<u8>,
    application: String,
    pub comment: String,
}

/// Leitor de campos SSH (`uint32` + bytes).
struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn bytes(&mut self, n: usize) -> Result<&'a [u8], String> {
        if self.0.len() < n {
            return Err("assinatura truncada".into());
        }
        let (head, rest) = self.0.split_at(n);
        self.0 = rest;
        Ok(head)
    }
    fn u32(&mut self) -> Result<u32, String> {
        let b = self.bytes(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.bytes(1)?[0])
    }
    fn string(&mut self) -> Result<&'a [u8], String> {
        let n = self.u32()? as usize;
        self.bytes(n)
    }
    fn text(&mut self) -> Result<&'a str, String> {
        std::str::from_utf8(self.string()?).map_err(|_| "texto inválido na assinatura".into())
    }
    fn done(&self) -> bool {
        self.0.is_empty()
    }
}

fn put_string(out: &mut Vec<u8>, s: &[u8]) {
    out.extend_from_slice(&(s.len() as u32).to_be_bytes());
    out.extend_from_slice(s);
}

fn parse_blob(blob: &[u8]) -> Result<(Kind, Vec<u8>, String), String> {
    let mut r = Reader(blob);
    let kind = Kind::from_name(r.text()?).ok_or("a chave não é uma chave FIDO2 aceita")?;
    let public = match kind {
        Kind::Ed25519 => {
            let pk = r.string()?;
            if pk.len() != 32 {
                return Err("chave pública com tamanho errado".into());
            }
            pk.to_vec()
        }
        Kind::EcdsaP256 => {
            if r.text()? != NISTP256 {
                return Err("chave ECDSA de uma curva não aceita".into());
            }
            let point = r.string()?;
            if point.len() != 65 || point[0] != 0x04 {
                return Err("chave pública com tamanho errado".into());
            }
            point.to_vec()
        }
    };
    let application = r.text()?.to_string();
    if !r.done() {
        return Err("chave pública com dados a mais".into());
    }
    Ok((kind, public, application))
}

/// Inteiro SSH (`mpint`) positivo como 32 bytes, para o `r` e o `s` de uma assinatura P-256.
fn mpint_32(mut n: &[u8]) -> Result<[u8; 32], String> {
    if n.first().is_some_and(|b| b & 0x80 != 0) {
        return Err("assinatura ECDSA com número negativo".into());
    }
    while n.len() > 1 && n[0] == 0 {
        n = &n[1..];
    }
    if n.is_empty() || n.len() > 32 {
        return Err("assinatura ECDSA com tamanho errado".into());
    }
    let mut out = [0u8; 32];
    out[32 - n.len()..].copy_from_slice(n);
    Ok(out)
}

/// Lê as chaves no formato do `.pub` do OpenSSH, ignorando linhas vazias e comentários. A
/// linha de uma chave do Secure Enclave do Mac começa com `secure-enclave `.
pub fn parse_keys(text: &str) -> Result<Vec<HardwareKey>, String> {
    let b64 = base64::engine::general_purpose::STANDARD;
    let mut keys = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (policy, line) = match line.strip_prefix(SECURE_ENCLAVE) {
            Some(rest) if rest.starts_with(' ') => (Policy::SecureEnclave, rest.trim_start()),
            _ => (Policy::TouchAndPin, line),
        };
        let mut parts = line.splitn(3, ' ');
        let (kind, data) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
        if Kind::from_name(kind).is_none() {
            return Err(format!("chave de atualização com tipo não aceito: {kind}"));
        }
        let blob = b64
            .decode(data)
            .map_err(|e| format!("chave de atualização: {e}"))?;
        let (blob_kind, public, application) = parse_blob(&blob)?;
        if blob_kind.name() != kind {
            return Err("chave de atualização com tipo diferente do conteúdo".into());
        }
        // O Secure Enclave só faz P-256: a exceção não vale para outro tipo de chave.
        if policy == Policy::SecureEnclave && blob_kind != Kind::EcdsaP256 {
            return Err("chave secure-enclave tem de ser sk-ecdsa-sha2-nistp256".into());
        }
        keys.push(HardwareKey {
            blob,
            kind: blob_kind,
            policy,
            public,
            application,
            comment: parts.next().unwrap_or("").trim().to_string(),
        });
    }
    Ok(keys)
}

/// Chaves embutidas neste app. Arquivo inválido é erro, nunca "sem chaves": a atualização é
/// recusada em vez de passar sem a YubiKey.
pub fn embedded() -> Result<Vec<HardwareKey>, String> {
    parse_keys(KEYS)
}

/// Confere a assinatura SSHSIG (texto `-----BEGIN SSH SIGNATURE-----`) de `data` com uma das
/// chaves, feita para `namespace` (veja [`namespace`]). Devolve o comentário da chave.
pub fn verify(
    keys: &[HardwareKey],
    armored: &str,
    data: &[u8],
    expected_namespace: &str,
) -> Result<String, String> {
    let b64 = base64::engine::general_purpose::STANDARD;
    let body: String = armored
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("-----"))
        .collect();
    let raw = b64
        .decode(body)
        .map_err(|_| "assinatura da YubiKey ilegível".to_string())?;
    let mut r = Reader(&raw);
    if r.bytes(MAGIC.len())? != MAGIC || r.u32()? != 1 {
        return Err("não é uma assinatura SSH (SSHSIG v1)".into());
    }
    let key_blob = r.string()?;
    let namespace = r.string()?;
    let reserved = r.string()?;
    let hash_alg = r.text()?;
    let sig_blob = r.string()?;
    if !r.done() {
        return Err("assinatura com dados a mais".into());
    }
    if namespace != expected_namespace.as_bytes() {
        return Err("assinatura feita para outro uso ou outra versão (namespace)".into());
    }
    let key = keys
        .iter()
        .find(|k| k.blob == key_blob)
        .ok_or("assinada por uma chave que este app não conhece")?;
    let digest = match hash_alg {
        "sha512" => Sha512::digest(data).to_vec(),
        "sha256" => Sha256::digest(data).to_vec(),
        other => return Err(format!("hash não aceito: {other}")),
    };
    // O que o OpenSSH assina: preâmbulo, namespace, reservado, algoritmo e o hash do arquivo.
    let mut signed = MAGIC.to_vec();
    put_string(&mut signed, namespace);
    put_string(&mut signed, reserved);
    put_string(&mut signed, hash_alg.as_bytes());
    put_string(&mut signed, &digest);

    let mut s = Reader(sig_blob);
    // O tipo da assinatura tem de ser o da chave que a fez.
    if s.text()? != key.kind.name() {
        return Err("a assinatura não veio de uma chave FIDO2".into());
    }
    let sig = s.string()?;
    let flags = s.u8()?;
    let counter = s.u32()?;
    if !s.done() {
        return Err("assinatura com dados a mais".into());
    }
    if flags & USER_PRESENT == 0 {
        return Err("a assinatura não teve a presença de quem assina".into());
    }
    if key.policy == Policy::TouchAndPin && flags & USER_VERIFIED == 0 {
        return Err("a assinatura não teve o toque e o PIN na YubiKey".into());
    }
    // Chaves FIDO2 assinam hash(application) ‖ flags ‖ contador ‖ hash(mensagem).
    let mut msg = Sha256::digest(key.application.as_bytes()).to_vec();
    msg.push(flags);
    msg.extend_from_slice(&counter.to_be_bytes());
    msg.extend_from_slice(&Sha256::digest(&signed));
    let checked = match key.kind {
        Kind::Ed25519 => {
            ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, &key.public)
                .verify(&msg, sig)
        }
        Kind::EcdsaP256 => {
            // No SSH a assinatura ECDSA são dois inteiros (r, s); o ring confere r ‖ s.
            let mut rs = Reader(sig);
            let (r_int, s_int) = (mpint_32(rs.string()?)?, mpint_32(rs.string()?)?);
            if !rs.done() {
                return Err("assinatura com dados a mais".into());
            }
            let fixed = [r_int, s_int].concat();
            ring::signature::UnparsedPublicKey::new(
                &ring::signature::ECDSA_P256_SHA256_FIXED,
                &key.public,
            )
            .verify(&msg, &fixed)
        }
    };
    checked.map_err(|_| "assinatura da YubiKey não confere com o pacote".to_string())?;
    Ok(key.comment.clone())
}

#[cfg(test)]
mod tests {
    use base64::Engine;
    use ring::rand::SystemRandom;
    use ring::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, Ed25519KeyPair, KeyPair};
    use sha2::{Digest, Sha256, Sha512};

    use super::*;

    const APP: &str = "ssh:treasury-update";
    const BOTH: u8 = USER_PRESENT | USER_VERIFIED;

    /// Test key of either type, signing the way the YubiKey signs.
    enum TestKey {
        Ed(Ed25519KeyPair),
        Ec(EcdsaKeyPair),
    }

    fn key_pair(seed: u8) -> TestKey {
        TestKey::Ed(Ed25519KeyPair::from_seed_unchecked(&[seed; 32]).unwrap())
    }

    fn ecdsa_key_pair() -> TestKey {
        let rng = SystemRandom::new();
        let alg = &ECDSA_P256_SHA256_FIXED_SIGNING;
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(alg, &rng).unwrap();
        TestKey::Ec(EcdsaKeyPair::from_pkcs8(alg, pkcs8.as_ref(), &rng).unwrap())
    }

    /// Positive integer in SSH `mpint` format: no leading zeros, with one zero in
    /// front when the first bit is set.
    fn mpint(n: &[u8]) -> Vec<u8> {
        let n = &n[n.iter().position(|b| *b != 0).unwrap_or(n.len() - 1)..];
        let mut out = Vec::new();
        if n[0] & 0x80 != 0 {
            out.push(0);
        }
        out.extend_from_slice(n);
        out
    }

    impl TestKey {
        fn kind(&self) -> &'static str {
            match self {
                Self::Ed(_) => SK_ED25519,
                Self::Ec(_) => SK_ECDSA_P256,
            }
        }

        fn pub_blob(&self) -> Vec<u8> {
            let mut b = Vec::new();
            put_string(&mut b, self.kind().as_bytes());
            match self {
                Self::Ed(kp) => put_string(&mut b, kp.public_key().as_ref()),
                Self::Ec(kp) => {
                    put_string(&mut b, NISTP256.as_bytes());
                    put_string(&mut b, kp.public_key().as_ref());
                }
            }
            put_string(&mut b, APP.as_bytes());
            b
        }

        fn pub_line(&self, comment: &str) -> String {
            let b64 = base64::engine::general_purpose::STANDARD;
            format!("{} {} {comment}", self.kind(), b64.encode(self.pub_blob()))
        }

        /// The line of a key declared as the Mac's Secure Enclave.
        fn enclave_line(&self, comment: &str) -> String {
            format!("{SECURE_ENCLAVE} {}", self.pub_line(comment))
        }

        /// The signature field as it goes in SSH: 64 bytes for Ed25519, `mpint r` and
        /// `mpint s` for ECDSA.
        fn sign_field(&self, msg: &[u8]) -> Vec<u8> {
            match self {
                Self::Ed(kp) => kp.sign(msg).as_ref().to_vec(),
                Self::Ec(kp) => {
                    let sig = kp.sign(&SystemRandom::new(), msg).unwrap();
                    let (r, s) = sig.as_ref().split_at(32);
                    let mut out = Vec::new();
                    put_string(&mut out, &mpint(r));
                    put_string(&mut out, &mpint(s));
                    out
                }
            }
        }
    }

    /// Signature in the format that the YubiKey and `ssh-keygen -Y sign` produce. `sig_kind` is
    /// the type written inside the signature (the key's own, except in the test that swaps types).
    fn sign_as(kp: &TestKey, data: &[u8], namespace: &str, flags: u8, sig_kind: &str) -> String {
        let digest = Sha512::digest(data);
        let mut signed = MAGIC.to_vec();
        put_string(&mut signed, namespace.as_bytes());
        put_string(&mut signed, b"");
        put_string(&mut signed, b"sha512");
        put_string(&mut signed, &digest);
        let counter = 7u32;
        let mut msg = Sha256::digest(APP.as_bytes()).to_vec();
        msg.push(flags);
        msg.extend_from_slice(&counter.to_be_bytes());
        msg.extend_from_slice(&Sha256::digest(&signed));
        let mut sig_blob = Vec::new();
        put_string(&mut sig_blob, sig_kind.as_bytes());
        put_string(&mut sig_blob, &kp.sign_field(&msg));
        sig_blob.push(flags);
        sig_blob.extend_from_slice(&counter.to_be_bytes());
        let mut raw = MAGIC.to_vec();
        raw.extend_from_slice(&1u32.to_be_bytes());
        put_string(&mut raw, &kp.pub_blob());
        put_string(&mut raw, namespace.as_bytes());
        put_string(&mut raw, b"");
        put_string(&mut raw, b"sha512");
        put_string(&mut raw, &sig_blob);
        let b64 = base64::engine::general_purpose::STANDARD.encode(raw);
        let lines: Vec<&str> = b64
            .as_bytes()
            .chunks(70)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect();
        format!(
            "-----BEGIN SSH SIGNATURE-----\n{}\n-----END SSH SIGNATURE-----\n",
            lines.join("\n")
        )
    }

    fn sign(kp: &TestKey, data: &[u8], namespace: &str, flags: u8) -> String {
        sign_as(kp, data, namespace, flags, kp.kind())
    }

    #[test]
    fn the_signature_of_one_of_the_yubikeys_is_accepted() {
        let (a, b) = (key_pair(1), key_pair(2));
        let keys = parse_keys(&format!(
            "# chaves de teste\n{}\n\n{}\n",
            a.pub_line("yubikey-1"),
            b.pub_line("yubikey-2")
        ))
        .unwrap();
        assert_eq!(keys.len(), 2);
        let data = b"pacote inventado";
        let ns = namespace("1.2.3");
        assert_eq!(
            verify(&keys, &sign(&b, data, &ns, BOTH), data, &ns).unwrap(),
            "yubikey-2"
        );
    }

    /// The older YubiKey signs with ECDSA P-256; it is valid next to an Ed25519 one.
    #[test]
    fn an_ecdsa_yubikey_is_accepted_next_to_an_ed25519_one() {
        let (a, b) = (key_pair(1), ecdsa_key_pair());
        let keys = parse_keys(&format!(
            "{}\n{}\n",
            a.pub_line("yubikey-1"),
            b.pub_line("yubikey-2")
        ))
        .unwrap();
        assert_eq!(keys.len(), 2);
        let data = b"pacote inventado";
        let ns = namespace("1.2.3");
        // Several times: `r` and `s` change with every signature (with and without a leading zero).
        for _ in 0..40 {
            assert_eq!(
                verify(&keys, &sign(&b, data, &ns, BOTH), data, &ns).unwrap(),
                "yubikey-2"
            );
        }
        assert_eq!(
            verify(&keys, &sign(&a, data, &ns, BOTH), data, &ns).unwrap(),
            "yubikey-1"
        );
    }

    #[test]
    fn altered_package_unknown_key_other_namespace_or_no_touch_are_refused() {
        for (a, stranger) in [
            (key_pair(1), key_pair(9)),
            (ecdsa_key_pair(), ecdsa_key_pair()),
        ] {
            let keys = parse_keys(&a.pub_line("yubikey-1")).unwrap();
            let data = b"pacote inventado";
            let ns = namespace("1.2.3");
            let good = sign(&a, data, &ns, BOTH);
            assert!(verify(&keys, &good, data, &ns).is_ok());
            assert!(verify(&keys, &good, b"pacote inventadO", &ns).is_err());
            assert!(verify(&keys, &sign(&stranger, data, &ns, BOTH), data, &ns).is_err());
            assert!(verify(&keys, &sign(&a, data, "git", BOTH), data, &ns).is_err());
            // Touch only, PIN only or neither: refused.
            assert!(verify(&keys, &sign(&a, data, &ns, USER_PRESENT), data, &ns).is_err());
            assert!(verify(&keys, &sign(&a, data, &ns, USER_VERIFIED), data, &ns).is_err());
            assert!(verify(&keys, &sign(&a, data, &ns, 0), data, &ns).is_err());
            assert!(verify(&keys, "-----BEGIN SSH SIGNATURE-----\nAAAA\n", data, &ns).is_err());
            // A signature of an old version is not valid for the version announced now.
            let old = sign(&a, data, &namespace("1.0.0"), BOTH);
            assert!(verify(&keys, &old, data, &ns).is_err());
        }
    }

    /// The type written in the signature must be the key's: one does not pass for the other.
    #[test]
    fn the_signature_type_must_match_the_key() {
        let data = b"pacote inventado";
        let ns = namespace("1.2.3");
        let ed = key_pair(1);
        let keys = parse_keys(&ed.pub_line("yubikey-1")).unwrap();
        assert!(
            verify(
                &keys,
                &sign_as(&ed, data, &ns, BOTH, SK_ECDSA_P256),
                data,
                &ns
            )
            .is_err()
        );
        let ec = ecdsa_key_pair();
        let keys = parse_keys(&ec.pub_line("yubikey-2")).unwrap();
        assert!(verify(&keys, &sign_as(&ec, data, &ns, BOTH, SK_ED25519), data, &ns).is_err());
        assert!(
            verify(
                &keys,
                &sign_as(&ec, data, &ns, BOTH, "ecdsa-sha2-nistp256"),
                data,
                &ns
            )
            .is_err()
        );
    }

    /// The Mac key signs with the presence flag only (macOS does not set verification); the
    /// exception holds only for the declared key, never for the others.
    #[test]
    fn a_declared_secure_enclave_key_needs_presence_only() {
        let (mac, yubi) = (ecdsa_key_pair(), key_pair(1));
        let keys = parse_keys(&format!(
            "{}\n{}\n",
            mac.enclave_line("mac-1"),
            yubi.pub_line("yubikey-1")
        ))
        .unwrap();
        let data = b"pacote inventado";
        let ns = namespace("1.2.3");
        // The flags macOS sets (0x21) and presence alone.
        for flags in [0x21, USER_PRESENT, BOTH] {
            assert_eq!(
                verify(&keys, &sign(&mac, data, &ns, flags), data, &ns).unwrap(),
                "mac-1"
            );
        }
        assert!(verify(&keys, &sign(&mac, data, &ns, 0), data, &ns).is_err());
        assert!(verify(&keys, &sign(&mac, data, &ns, 0x20), data, &ns).is_err());
        // The rest still applies to it: package, version and key.
        let good = sign(&mac, data, &ns, 0x21);
        assert!(verify(&keys, &good, b"pacote inventadO", &ns).is_err());
        assert!(
            verify(
                &keys,
                &sign(&mac, data, &namespace("1.0.0"), 0x21),
                data,
                &ns
            )
            .is_err()
        );
        assert!(verify(&keys, &sign(&ecdsa_key_pair(), data, &ns, 0x21), data, &ns).is_err());
        // The YubiKey in the same file still needs the PIN.
        assert!(verify(&keys, &sign(&yubi, data, &ns, USER_PRESENT), data, &ns).is_err());
        assert!(verify(&keys, &sign(&yubi, data, &ns, 0x21), data, &ns).is_err());
        assert!(verify(&keys, &sign(&yubi, data, &ns, BOTH), data, &ns).is_ok());
        // The same P-256 key without the word is treated as a YubiKey: no PIN, refused.
        let keys = parse_keys(&mac.pub_line("mac-1")).unwrap();
        assert!(verify(&keys, &sign(&mac, data, &ns, 0x21), data, &ns).is_err());
    }

    #[test]
    fn the_secure_enclave_word_is_only_for_p256_hardware_keys() {
        let (ed, ec) = (key_pair(1), ecdsa_key_pair());
        assert!(parse_keys(&ec.enclave_line("mac-1")).is_ok());
        assert!(parse_keys(&ed.enclave_line("mac-1")).is_err());
        // Unknown word, or glued to the type, is not an option: it is an unsupported type.
        assert!(parse_keys(&format!("no-pin {}", ec.pub_line("x"))).is_err());
        assert!(parse_keys(&format!("{SECURE_ENCLAVE}{}", ec.pub_line("x"))).is_err());
        assert!(parse_keys(SECURE_ENCLAVE).is_err());
    }

    #[test]
    fn only_fido2_keys_are_accepted_in_the_key_file() {
        // Ordinary keys, in a file, of both algorithms.
        assert!(
            parse_keys(
                "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOMqqnkVzrm0SdG6UOoqKLsabgH5C9okWi0dh2l9GKJl x"
            )
            .is_err()
        );
        let b64 = base64::engine::general_purpose::STANDARD;
        let TestKey::Ec(ec) = ecdsa_key_pair() else {
            unreachable!()
        };
        let mut plain = Vec::new();
        put_string(&mut plain, b"ecdsa-sha2-nistp256");
        put_string(&mut plain, NISTP256.as_bytes());
        put_string(&mut plain, ec.public_key().as_ref());
        assert!(parse_keys(&format!("ecdsa-sha2-nistp256 {} x", b64.encode(&plain))).is_err());
        // Line type different from the type inside the key.
        let ed = key_pair(1);
        let line = format!("{SK_ECDSA_P256} {} x", b64.encode(ed.pub_blob()));
        assert!(parse_keys(&line).is_err());
        // Another curve and a compressed point.
        for (curve, point) in [
            ("nistp384", ec.public_key().as_ref().to_vec()),
            (NISTP256, {
                let mut p = ec.public_key().as_ref()[..33].to_vec();
                p[0] = 0x02;
                p
            }),
        ] {
            let mut blob = Vec::new();
            put_string(&mut blob, SK_ECDSA_P256.as_bytes());
            put_string(&mut blob, curve.as_bytes());
            put_string(&mut blob, &point);
            put_string(&mut blob, APP.as_bytes());
            assert!(parse_keys(&format!("{SK_ECDSA_P256} {} x", b64.encode(&blob))).is_err());
        }
        assert!(parse_keys("# só comentário\n").unwrap().is_empty());
    }

    #[test]
    fn ssh_integers_become_32_bytes() {
        assert_eq!(mpint_32(&[1]).unwrap()[31], 1);
        assert_eq!(mpint_32(&[0, 0x80]).unwrap()[31], 0x80);
        let mut full = vec![0u8];
        full.extend_from_slice(&[0xff; 32]);
        assert_eq!(mpint_32(&full).unwrap(), [0xff; 32]);
        assert!(mpint_32(&[]).is_err());
        assert!(mpint_32(&[0x80]).is_err(), "negative");
        assert!(mpint_32(&[1; 33]).is_err(), "too large");
    }

    /// The repository file always parses: a wrong key in it would break the update.
    #[test]
    fn the_embedded_key_file_parses() {
        embedded().unwrap();
    }

    /// Manual check against OpenSSH: writes a test key and signature of each
    /// type to `TREASURY_SSHSIG_OUT` for `ssh-keygen -Y verify` (docs/auto-update.md).
    #[test]
    #[ignore]
    fn write_vector_for_ssh_keygen() {
        let Ok(dir) = std::env::var("TREASURY_SSHSIG_OUT") else {
            return;
        };
        let dir = std::path::Path::new(&dir);
        let data = b"pacote inventado";
        std::fs::write(dir.join("pacote.bin"), data).unwrap();
        for (name, key) in [("ed25519", key_pair(1)), ("ecdsa", ecdsa_key_pair())] {
            std::fs::write(
                dir.join(format!("pacote.bin.{name}.sig")),
                sign(&key, data, &namespace("1.2.3"), BOTH),
            )
            .unwrap();
            std::fs::write(
                dir.join(format!("allowed_signers.{name}")),
                format!(
                    "teste namespaces=\"treasury-update@1.2.3\" {}\n",
                    key.pub_line("")
                ),
            )
            .unwrap();
        }
    }

    /// Check with the real keys (YubiKey and Mac): `TREASURY_SSHSIG_REAL` points to
    /// a folder with `chaves.pub` (the lines of `update-keys.pub`), `pacote.bin` and the
    /// `pacote.bin.*.sig` signatures made with `ssh-keygen -Y sign -n treasury-update@0.0.0`.
    #[test]
    #[ignore]
    fn real_yubikey_signatures_are_accepted() {
        let Ok(dir) = std::env::var("TREASURY_SSHSIG_REAL") else {
            return;
        };
        let dir = std::path::Path::new(&dir);
        let keys = parse_keys(&std::fs::read_to_string(dir.join("chaves.pub")).unwrap()).unwrap();
        let data = std::fs::read(dir.join("pacote.bin")).unwrap();
        let ns = namespace("0.0.0");
        let mut checked = 0;
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "sig") {
                let sig = std::fs::read_to_string(&path).unwrap();
                let who = verify(&keys, &sig, &data, &ns).unwrap();
                println!("{}: {who}", path.display());
                assert!(verify(&keys, &sig, b"outro pacote", &ns).is_err());
                checked += 1;
            }
        }
        assert!(checked > 0, "no signature in {}", dir.display());
    }
}
