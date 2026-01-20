use crate::application::commands::CreatePurchaseOrderCommand;
use crate::domain::{PurchaseOrder, PurchaseOrderItem, PurchaseOrderScheduleLine};
use crate::infrastructure::repository::PurchaseOrderRepository;
use anyhow::Result;
use chrono::Utc;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

pub struct CreatePurchaseOrderHandler {
    repo: Arc<PurchaseOrderRepository>,
}

impl CreatePurchaseOrderHandler {
    pub fn new(repo: Arc<PurchaseOrderRepository>) -> Self {
        Self { repo }
    }

    pub async fn handle(&self, cmd: CreatePurchaseOrderCommand) -> Result<String> {
        let order_id = Uuid::new_v4();
        let order_number = format!("45{}", Utc::now().timestamp_subsec_nanos()); // 45xxxxxxxx range
        let created_at = Utc::now();

        let items = cmd
            .items
            .into_iter()
            .map(|i| {
                PurchaseOrderItem {
                    item_id: Uuid::new_v4(),
                    order_id,
                    item_number: i.item_number,
                    item_category: 1, // Standard
                    material: i.material,
                    short_text: None,
                    plant: i.plant,
                    storage_location: None,
                    material_group: None,
                    quantity: i.quantity,
                    quantity_unit: i.quantity_unit,
                    net_price: i.net_price,
                    price_unit: 1,
                    currency: cmd.currency.clone(),
                    gr_based_iv: true,
                    tax_code: None,
                    account_assignment_category: None,
                    requisition_number: None,
                    requisition_item: None,
                    deletion_indicator: false,
                    schedule_lines: vec![PurchaseOrderScheduleLine {
                        schedule_line_id: Uuid::new_v4(),
                        item_id: Uuid::nil(),
                        schedule_line_number: 1,
                        delivery_date: Utc::now().date_naive(), // MVP: immediate
                        scheduled_quantity: i.quantity,
                        goods_receipt_quantity: Decimal::ZERO,
                    }],
                }
            })
            .collect();

        let po = PurchaseOrder {
            order_id,
            order_number: order_number.clone(),
            document_type: cmd.document_type,
            company_code: cmd.company_code,
            purchasing_org: cmd.purchasing_org,
            purchasing_group: cmd.purchasing_group,
            supplier: cmd.supplier,
            order_date: cmd.order_date.unwrap_or(Utc::now().date_naive()),
            currency: cmd.currency,
            payment_terms: None,
            incoterms: None,
            incoterms_location: None,
            complete_delivery: false,
            release_status: 1,
            items,
            created_at,
            updated_at: created_at,
        };

        self.repo.save(&po).await?;
        Ok(order_number)
    }
}
