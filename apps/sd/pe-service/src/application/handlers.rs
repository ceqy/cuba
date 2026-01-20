use crate::application::commands::{CalculatePriceCommand, UpdateConditionCommand};
use crate::domain::{AppliedCondition, PricingCondition, PricingResult};
use crate::infrastructure::repository::PricingRepository;
use anyhow::Result;
use chrono::Utc;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct PricingHandler {
    repo: Arc<PricingRepository>,
}

impl PricingHandler {
    pub fn new(repo: Arc<PricingRepository>) -> Self {
        Self { repo }
    }

    pub async fn calculate_price(&self, cmd: CalculatePriceCommand) -> Result<Vec<PricingResult>> {
        let mut results = Vec::new();
        for item in cmd.items {
            let conditions = self
                .repo
                .find_conditions(
                    &item.material,
                    cmd.customer.as_deref(),
                    &cmd.sales_org,
                    cmd.pricing_date,
                )
                .await?;

            // Apply conditions
            let mut net_price = Decimal::ZERO;
            let mut applied = Vec::new();
            for c in conditions {
                net_price += c.amount * item.quantity;
                applied.push(AppliedCondition {
                    condition_type: c.condition_type,
                    value: c.amount,
                    currency: c.currency,
                    description: format!("Condition {} applied", c.condition_id),
                });
            }

            let tax_rate = Decimal::new(13, 2); // 13%
            let tax_amount = net_price * tax_rate;
            let gross_price = net_price + tax_amount;

            results.push(PricingResult {
                item_id: item.item_id,
                net_price,
                tax_amount,
                gross_price,
                conditions: applied,
            });
        }
        Ok(results)
    }

    pub async fn update_condition(&self, cmd: UpdateConditionCommand) -> Result<String> {
        let condition_id = Uuid::new_v4();
        let c = PricingCondition {
            condition_id,
            condition_type: cmd.condition_type,
            material: cmd.material,
            customer: cmd.customer,
            sales_org: cmd.sales_org,
            amount: cmd.amount,
            currency: cmd.currency,
            valid_from: cmd.valid_from,
            valid_to: cmd.valid_to,
            created_at: Utc::now(),
        };
        self.repo.save(&c).await?;
        Ok(condition_id.to_string())
    }
}
