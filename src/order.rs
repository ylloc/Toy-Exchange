use std::cmp::Ordering;
use std::collections::{Bound, BTreeMap};
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TypeOrder {
  Sell,
  Buy,
}

pub enum SearchTime {
  Lower,
  Upper,
}

#[derive(Clone, Copy, Debug)]
pub struct Order {
  pub type_order: TypeOrder,
  pub price: f64,
  pub quantity: f64,
  pub time: Instant,
  pub hash: i64,
}

impl Order {
  pub fn new(type_order: TypeOrder,
             price: f64,
             quantity: f64,
             hash: i64) -> Order {
    Order {
      type_order,
      price,
      quantity,
      time: Instant::now(),
      hash,
    }
  }
}

impl Eq for Order {}

impl PartialEq<Self> for Order {
  fn eq(&self, other: &Self) -> bool {
    self.hash == other.hash
  }
}

impl PartialOrd<Self> for Order {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    if self == other {
      return Some(Ordering::Equal);
    }
    Some(match self.price.total_cmp(&other.price) {
      Ordering::Equal => match self.time.cmp(&other.time) {
        Ordering::Less => match self.type_order {
          TypeOrder::Sell => Ordering::Greater,
          TypeOrder::Buy => Ordering::Less,
        },
        _ => match self.type_order {
          TypeOrder::Sell => Ordering::Less,
          TypeOrder::Buy => Ordering::Greater,
        }
      },
      it => it,
    },
    )
  }
}

impl Ord for Order {
  fn cmp(&self, other: &Self) -> Ordering {
    self.partial_cmp(other).unwrap_or(Ordering::Greater)
  }
}
