use crate::application::commands::{
    CreateSalesOrderCommand, GetSalesOrderQuery, ListSalesOrdersQuery,
};
use crate::domain::{SalesOrder, SalesOrderItem, SalesOrderScheduleLine};
use crate::infrastructure::repository::SalesOrderRepository;
use anyhow::Result;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::sync::Arc;
use uuid::Uuid; // Needed for Decimal calculations

pub struct CreateSalesOrderHandler {
    repo: Arc<SalesOrderRepository>,
}

impl CreateSalesOrderHandler {
    pub fn new(repo: Arc<SalesOrderRepository>) -> Self {
        Self { repo }
    }

    pub async fn handle(&self, cmd: CreateSalesOrderCommand) -> Result<SalesOrder> {
        let order_id = Uuid::new_v4();
        // Generate a random order number or use a sequence in real app
        let order_number = format!("OR{}", Utc::now().timestamp_subsec_nanos());

        let created_at = Utc::now();
        let document_date = Utc::now().date_naive();

        let mut total_net_value = Decimal::ZERO;

        let items: Vec<SalesOrderItem> = cmd
            .items
            .iter()
            .map(|i| {
                // Mock pricing logic: quantity * 10
                let unit_price = Decimal::from_i32(10).unwrap();
                let net_value = i.order_quantity * unit_price;
                total_net_value += net_value;

                SalesOrderItem {
                    item_id: Uuid::new_v4(),
                    order_id,
                    item_number: i.item_number,
                    material: i.material.clone(),
                    item_description: Some(format!("Item {}", i.material)),
                    order_quantity: i.order_quantity,
                    sales_unit: i.sales_unit.clone(),
                    plant: i.plant.clone(),
                    storage_location: i.storage_location.clone(),
                    net_value,
                    tax_amount: Some(net_value * Decimal::from_f32(0.19).unwrap()), // 19% Tax mock
                    item_category: Some("TAN".to_string()),
                    rejection_reason: None,
                    higher_level_item: None,
                    schedule_lines: vec![SalesOrderScheduleLine {
                        schedule_line_id: Uuid::new_v4(),
                        item_id: Uuid::nil(), // Will be updated
                        schedule_line_number: 1,
                        delivery_date: cmd.requested_delivery_date.unwrap_or(document_date),
                        confirmed_quantity: i.order_quantity,
                    }],
                }
            })
            .collect();

        let order = SalesOrder {
            order_id,
            order_number,
            order_type: cmd.order_type,
            sales_org: cmd.sales_org,
            distribution_channel: cmd.distribution_channel,
            division: cmd.division,
            sold_to_party: cmd.sold_to_party,
            ship_to_party: cmd.ship_to_party,
            customer_po: cmd.customer_po,
            customer_po_date: cmd.customer_po_date,
            document_date,
            requested_delivery_date: cmd.requested_delivery_date,
            currency: cmd.currency,
            net_value: total_net_value,
            pricing_procedure: None,
            shipping_conditions: None,
            overall_status: "OPEN".to_string(),
            delivery_block: None,
            billing_block: None,
            items,
            created_at,
            updated_at: created_at,
        };

        self.repo.save(&order).await?;
        Ok(order)
    }
}

pub struct GetSalesOrderHandler {
    repo: Arc<SalesOrderRepository>,
}

impl GetSalesOrderHandler {
    pub fn new(repo: Arc<SalesOrderRepository>) -> Self {
        Self { repo }
    }

    pub async fn handle(&self, query: GetSalesOrderQuery) -> Result<Option<SalesOrder>> {
        self.repo.find_by_number(&query.order_number).await
    }
}

pub struct ListSalesOrdersHandler {
    repo: Arc<SalesOrderRepository>,
}

impl ListSalesOrdersHandler {
    pub fn new(repo: Arc<SalesOrderRepository>) -> Self {
        Self { repo }
    }

    pub async fn handle(&self, query: ListSalesOrdersQuery) -> Result<Vec<SalesOrder>> {
        self.repo
            .list(query.sold_to_party, query.limit as i64)
            .await
    }
}
