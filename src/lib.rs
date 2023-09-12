#![feature(btree_cursors)]
mod engine;
mod order;

#[cfg(test)]
mod tests {
  use std::time::Instant;
  use rand::Rng;
  use crate::engine::*;
  use crate::order::*;

  fn create_engine() -> Engine {
    let usd = "USD";
    Engine {
      asset: Asset(usd.to_string()),
      buy_book: OrderBook::new(TypeOrder::Buy, usd.into()),
      sell_book: OrderBook::new(TypeOrder::Sell, usd.into()),
      current_price: 0.,
    }
  }

  #[test]
  fn small_test1() {
    let mut engine = create_engine();
    engine.add_order(Order::new(TypeOrder::Sell, 1., 1., 1));
    engine.add_order(Order::new(TypeOrder::Sell, 1., 1., 2));
    engine.add_order(Order::new(TypeOrder::Buy, 2., 1.5, 3));

    assert_eq!(engine.sell_book.len(), 1);
    assert_eq!(engine.sell_book.tree.first_key_value().unwrap().0.quantity, 0.5);
    assert_eq!(engine.sell_book.len(), 1);
  }

  #[test]
  fn small_test2() {
    /// 0.01 < 0.1, so there will be no active orders!
    let mut engine = create_engine();
    engine.add_order(Order::new(TypeOrder::Sell, 1.2, 1., 1));
    engine.add_order(Order::new(TypeOrder::Sell, 1.2, 1.1, 2));
    engine.add_order(Order::new(TypeOrder::Buy, 1.3, 2.101, 3));
    assert_eq!(engine.sell_book.len(), 0);
    assert_eq!(engine.buy_book.len(), 0);
  }

  #[test]
  fn small_test3() {
    let mut engine = create_engine();
    engine.add_order(Order::new(TypeOrder::Sell, 1., 1., 1));
    assert_eq!(engine.sell_book.len(), 1);
    engine.add_order(Order::new(TypeOrder::Buy, 1.2, 0.3, 2));
    assert_eq!(engine.buy_book.len(), 0);
  }

  #[test]
  /// It must took no more then 10 seconds
  fn large_test() {
    let time_st = Instant::now();
    let mut engine = create_engine();
    let mut rng = rand::thread_rng();
    let n = 1_000_000;

    for i in 0..n {
      let sell_or_buy = rand::random::<bool>();
      let price = engine.current_price();
      match sell_or_buy {
        true => { engine.add_order(Order::new(TypeOrder::Buy, price + (rng.gen::<f64>() - 0.4) / 10., 3. + rng.gen::<f64>(), i)); }
        false => { engine.add_order(Order::new(TypeOrder::Sell, price + (rng.gen::<f64>() - 0.4) / 10., 3. + rng.gen::<f64>(), i)); }
      }
    }

    let duration = Instant::now() - time_st;
    assert!(duration.as_secs() < 10);
  }

  #[test]
  fn one_more_speed_test() {
    let n = 1_000_000;
    let time_st = Instant::now();
    let mut orderbook = OrderBook::new(TypeOrder::Buy, "near".into());
    for i in 0..n {
      orderbook.add_order(Order::new(TypeOrder::Buy, i as f64, 1., i));
    }

    for j in (0..n / 1000).map(|x| x * 999 + 2) {
      let x = orderbook.lower_bound(j as f64).key().unwrap().clone().hash;
      orderbook.erase_order(x);
    }

    let duration = Instant::now() - time_st;
    assert_eq!(orderbook.len(), 999000);
    assert!(duration.as_secs() < 10);
  }
}