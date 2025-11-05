use async_trait::async_trait;
use serde::{Serialize, Deserialize};

#[async_trait]
pub trait AsyncPaymentProcessor {
    type Payment<'a> where Self: 'a;
    
    async fn next_payment(&mut self) -> Option<Self::Payment<'_>>;
    
    // Transform method - what should this do?
    async fn process<F, T>(&mut self, f: F) -> Option<T>
    where 
        F: Fn(Self::Payment<'_>) -> T + Send,
        T: Send;
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PaymentInfo {
    date: String,
    amount: f64,
    method: String,
    is_successful: bool,
}

pub struct PaymentsData {
    tx_list: Vec<String>, 
    position: usize,
}

#[async_trait]
impl AsyncPaymentProcessor for PaymentsData {
    type Payment<'a> = &'a str
        where Self: 'a;

    async fn next_payment(&mut self) -> Option<Self::Payment<'_>> {
        if self.position >= self.tx_list.len() {
            return None;
        }

        let res = &self.tx_list[self.position];
        self.position += 1;
        Some(res)
    }

    async fn process<F, T>(&mut self, f: F) -> Option<T>
    where 
        F: Fn(Self::Payment<'_>) -> T + Send,
        T: Send,
    {
        self.next_payment().await.map(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function idea - or inline the data?
    fn sample_payments() -> Vec<String> {
        vec![
            r#"{"date":"2025-01-01","amount":100.0,"method":"credit","is_successful":true}"#.to_string(),
            r#"{"date":"2025-01-02","amount":50.0,"method":"debit","is_successful":true}"#.to_string(),
            r#"{"date":"2025-01-03","amount":75.0,"method":"credit","is_successful":false}"#.to_string(),
        ]
    }
    
    #[tokio::test]
    async fn test_next_payment_sequential() {
        // Your implementation here
        // Remember: Your code has the boundary bug!
        let json_payments = sample_payments();
        let mut processor = PaymentsData {
            tx_list: json_payments,
            position: 0,
        };
    
        // Test next_payment()
        // Remark: we cannot test this code if we had `first`, `second` and `third`
        // initialized AND then asserting - this limitation, related to lifetime borrowing, is by design 
        // that several async mutable borrows cannot be allowed at the current stage 
        let first = processor.next_payment().await;
        assert_eq!(
            first, 
            Some(r#"{"date":"2025-01-01","amount":100.0,"method":"credit","is_successful":true}"#)
        );

        let second = processor.next_payment().await;
        assert_eq!(
            second, 
            Some(r#"{"date":"2025-01-02","amount":50.0,"method":"debit","is_successful":true}"#)
        );

        let third = processor.next_payment().await;
        assert_eq!(
            third, 
            Some(r#"{"date":"2025-01-03","amount":75.0,"method":"credit","is_successful":false}"#)
        );
        assert_eq!(processor.next_payment().await, None);

    }
    
    // Continue with other tests...
    #[tokio::test]
    async fn test_process_success() {
        let json_payments = sample_payments();
        let mut processor = PaymentsData {
            tx_list: json_payments,
            position: 0,
        };
        // Test process() with amount extraction
        let amount = processor.process(|json: &str| {
            let info: PaymentInfo = serde_json::from_str(json).unwrap();
            info.amount  // Returns f64
        }).await;

        assert_eq!(amount, Some(100.0));
    }

    #[tokio::test]
    async fn test_empty_data() {
        let mut processor = PaymentsData {
            tx_list: vec![],
            position: 0,
        };

        let amount = processor.process(|json: &str| {
            let info: PaymentInfo = serde_json::from_str(json).unwrap();
            info.amount  // Returns f64
        }).await;

        assert_eq!(amount, None);
    }
}

