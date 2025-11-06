# Week 5: Async + GAT Payment Processor

Consolidation week project combining async programming with Generic Associated Types.

## Concept

Demonstrates lending iterator patterns in async context using GATs to yield borrowed `&str` 
references to payment data.

## Implementations

- **PaymentsData**: JSON payment processor
- **CsvPaymentsData**: CSV payment processor

Both use the same `AsyncPaymentProcessor` trait with GAT pattern.

## Key Learning

Lending iterators (GAT with borrowed data) cannot hold multiple items simultaneously - 
this is by design, not a limitation of async specifically.

## Run Tests
```bash
cargo test
```
