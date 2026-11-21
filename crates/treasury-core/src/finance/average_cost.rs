//! Preço médio ponderado (regra da Receita Federal): custos de compra (corretagem,
//! emolumentos) entram no custo; vendas realizam resultado contra o preço médio e não
//! alteram o médio; desdobramentos e grupamentos ajustam quantidade e médio sem mudar o
//! custo total; bonificações somam quantidade com o custo atribuído.

use rust_decimal::Decimal;

use crate::error::{Error, Result};
use crate::i18n::t;

#[derive(Debug, Clone, PartialEq)]
pub enum PositionEvent {
    Buy {
        qty: Decimal,
        price: Decimal,
        fees: Decimal,
    },
    Sell {
        qty: Decimal,
        price: Decimal,
        fees: Decimal,
    },
    /// Desdobramento (fator > 1) ou grupamento (fator < 1): nova quantidade = antiga × fator.
    Split { factor: Decimal },
    /// Bonificação em ações, com o custo unitário atribuído pela companhia.
    Bonus { qty: Decimal, unit_cost: Decimal },
    /// Amortização/devolução de capital: reduz o custo total sem mudar a quantidade.
    CapitalReturn { amount: Decimal },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Position {
    pub qty: Decimal,
    /// Custo total (moeda do ativo).
    pub cost: Decimal,
    /// Resultado realizado acumulado.
    pub realized: Decimal,
}

impl Position {
    pub fn avg_price(&self) -> Option<Decimal> {
        (!self.qty.is_zero()).then(|| self.cost / self.qty)
    }

    /// Aplica um evento. Devolve o resultado realizado pelo evento (vendas).
    pub fn apply(&mut self, ev: &PositionEvent) -> Result<Decimal> {
        match ev {
            PositionEvent::Buy { qty, price, fees } => {
                if *qty <= Decimal::ZERO {
                    return Err(Error::invalid(t(
                        "nucleo-planejamento-erro-compra-positiva",
                    )));
                }
                self.qty += qty;
                self.cost += qty * price + fees;
                Ok(Decimal::ZERO)
            }
            PositionEvent::Sell { qty, price, fees } => {
                if *qty <= Decimal::ZERO {
                    return Err(Error::invalid(t("nucleo-planejamento-erro-venda-positiva")));
                }
                if *qty > self.qty {
                    return Err(Error::invalid(t("nucleo-planejamento-erro-venda-maior")));
                }
                let avg = self.cost / self.qty;
                let cost_out = avg * qty;
                let realized = qty * price - fees - cost_out;
                self.qty -= qty;
                self.cost = if self.qty.is_zero() {
                    Decimal::ZERO
                } else {
                    self.cost - cost_out
                };
                self.realized += realized;
                Ok(realized)
            }
            PositionEvent::Split { factor } => {
                if *factor <= Decimal::ZERO {
                    return Err(Error::invalid(t(
                        "nucleo-planejamento-erro-fator-desdobramento",
                    )));
                }
                self.qty *= factor;
                Ok(Decimal::ZERO)
            }
            PositionEvent::Bonus { qty, unit_cost } => {
                self.qty += qty;
                self.cost += qty * unit_cost;
                Ok(Decimal::ZERO)
            }
            PositionEvent::CapitalReturn { amount } => {
                self.cost = (self.cost - amount).max(Decimal::ZERO);
                Ok(Decimal::ZERO)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn average_price_with_fees_and_partial_sale() {
        let mut p = Position::default();
        p.apply(&PositionEvent::Buy {
            qty: dec!(100),
            price: dec!(10),
            fees: dec!(5),
        })
        .unwrap();
        p.apply(&PositionEvent::Buy {
            qty: dec!(100),
            price: dec!(12),
            fees: dec!(5),
        })
        .unwrap();
        assert_eq!(p.avg_price().unwrap(), dec!(11.05)); // (1005 + 1205) / 200
        let r = p
            .apply(&PositionEvent::Sell {
                qty: dec!(50),
                price: dec!(15),
                fees: dec!(2),
            })
            .unwrap();
        assert_eq!(r, dec!(195.50)); // 750 − 2 − 552,50
        assert_eq!(
            p.avg_price().unwrap(),
            dec!(11.05),
            "venda não muda o médio"
        );
        assert_eq!(p.qty, dec!(150));
    }

    #[test]
    fn split_keeps_cost_and_invariant() {
        let mut p = Position::default();
        p.apply(&PositionEvent::Buy {
            qty: dec!(10),
            price: dec!(30),
            fees: dec!(0),
        })
        .unwrap();
        let cost = p.cost;
        p.apply(&PositionEvent::Split { factor: dec!(3) }).unwrap();
        assert_eq!(p.qty, dec!(30));
        assert_eq!(p.cost, cost);
        assert_eq!(p.avg_price().unwrap(), dec!(10));
        p.apply(&PositionEvent::Split { factor: dec!(0.1) })
            .unwrap(); // grupamento 10:1
        assert_eq!(p.qty, dec!(3));
        assert_eq!(p.avg_price().unwrap(), dec!(100));
    }

    #[test]
    fn bonus_and_full_sale_and_errors() {
        let mut p = Position::default();
        p.apply(&PositionEvent::Buy {
            qty: dec!(100),
            price: dec!(20),
            fees: dec!(0),
        })
        .unwrap();
        p.apply(&PositionEvent::Bonus {
            qty: dec!(10),
            unit_cost: dec!(5),
        })
        .unwrap();
        assert_eq!(p.cost, dec!(2050));
        assert!(
            p.apply(&PositionEvent::Sell {
                qty: dec!(111),
                price: dec!(1),
                fees: dec!(0)
            })
            .is_err()
        );
        let r = p
            .apply(&PositionEvent::Sell {
                qty: dec!(110),
                price: dec!(25),
                fees: dec!(0),
            })
            .unwrap();
        assert_eq!(r, dec!(700));
        assert_eq!(p.cost, dec!(0));
        assert!(p.avg_price().is_none());
        assert!(
            p.apply(&PositionEvent::Buy {
                qty: dec!(0),
                price: dec!(1),
                fees: dec!(0)
            })
            .is_err()
        );
    }
}
