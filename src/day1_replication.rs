use serde::{Serialize, Deserialize};
use async_trait::async_trait;

#[async_trait]
pub trait AsyncPaymentProcessor {
    type Payment<'p>
        where Self: 'p;

    async fn next_payment(&mut self) -> Option<Self::Payment<'_>>;

    async fn process<F, T>(&mut self, f: F) -> Option<T> 
        where F: FnOnce(&str) -> T + Send,
              T: Send;
}

#[derive(Debug, PartialEq)]
pub struct JSONPayments {
    tx_list: Vec<String>,
    position: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PaymentInfo {
    date: String,
    amount: f64,
    method: String,
    is_successful: bool,
}

#[async_trait]
impl AsyncPaymentProcessor for JSONPayments {
    type Payment<'p> = &'p str
        where Self: 'p;

    async fn next_payment(&mut self) -> Option<Self::Payment<'_>> {
        if self.tx_list.len() == self.position {
            return None;
        }
        let res = &self.tx_list[self.position];
        self.position += 1;
        Some(res)
    }

    async fn process<F, T>(&mut self, f: F) -> Option<T> 
        where F: FnOnce(&str) -> T + Send,
              T: Send 
        {
            self.next_payment().await.map(f)
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_replication_works() {
        let mut processor = JSONPayments {
            tx_list: vec![
                r#"{"date":"2025-01-01","amount":100.0,"method":"credit","is_successful":true}"#.to_string(),
                r#"{"date":"2025-01-02","amount":50.0,"method":"debit","is_successful":true}"#.to_string(),
                r#"{"date":"2025-01-03","amount":75.0,"method":"credit","is_successful":false}"#.to_string(),
            ],
            position: 0,
        };

        let first = processor.next_payment().await;
        assert_eq!(first, Some(r#"{"date":"2025-01-01","amount":100.0,"method":"credit","is_successful":true}"#));

        let second_processed = processor.process(|jd: &str| {
            let p_info: PaymentInfo = serde_json::from_str(jd).unwrap();
            p_info.amount
        }).await;

        assert_eq!(second_processed, Some(50.0));
    }
}
