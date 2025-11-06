use async_trait::async_trait;
use serde::{Serialize, Deserialize};

pub mod day1_replication;

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

/// `PaymentsData` processes a list of payment JSON strings.
/// 
/// # GAT Justification
/// 
/// This type requires GATs because it yields borrowed `&str` references to payment 
/// JSON from its owned `Vec<String>`. Each call to `next_payment()` borrows from 
/// the internal vector with a fresh lifetime tied to that specific call.
/// 
/// Without GATs, we couldn't express "return a reference that borrows from self 
/// with a lifetime determined per method call." The GAT syntax `type Payment<'a>` 
/// where Self: 'a` explicitly links the borrowed data's lifetime to the implementor.
/// 
/// # Lending Iterator Limitation
/// 
/// Due to the lending pattern, you cannot hold multiple payment references 
/// simultaneously. Each `next_payment()` call mutably borrows `self`, and the 
/// returned `&str` keeps that borrow active.
/// 
/// # Example
/// 
/// ```
/// use week5_async_stream_proc::{PaymentsData, AsyncPaymentProcessor};
/// 
/// # tokio_test::block_on(async {
/// let mut processor = PaymentsData {
///     tx_list: vec![
///         r#"{"date":"2025-01-01","amount":100.0,"method":"credit","is_successful":true}"#.to_string()
///     ],
///     position: 0,
/// };
/// 
/// // Sequential processing works:
/// let payment = processor.next_payment().await;
/// assert!(payment.is_some());
/// 
/// // Holding multiple refs doesn't compile:
/// // let p1 = processor.next_payment().await;
/// // let p2 = processor.next_payment().await; // ERROR: already borrowed
/// # });
/// ```
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

/// `CsvPaymentsData` processes a list of payment CSV rows.
/// 
/// # GAT Justification
/// 
/// This type requires GATs because it yields borrowed `&str` references to payment 
/// CSV from its owned `Vec<String>`. Each call to `next_payment()` borrows from 
/// the internal vector with a fresh lifetime tied to that specific call.
/// 
/// Without GATs, we couldn't express "return a reference that borrows from self 
/// with a lifetime determined per method call." The GAT syntax `type Payment<'a>` 
/// where Self: 'a` explicitly links the borrowed data's lifetime to the implementor.
/// 
/// # Lending Iterator Limitation
/// 
/// Due to the lending pattern, you cannot hold multiple payment references 
/// simultaneously. Each `next_payment()` call mutably borrows `self`, and the 
/// returned `&str` keeps that borrow active.
/// 
/// # Example
/// 
/// ```
/// use week5_async_stream_proc::{CsvPaymentsData, AsyncPaymentProcessor};
/// 
/// # tokio_test::block_on(async {
/// let data = vec![ 
///     r#"2025-01-01","100.0","credit","true"#.to_string(),
///     r#"2025-01-02","150.0","debit","false"#.to_string(),
/// ];
/// let mut csv_processor = CsvPaymentsData {
///     rows: data,
///     position: 0,
/// };
/// 
/// // Sequential processing works:
/// let payment = csv_processor.next_payment().await;
/// assert!(payment.is_some());
/// 
/// // Holding multiple refs doesn't compile:
/// // let p1 = csv_processor.next_payment().await;
/// // let p2 = csv_processor.next_payment().await; // ERROR: already borrowed
/// # });
/// ```
#[derive(Debug, PartialEq)]
pub struct CsvPaymentsData {
    rows: Vec<String>,  // Each String is one CSV row: "2025-01-01,100.50,credit,true"
    position: usize,
}

#[async_trait]
impl AsyncPaymentProcessor for CsvPaymentsData {
    type Payment<'a> = &'a str
        where Self: 'a;

    async fn next_payment(&mut self) -> Option<Self::Payment<'_>> {
        if self.position >= self.rows.len() {
            return None;
        }

        let res = &self.rows[self.position];
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

    #[tokio::test]
    async fn test_csv_next_and_process() {
        let data = vec![ 
            r#"2025-01-01","100.0","credit","true"#.to_string(),
            r#"2025-01-02","150.0","debit","false"#.to_string(),
        ];

        let mut csv_processor = CsvPaymentsData {
            rows: data,
            position: 0,
        };

        let first = csv_processor.next_payment().await;
        assert_eq!(
            first, 
            Some(r#"2025-01-01","100.0","credit","true"#)
        );


        let amount = csv_processor.process(|csv_str: &str| {
            let split_as_vals: Vec<String> = csv_str.split(',').map(|x| x.to_string()).collect();
            split_as_vals[1].trim_matches('"').parse::<f64>().unwrap()
        }).await;

        assert_eq!(amount, Some(150.0));
    }

    #[tokio::test]
    async fn test_borrowing_limitation_documented() {
        // This test demonstrates the lending limitation by design
        // UNCOMMENT to verify it won't compile:
    
        /*
        let mut processor = PaymentsData {
            tx_list: sample_payments(),
            position: 0,
        };
    
        let payment1 = processor.next_payment().await;
        let payment2 = processor.next_payment().await;  // ERROR: already borrowed
    
        // Fails because: Can't hold multiple mutable borrows of processor
        // This is BY DESIGN for lending iterators
        assert_ne!(payment1, payment2);
        */
    }
}

