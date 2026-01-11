use std::sync::Arc;
use crate::domain::{MaterialDocument, MaterialDocumentItem, MaterialStock};
use crate::infrastructure::repository::InventoryRepository;
use crate::application::commands::{PostStockMovementCommand, GetStockOverviewQuery};
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::{Utc, Datelike};

pub struct PostStockMovementHandler {
    repo: Arc<InventoryRepository>,
}

impl PostStockMovementHandler {
    pub fn new(repo: Arc<InventoryRepository>) -> Self {
        Self { repo }
    }

    pub async fn handle(&self, cmd: PostStockMovementCommand) -> Result<String> {
        let document_id = Uuid::new_v4();
        // Generate Doc Number: 4900000000 + Random
        let document_number = format!("49{}", Utc::now().timestamp_subsec_nanos());
        let fiscal_year = cmd.posting_date.year();

        let mut items = Vec::new();
        for (idx, i) in cmd.items.into_iter().enumerate() {
            // Simple Movement Type Logic for MVP
            // 101 (GR for PO) -> S (Debit) -> Increase Stock
            // 561 (Initial Entry) -> S (Debit) -> Increase Stock
            // 601 (GI for Delivery) -> H (Credit) -> Decrease Stock
            // 201 (GI for Cost Center) -> H (Credit) -> Decrease Stock
            let debit_credit = match i.movement_type.as_str() {
                "101" | "501" | "561" => "S",
                "201" | "261" | "601" => "H",
                _ => "S", // Defaulting to S for safety, but should error in real app
            };

            items.push(MaterialDocumentItem {
                item_id: Uuid::new_v4(),
                document_id,
                line_item_number: (idx + 1) as i32,
                movement_type: i.movement_type,
                debit_credit_indicator: debit_credit.to_string(),
                material: i.material,
                plant: i.plant,
                storage_location: i.storage_location,
                batch: i.batch,
                quantity: i.quantity,
                unit_of_measure: i.unit_of_measure,
                amount_lc: None, // Simplified
            });
        }

        let doc = MaterialDocument {
            document_id,
            document_number: document_number.clone(),
            fiscal_year,
            document_date: cmd.document_date,
            posting_date: cmd.posting_date,
            document_type: Some("WA".to_string()), // Default
            reference_document: cmd.reference_document,
            header_text: cmd.header_text,
            items,
            created_at: Utc::now(),
        };

        self.repo.save_material_document(&doc).await?;
        Ok(document_number)
    }
}

pub struct GetStockOverviewHandler {
    repo: Arc<InventoryRepository>,
}

impl GetStockOverviewHandler {
    pub fn new(repo: Arc<InventoryRepository>) -> Self {
        Self { repo }
    }

    pub async fn handle(&self, query: GetStockOverviewQuery) -> Result<Vec<MaterialStock>> {
        self.repo.get_stock(&query.material, &query.plant, query.storage_location.as_deref()).await
    }
}
