use std::collections::HashMap;
use serde_json::Value;

use crate::util;
use crate::traits::exchange::{
    Exchange, Error, Depth, Account, Order, Market,
};

pub struct Rfinex {
    domain: String,
    api_verion: String,
    access_key: String,
    secret_key: String,
}

impl Rfinex {
    pub fn new(domain: &str, api_verion: &str, access_key: &str, secret_key: &str) -> Rfinex {
        Rfinex {
            domain: domain.to_owned(),
            api_verion: api_verion.to_owned(),
            access_key: access_key.to_owned(),
            secret_key: secret_key.to_owned(),
        }
    }

    // ///////////////////////////////// 
    // private helper functions
    // /////////////////////////////////
    fn call(&self, action: &str, path: &str, params: &Vec<(&str, &str)>) -> Result<String, reqwest::Error> {
        let query_str = self.build_query_str(params);
        let url = self.build_url(path, &query_str);

        let client = reqwest::Client::new();
        let mut resp = if action == "POST" {
            client.post(&url).send()?
        } else {
            client.get(&url).send()?
        };

        Ok(resp.text()?)
    }

    fn private_call(&self, action: &str, path: &str, params: &mut Vec<(&str, &str)>, secret_key: &str) -> Result<String, reqwest::Error> {
        let payload = self.build_payload(action, path, params);
        let signature = util::sign(&payload, secret_key);
        // println!("{}", payload);

        let mut params_with_signature = params.clone();
        params_with_signature.push(("signature", &signature));
        let query_str = self.build_query_str(&params_with_signature);
        // println!("{}", query_str);

        let url = self.build_url(path, &query_str);
        // println!("{}", url);

        let client = reqwest::Client::new();
        let mut resp = if action == "POST" {
            client.post(&url).send()?
        } else {
            client.get(&url).send()?
        };

        Ok(resp.text()?)
    }

    fn build_payload(&self, action: &str, path: &str, params: &mut [(&str, &str)]) -> String {
        params.sort();
        let params_str = self.build_query_str(params);
        action.to_owned() + "|" + path + "|" + &params_str
    }

    fn build_query_str(&self, params: &[(&str, &str)]) -> String {
        let mut result = Vec::new();
        for e in params {
            let name = e.0;
            let value = e.1;
            result.push(name.to_owned()+"="+value);
        }
        result.join("&")
    }

    fn build_url(&self, path: &str, query_str: &str) -> String {
        ["https://", &self.domain, path, "?", query_str].concat().to_string()
    }
}

impl Exchange for Rfinex {
    fn get_depth(&self, market: &str, limit: u16) -> Result<Depth, Error> {
        let path = ["/api/", &self.api_verion, "/depth"].concat();

        let timestamp = &util::get_unix_timestamp().to_string();
        let limit = limit.to_string();
        let params = vec![
            ("market", market),
            ("limit", &limit),
            ("tonce", timestamp)
        ];

        let data = self.call("GET", &path, &params)?;

        let v: Value = serde_json::from_str(&data)?;

        let mut depth = Depth {
            asks: vec![],
            bids: vec![]
        };

        for ask_value in v["body"]["ask"].as_array().unwrap() {
            let ask = ask_value.as_array().unwrap();
            let price = ask[0].as_str().unwrap().parse::<f64>().unwrap();
            let volume = ask[1].as_str().unwrap().parse::<f64>().unwrap();
            depth.asks.push((price, volume));
        }

        for bid_value in v["body"]["bid"].as_array().unwrap() {
            let bid = bid_value.as_array().unwrap();
            let price = bid[0].as_str().unwrap().parse::<f64>().unwrap();
            let volume = bid[1].as_str().unwrap().parse::<f64>().unwrap();
            depth.bids.push((price, volume));
        }

        Ok(depth)
    }

    fn create_order(&self, market: &str, side: &str, price: f64, volume: f64) -> Result<String, Error> {
        let path = ["/api/", &self.api_verion, "/orders"].concat();

        let timestamp = &util::get_unix_timestamp().to_string();
        let p = price.to_string();
        let v = volume.to_string();
        let mut params: Vec<(&str, &str)> = vec![
            ("access_key", &self.access_key),
            ("price", &p),
            ("volume", &v),
            ("market", market),
            ("side", side),
            ("tonce", timestamp),
        ];

        let data = self.private_call("POST", &path, &mut params, &self.secret_key)?;

        let v: Value = serde_json::from_str(&data)?;
        let id = v["body"]["id"].as_u64().unwrap().to_string();
        
        Ok(id)
    }

    fn cancel_order(&self, id: &str) {
        let path = ["/api/", &self.api_verion, "/order/delete"].concat();

        let timestamp = &util::get_unix_timestamp().to_string();
        let mut params: Vec<(&str, &str)> = vec![
            ("access_key", &self.access_key),
            ("id", id),
            ("tonce", timestamp),
        ];

        let _data = self.private_call("POST", &path, &mut params, &self.secret_key);
    }

    fn get_order(&self, id: &str) -> Result<Order, Error> {
        let path = ["/api/", &self.api_verion, "/order"].concat();

        let timestamp = &util::get_unix_timestamp().to_string();
        let mut params: Vec<(&str, &str)> = vec![
            ("access_key", &self.access_key),
            ("id", id),
            ("tonce", timestamp),
        ];

        let data = self.private_call("GET", &path, &mut params, &self.secret_key)?;
        let v: Value = serde_json::from_str(&data)?;

        Ok(
            Order {
                id: v["body"]["id"].as_str().unwrap().to_owned(),
                state: v["body"]["state"].as_str().unwrap().to_owned(),
                price: v["body"]["price"].as_f64().unwrap(),
                origin_volume: v["body"]["origin_volume"].as_f64().unwrap()
            }
        )
    }

    fn get_accounts(&self) -> Result<HashMap<String, Account>, Error> {
        let path = ["/api/", &self.api_verion, "/members/accounts"].concat();

        let timestamp = &util::get_unix_timestamp().to_string();
        let mut params: Vec<(&str, &str)> = vec![
            ("access_key", &self.access_key),
            ("tonce", timestamp)
        ];

        let data = self.private_call("GET", &path, &mut params, &self.secret_key)?;
        
        let mut accounts = HashMap::new();
        let v: Value = serde_json::from_str(&data)?;

        for account_value in v["body"].as_array().unwrap() {
            let currency = account_value["currency"].as_str().unwrap();
            let balance  = account_value["balance"].as_str().unwrap();
            let locked   = account_value["locked"].as_str().unwrap();
            let account = Account {
                asset: currency.to_string(),
                free: balance.parse::<f64>().unwrap(),
                locked: locked.parse::<f64>().unwrap()
            };
            accounts.insert(currency.to_string(), account);

        }

        Ok(accounts)
    }

    fn get_markets(&self) -> Result<Vec<Market>, Error> {
        let path = ["/api/", &self.api_verion, "/markets"].concat();

        let params = vec![];

        let data = self.call("GET", &path, &params)?;

        let mut markets = vec![];
        let v: Value = serde_json::from_str(&data)?;
        
        for market_value in v["body"].as_array().unwrap() {
            markets.push(
                Market {
                    id: market_value["id"].as_str().unwrap().to_owned(),
                    base_unit: market_value["base_unit"].as_str().unwrap().to_owned(),
                    quote_unit: market_value["quote_unit"].as_str().unwrap().to_owned()
                }
            )
        }

        Ok(markets)
    }

}

