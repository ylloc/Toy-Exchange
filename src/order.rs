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
             hash: i64) -> Order
  {
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
    Some(
      match self
      .price
      .total_cmp(&other.price)
      {
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

#[test]
fn something_strange_work() {
  let a = Order::new(TypeOrder::Buy, 1., 1., 1);
  let c = Order::new(TypeOrder::Buy, 0.2, 2., 3);
  let mut z = BTreeMap::new();
  z.insert(a, 1);
  z.insert(c, 1);
  let mut cursor = z.upper_bound_mut(Bound::Included(&Order::new(TypeOrder::Buy, 1., 0., 44)));
  println!("{:#?}", cursor);
}
