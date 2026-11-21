//! Divisão de segredo de Shamir sobre GF(256) (polinômio do AES, 0x11b), para a
//! recuperação dividida: a chave de recuperação do cofre vira `n` partes e quaisquer `k`
//! delas a reconstroem; menos de `k` partes não revelam nada.
//!
//! Cada parte leva a versão, o índice `x`, os bytes `y` e 4 bytes de conferência (início do
//! SHA-256 do segredo), para recusar partes trocadas, digitadas errado ou de outro cofre.

use sha2::{Digest, Sha256};
use zeroize::{Zeroize, Zeroizing};

use crate::error::{Error, Result};
// `t` é a tabela de GF(256) aqui; o texto do catálogo é `tr`.
use crate::i18n::t as tr;

const VERSION: u8 = 1;
const CHECK_LEN: usize = 4;
const PREFIX: &str = "TRSP";

fn tables() -> ([u8; 256], [u8; 256]) {
    let (mut exp, mut log) = ([0u8; 256], [0u8; 256]);
    let mut x: u8 = 1;
    for i in 0..255u16 {
        exp[i as usize] = x;
        log[x as usize] = i as u8;
        // x *= 3 em GF(256)
        let hi = x & 0x80;
        let mut x2 = x << 1;
        if hi != 0 {
            x2 ^= 0x1b;
        }
        x ^= x2;
    }
    exp[255] = exp[0];
    (exp, log)
}

fn mul(a: u8, b: u8, t: &([u8; 256], [u8; 256])) -> u8 {
    if a == 0 || b == 0 {
        return 0;
    }
    let s = (t.1[a as usize] as u16 + t.1[b as usize] as u16) % 255;
    t.0[s as usize]
}

fn div(a: u8, b: u8, t: &([u8; 256], [u8; 256])) -> u8 {
    assert!(b != 0, "divisão por zero em GF(256)");
    if a == 0 {
        return 0;
    }
    let s = (t.1[a as usize] as i16 - t.1[b as usize] as i16).rem_euclid(255);
    t.0[s as usize]
}

fn check_of(secret: &[u8]) -> [u8; CHECK_LEN] {
    let h = Sha256::digest(secret);
    let mut c = [0u8; CHECK_LEN];
    c.copy_from_slice(&h[..CHECK_LEN]);
    c
}

/// Divide `secret` em `n` partes, `k` necessárias (2 ≤ k ≤ n ≤ 10).
pub fn split(secret: &[u8], k: u8, n: u8) -> Result<Vec<String>> {
    if !(2..=n).contains(&k) || n > 10 || secret.is_empty() {
        return Err(Error::invalid(tr("nucleo-geral-erro-divisao")));
    }
    let t = tables();
    // Coeficientes aleatórios: um polinômio de grau k-1 por byte do segredo.
    let mut coeffs = Zeroizing::new(vec![0u8; secret.len() * (k as usize - 1)]);
    getrandom::fill(&mut coeffs).map_err(|e| Error::Crypto(e.to_string()))?;
    let check = check_of(secret);
    let mut out = vec![];
    for x in 1..=n {
        let mut raw = Zeroizing::new(Vec::with_capacity(2 + secret.len() + CHECK_LEN));
        raw.push(VERSION);
        raw.push(x);
        for (i, s) in secret.iter().enumerate() {
            // Horner: y = s + c1·x + c2·x² + …
            let mut y = 0u8;
            for j in (0..k as usize - 1).rev() {
                y = mul(y, x, &t) ^ coeffs[i * (k as usize - 1) + j];
            }
            y = mul(y, x, &t) ^ s;
            raw.push(y);
        }
        raw.extend_from_slice(&check);
        let enc = base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &raw);
        let groups: Vec<&str> = enc
            .as_bytes()
            .chunks(4)
            .map(|c| std::str::from_utf8(c).unwrap_or_default())
            .collect();
        out.push(format!("{PREFIX}-{}", groups.join("-")));
    }
    Ok(out)
}

struct Share {
    x: u8,
    y: Zeroizing<Vec<u8>>,
    check: [u8; CHECK_LEN],
}

fn parse(text: &str) -> Result<Share> {
    let clean: String = text
        .trim()
        .to_ascii_uppercase()
        .trim_start_matches(PREFIX)
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    let mut raw = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &clean)
        .ok_or_else(|| Error::invalid(tr("nucleo-geral-erro-parte-invalida")))?;
    if raw.len() < 2 + 1 + CHECK_LEN || raw[0] != VERSION || raw[1] == 0 {
        raw.zeroize();
        return Err(Error::invalid(tr("nucleo-geral-erro-parte-invalida")));
    }
    let n = raw.len();
    let mut check = [0u8; CHECK_LEN];
    check.copy_from_slice(&raw[n - CHECK_LEN..]);
    let share = Share {
        x: raw[1],
        y: Zeroizing::new(raw[2..n - CHECK_LEN].to_vec()),
        check,
    };
    raw.zeroize();
    Ok(share)
}

/// Reconstrói o segredo a partir de `k` ou mais partes (as que sobrarem também servem).
pub fn combine<S: AsRef<str>>(parts: &[S]) -> Result<Zeroizing<Vec<u8>>> {
    let shares: Vec<Share> = parts
        .iter()
        .map(AsRef::as_ref)
        .filter(|p| !p.trim().is_empty())
        .map(parse)
        .collect::<Result<_>>()?;
    if shares.len() < 2 {
        return Err(Error::invalid(tr("nucleo-geral-erro-duas-partes")));
    }
    let len = shares[0].y.len();
    let check = shares[0].check;
    for (i, s) in shares.iter().enumerate() {
        if s.y.len() != len || s.check != check {
            return Err(Error::invalid(tr(
                "nucleo-geral-erro-partes-outro-conjunto",
            )));
        }
        if shares[..i].iter().any(|o| o.x == s.x) {
            return Err(Error::invalid(tr("nucleo-geral-erro-parte-repetida")));
        }
    }
    let t = tables();
    let mut secret = Zeroizing::new(vec![0u8; len]);
    for (i, si) in shares.iter().enumerate() {
        // Lagrange em x = 0: Π_{j≠i} x_j / (x_j − x_i); em GF(256), subtração = xor.
        let mut l = 1u8;
        for (j, sj) in shares.iter().enumerate() {
            if i != j {
                l = mul(l, div(sj.x, sj.x ^ si.x, &t), &t);
            }
        }
        for b in 0..len {
            secret[b] ^= mul(si.y[b], l, &t);
        }
    }
    if check_of(&secret) != check {
        return Err(Error::invalid(tr(
            "nucleo-geral-erro-partes-nao-reconstroem",
        )));
    }
    Ok(secret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_two_of_three_rebuild_the_secret() {
        let secret: Vec<u8> = (0..32).map(|i| (i * 7 + 3) as u8).collect();
        let parts = split(&secret, 2, 3).unwrap();
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|p| p.starts_with("TRSP-")));
        for (a, b) in [(0, 1), (0, 2), (1, 2), (2, 0)] {
            let got = combine(&[parts[a].clone(), parts[b].clone()]).unwrap();
            assert_eq!(got.as_slice(), secret.as_slice());
        }
        assert_eq!(combine(&parts).unwrap().as_slice(), secret.as_slice());
        // Minúsculas e sem hífens também valem.
        let loose = parts[1].to_lowercase().replace('-', " ");
        assert_eq!(
            combine(&[parts[0].clone(), loose]).unwrap().as_slice(),
            secret.as_slice()
        );
    }

    #[test]
    fn one_part_or_mixed_sets_are_refused() {
        let s1 = [1u8; 32];
        let s2 = [2u8; 32];
        let a = split(&s1, 2, 3).unwrap();
        let b = split(&s2, 2, 3).unwrap();
        assert!(combine(&[a[0].clone()]).is_err());
        assert!(combine(&[a[0].clone(), a[0].clone()]).is_err());
        assert!(combine(&[a[0].clone(), b[1].clone()]).is_err());
        // Um caractere trocado é detectado.
        let mut bad: Vec<char> = a[1].chars().collect();
        let i = bad.len() - 9;
        bad[i] = if bad[i] == 'A' { 'B' } else { 'A' };
        assert!(combine(&[a[0].clone(), bad.into_iter().collect()]).is_err());
    }

    #[test]
    fn three_of_five_also_works() {
        let secret = [9u8; 32];
        let p = split(&secret, 3, 5).unwrap();
        assert!(combine(&p[..2]).is_err());
        assert_eq!(combine(&p[1..4]).unwrap().as_slice(), &secret);
    }
}
