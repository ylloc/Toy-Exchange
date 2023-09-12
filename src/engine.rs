use std::collections::btree_map::CursorMut;
use std::collections::BTreeMap;
use std::ops::Bound;
use std::time::{Duration, Instant};

use rand::prelude::*;

use crate::order::*;

const EPS: f64 = 0.01;
const INF_DURATION: Duration = Duration::new(1_000_000_000, 0);

#[derive(Debug, Clone)]
pub struct Asset(pub String);

impl From<&str> for Asset {
  fn from(value: &str) -> Self {
    Asset(value.to_string())
  }
}

#[derive(Debug)]
pub struct OrderBook {
  pub book_type: TypeOrder,
  pub tree: BTreeMap<Order, i64>,
  pub references: BTreeMap<i64, Order>,
  pub asset: Asset,
}

impl OrderBook {
  pub fn new(book_type: TypeOrder,
             name: Asset) -> OrderBook {
    OrderBook {
      book_type,
      tree: BTreeMap::new(),
      references: BTreeMap::new(),
      asset: name,
    }
  }

  pub fn add_order(&mut self, order: Order) {
    self.references.insert(order.hash, order.clone());
    self.tree.insert(order, order.hash);
  }

  pub fn create_fantom_order_for_search(&self, search_type: SearchTime, price: f64) -> Order {
    let time = match search_type {
      SearchTime::Lower => Instant::now() - INF_DURATION,
      SearchTime::Upper => Instant::now() + INF_DURATION,
    };
    return Order {
      type_order: self.book_type,
      price,
      quantity: 0.,
      hash: -1,
      time,
    };
  }

  pub fn lower_bound(&mut self, price: f64) -> CursorMut<'_, Order, i64> {
    let fantom_order = self.create_fantom_order_for_search(SearchTime::Lower, price);
    self.tree.lower_bound_mut(Bound::Included(&fantom_order))
  }

  pub fn upper_bound(&mut self, price: f64) -> CursorMut<'_, Order, i64> {
    let fantom_order = self.create_fantom_order_for_search(SearchTime::Lower, price);
    self.tree.upper_bound_mut(Bound::Included(&fantom_order))
  }

  pub fn erase_order(&mut self, hash: i64) {
    let key = self.references.remove(&hash);
    if let Some(order) = key {
      self.tree.remove(&order);
    }
  }

  pub fn len(&self) -> usize {
    self.tree.len()
  }
}

#[derive(Debug)]
pub struct Engine {
  pub asset: Asset,
  pub buy_book: OrderBook,
  pub sell_book: OrderBook,
  pub current_price: f64,
}

impl Engine {
  pub fn add_order(&mut self, order: Order) {
    let mut cursor = match order.type_order {
      TypeOrder::Sell => self.buy_book.lower_bound(order.price),
      TypeOrder::Buy => self.sell_book.upper_bound(order.price),
    };
    let mut add = Vec::new();
    let mut remove = Vec::new();
    if cursor.key().is_none() {
      match order.type_order {
        TypeOrder::Sell => { self.sell_book.add_order(order); }
        TypeOrder::Buy => { self.buy_book.add_order(order); }
      }
    } else {
      let mut need = order.quantity;
      while need > EPS && cursor.key().is_some() {
        let value = cursor.value().unwrap().clone();
        if cursor.key().unwrap().quantity >= need {
          let mut incomplete_order = cursor.remove_current().unwrap().0;
          self.current_price = incomplete_order.price;
          incomplete_order.quantity -= need;
          if incomplete_order.quantity > EPS {
            add.push((incomplete_order.hash, incomplete_order.clone()));
          } else {
            remove.push(incomplete_order.hash);
          }
          need = 0.;
        } else {
          remove.push(value);
          let part_filled_order = cursor.remove_current_and_move_back().unwrap().0;
          self.current_price = part_filled_order.price;
          if order.type_order == TypeOrder::Sell {
            cursor.move_next();
          }
          need -= part_filled_order.quantity;
        }
      }
      if need > EPS {
        match order.type_order {
          TypeOrder::Sell => { self.sell_book.add_order(Order { quantity: need, ..order }); }
          TypeOrder::Buy => { self.buy_book.add_order(Order { quantity: need, ..order }); }
        }
      }
    }
    let book = match order.type_order {
      TypeOrder::Sell => &mut self.buy_book,
      TypeOrder::Buy => &mut self.sell_book,
    };
    remove.into_iter().for_each(|x| {
      book.references.remove(&x);
    });
    add.into_iter().for_each(|(_, x)| {
      book.add_order(x);
    });
  }

  pub fn erase_order(&mut self, hash: i64, order_type: TypeOrder) {
    match order_type {
      TypeOrder::Sell => self.sell_book.erase_order(hash),
      TypeOrder::Buy => self.buy_book.erase_order(hash),
    }
  }

  pub(crate) fn current_price(&self) -> f64 {
    self.current_price
  }
}