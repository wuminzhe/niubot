mod util;
mod traits;
mod exchanges;

use std::env;
use std::collections::HashMap;
use traits::exchange::{Exchange, Market};
use exchanges::rfinex::Rfinex;

fn get_markets_with_different_quotes<'a>(markets: &'a Vec<Market>, includes_quotes: &[&'a str]) -> HashMap<&'a str, Vec<&'a Market>> {
    // 按base分类: base -> the_markets
    let mut markets_by_base: HashMap<&str, Vec<&Market>> = HashMap::new();
    for market in markets {
        let base_unit = &market.base_unit;
        let the_markets = markets_by_base.entry(base_unit).or_insert(vec![]);
        the_markets.push(market);
    }

    markets_by_base.retain(|_, the_markets| the_markets.len() > 1 && the_markets.iter().all(|the_market| includes_quotes.contains(&&the_market.quote_unit[..])));
    markets_by_base
}

fn main() {
    let access_key = env::var("ACCESS").unwrap();
    let secret_key = env::var("SECRET").unwrap();

    let rfinex = Rfinex::new("rfinex.vip", "v2", &access_key, &secret_key);
    let markets = rfinex.get_markets().unwrap();
    let result = get_markets_with_different_quotes(&markets, &["usdt", "cnst"]);
    for (base, the_markets) in result {
        println!("{}", base);
        the_markets.iter().for_each(|the_market| {
            println!("  {}", the_market.id);
        });
    }
}

