use crate::application::commands::{
    InvoiceItemCmd, MatchInvoiceCommand, PostInvoiceCommand, ReceiveInvoiceCommand,
};
use crate::application::handlers::InvoiceHandler;
use crate::infrastructure::repository::InvoiceRepository;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::pm::iv::v1 as iv_v1;

use iv_v1::invoice_processing_service_server::InvoiceProcessingService;
use iv_v1::*;

pub struct IvServiceImpl {
    handler: Arc<InvoiceHandler>,
    repo: Arc<InvoiceRepository>,
}

impl IvServiceImpl {
    pub fn new(handler: Arc<InvoiceHandler>, repo: Arc<InvoiceRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl InvoiceProcessingService for IvServiceImpl {
    async fn receive_invoice(
        &self,
        request: Request<ReceiveInvoiceRequest>,
    ) -> Result<Response<ReceiveInvoiceResponse>, Status> {
        let req = request.into_inner();
        let header = req.header.unwrap_or_default();
        let gross = header
            .gross_amount
            .map(|a| Decimal::from_str(&a.value).unwrap_or_default())
            .unwrap_or_default();
        let tax = header
            .tax_amount
            .map(|a| Decimal::from_str(&a.value).unwrap_or_default())
            .unwrap_or_default();
        let cmd = ReceiveInvoiceCommand {
            company_code: header.company_code,
            supplier_invoice_number: header.supplier_invoice_number,
            document_date: chrono::Utc::now().date_naive(),
            gross_amount: gross,
            tax_amount: tax,
            items: req
                .items
                .into_iter()
                .map(|i| InvoiceItemCmd {
                    item_number: i.item_number,
                    po_number: if i.purchase_order_number.is_empty() {
                        None
                    } else {
                        Some(i.purchase_order_number)
                    },
                    po_item: if i.purchase_order_item == 0 {
                        None
                    } else {
                        Some(i.purchase_order_item)
                    },
                    material: if i.material.is_empty() {
                        None
                    } else {
                        Some(i.material)
                    },
                    quantity: i
                        .quantity
                        .map(|q| Decimal::from_str(&q.value).unwrap_or_default())
                        .unwrap_or_default(),
                    amount: i
                        .amount
                        .map(|a| Decimal::from_str(&a.value).unwrap_or_default())
                        .unwrap_or_default(),
                })
                .collect(),
        };
        let id = self
            .handler
            .receive_invoice(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ReceiveInvoiceResponse {
            internal_invoice_id: id,
            status: InvoiceProcessingStatus::Received as i32,
            job_id: "".to_string(),
        }))
    }

    async fn match_invoice(
        &self,
        request: Request<MatchInvoiceRequest>,
    ) -> Result<Response<MatchInvoiceResponse>, Status> {
        let req = request.into_inner();
        let invoice_id = uuid::Uuid::parse_str(&req.internal_invoice_id)
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;
        let cmd = MatchInvoiceCommand { invoice_id };
        let (success, _) = self
            .handler
            .match_invoice(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(MatchInvoiceResponse {
            internal_invoice_id: req.internal_invoice_id,
            match_status: if success {
                MatchStatus::Success as i32
            } else {
                MatchStatus::PriceVariance as i32
            },
            variances: vec![],
        }))
    }

    async fn post_invoice(
        &self,
        request: Request<PostInvoiceRequest>,
    ) -> Result<Response<PostInvoiceResponse>, Status> {
        let req = request.into_inner();
        let invoice_id = uuid::Uuid::parse_str(&req.internal_invoice_id)
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;
        let cmd = PostInvoiceCommand {
            invoice_id,
            accept_variances: req.accept_variances,
        };
        let doc_num = self
            .handler
            .post_invoice(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(PostInvoiceResponse {
            success: true,
            document_reference: Some(common_v1::SystemDocumentReference {
                document_number: doc_num,
                fiscal_year: 2026,
                company_code: "".to_string(),
                document_category: "".to_string(),
                document_type: "".to_string(),
            }),
            messages: vec![],
        }))
    }

    async fn get_invoice(
        &self,
        request: Request<GetInvoiceRequest>,
    ) -> Result<Response<InvoiceDetail>, Status> {
        let req = request.into_inner();
        let id_str = match req.identifier {
            Some(get_invoice_request::Identifier::InternalInvoiceId(id)) => id,
            _ => return Err(Status::invalid_argument("ID required")),
        };
        let invoice_id =
            uuid::Uuid::parse_str(&id_str).map_err(|_| Status::invalid_argument("Invalid UUID"))?;
        let inv = self
            .repo
            .find_by_id(invoice_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Invoice not found"))?;
        Ok(Response::new(InvoiceDetail {
            internal_invoice_id: inv.invoice_id.to_string(),
            document_reference: None,
            status: InvoiceProcessingStatus::Received as i32,
            header: Some(InvoiceHeader {
                company_code: inv.company_code,
                document_date: None,
                posting_date: None,
                supplier_invoice_number: inv.supplier_invoice_number,
                gross_amount: Some(common_v1::MonetaryValue {
                    value: inv.gross_amount.to_string(),
                    currency_code: inv.currency.clone(),
                }),
                tax_amount: None,
                currency: inv.currency,
                payment_terms: inv.payment_terms.unwrap_or_default(),
                header_text: inv.header_text.unwrap_or_default(),
            }),
            items: inv
                .items
                .into_iter()
                .map(|i| InvoiceItem {
                    item_number: i.item_number,
                    purchase_order_number: i.po_number.unwrap_or_default(),
                    purchase_order_item: i.po_item.unwrap_or(0),
                    material: i.material.unwrap_or_default(),
                    short_text: i.short_text.unwrap_or_default(),
                    quantity: Some(common_v1::QuantityValue {
                        value: i.quantity.to_string(),
                        unit_code: i.unit,
                    }),
                    unit: "".to_string(),
                    amount: Some(common_v1::MonetaryValue {
                        value: i.amount.to_string(),
                        currency_code: "CNY".to_string(),
                    }),
                    tax_code: i.tax_code.unwrap_or_default(),
                    goods_receipt_document: "".to_string(),
                    goods_receipt_year: 0,
                    goods_receipt_item: 0,
                })
                .collect(),
        }))
    }

    // Stubs
    async fn park_invoice(
        &self,
        _r: Request<ParkInvoiceRequest>,
    ) -> Result<Response<ParkInvoiceResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn reverse_invoice(
        &self,
        _r: Request<ReverseInvoiceRequest>,
    ) -> Result<Response<ReverseInvoiceResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn get_processing_status(
        &self,
        _r: Request<GetProcessingStatusRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        Err(Status::unimplemented(""))
    }
}
