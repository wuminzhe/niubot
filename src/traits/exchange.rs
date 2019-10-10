use std::collections::HashMap;
use custom_error::custom_error;

custom_error!{pub Error
    Network{source: reqwest::Error} = "unable to request to exchange",
    Json{source: serde_json::Error} = "unable to parse json content",
}

pub trait Exchange {
    fn get_depth(&self, market: &str, limit: u16) -> Result<Depth, Error>;

    fn create_order(&self, market: &str, side: &str, price: f64, volume: f64) -> Result<String, Error>;
    fn cancel_order(&self, id: &str);
    fn get_order(&self, id: &str) -> Result<Order, Error>;

    fn get_accounts(&self) -> Result<HashMap<String, Account>, Error>;

    fn get_markets(&self) -> Result<Vec<Market>, Error>;
}

#[derive(Debug)]
pub struct Depth {
    pub asks: Vec<(f64, f64)>,
    pub bids: Vec<(f64, f64)>,
}

#[derive(Debug)]
pub struct Account {
    pub asset: String,
    pub free: f64,
    pub locked: f64,
}

#[derive(Debug)]
pub struct Order {
    pub id: String,
    pub state: String,
    pub price: f64,
    pub origin_volume: f64,
}

#[derive(Debug)]
pub struct Market {
    pub id: String,
    pub base_unit: String,
    pub quote_unit: String,
}
