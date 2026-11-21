//! Pix "copia e cola" (BR Code, padrão EMV QRCPS-MPM do Banco Central): montar o de uma chave
//! aleatória, sem valor, e o CRC que a leitura de comprovantes (`payments::pix`) confere. Tudo
//! aqui, sem rede.

use crate::error::{Error, Result};

fn field(id: &str, value: &str) -> String {
    format!("{id}{:02}{value}", value.len())
}

/// Só letras sem acento, dígitos e espaço, em maiúsculas, até `max` caracteres.
fn ascii(s: &str, max: usize) -> String {
    let folded = crate::import::text::fold(s).to_ascii_uppercase();
    folded
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
        .take(max)
        .collect::<String>()
        .trim()
        .to_string()
}

/// CRC16-CCITT (polinômio 0x1021, início 0xFFFF), como pede o BR Code.
pub(crate) fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for b in data {
        crc ^= (*b as u16) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

/// Chave aleatória (EVP): um UUID.
pub fn is_random_key(key: &str) -> bool {
    let k = key.trim();
    k.len() == 36
        && k.chars().enumerate().all(|(i, c)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                c == '-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
}

/// O "copia e cola" do Pix, sem valor (quem doa escolhe no app do banco).
pub fn payload(key: &str, name: &str, city: &str) -> Result<String> {
    if !is_random_key(key) {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-geral-erro-pix-chave",
        )));
    }
    let name = ascii(name, 25);
    let city = ascii(city, 15);
    if name.is_empty() || city.is_empty() {
        return Err(Error::invalid(crate::i18n::t(
            "nucleo-geral-erro-pix-recebedor",
        )));
    }
    let account = field("00", "br.gov.bcb.pix") + &field("01", &key.trim().to_ascii_lowercase());
    let mut p = String::new();
    p += &field("00", "01");
    p += &field("26", &account);
    p += &field("52", "0000");
    p += &field("53", "986");
    p += &field("58", "BR");
    p += &field("59", &name);
    p += &field("60", &city);
    p += &field("62", &field("05", "***"));
    p += "6304";
    let crc = crc16(p.as_bytes());
    Ok(format!("{p}{crc:04X}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn br_code_has_the_fields_and_a_valid_crc() {
        let p = payload(
            "123e4567-e89b-12d3-a456-426614174000",
            "Loja Exemplo Ltda",
            "Florianópolis",
        )
        .unwrap();
        assert!(p.starts_with("000201"));
        assert!(p.contains("0014br.gov.bcb.pix0136123e4567-e89b-12d3-a456-426614174000"));
        assert!(p.contains("5917LOJA EXEMPLO LTDA"));
        assert!(p.contains("6013FLORIANOPOLIS"));
        assert!(p.contains("62070503***"));
        // O CRC confere com o calculado sobre tudo até "6304".
        let (body, crc) = p.split_at(p.len() - 4);
        assert_eq!(format!("{:04X}", crc16(body.as_bytes())), crc);
        // Exemplo conhecido do CRC16-CCITT-FALSE.
        assert_eq!(crc16(b"123456789"), 0x29B1);
        assert!(payload("email@exemplo.com", "x", "y").is_err());
    }
}
