use tonic::{Request, Response, Status};
use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::infrastructure::grpc::fi::gl::v1::gl_journal_entry_service_server::GlJournalEntryService;
use crate::infrastructure::grpc::fi::gl::v1::*;
use crate::application::handlers::{
    CreateJournalEntryHandler, GetJournalEntryHandler, ListJournalEntriesHandler, PostJournalEntryHandler
};
use crate::application::commands::{
    CreateJournalEntryCommand, PostJournalEntryCommand, LineItemDTO
};
use crate::application::queries::{
    GetJournalEntryQuery, ListJournalEntriesQuery
};
use crate::domain::repositories::JournalRepository;

pub struct GlServiceImpl<R> {
    create_handler: Arc<CreateJournalEntryHandler<R>>,
    get_handler: Arc<GetJournalEntryHandler<R>>,
    list_handler: Arc<ListJournalEntriesHandler<R>>,
    post_handler: Arc<PostJournalEntryHandler<R>>,
}

impl<R: JournalRepository> GlServiceImpl<R> {
    pub fn new(
        create_handler: Arc<CreateJournalEntryHandler<R>>,
        get_handler: Arc<GetJournalEntryHandler<R>>,
        list_handler: Arc<ListJournalEntriesHandler<R>>,
        post_handler: Arc<PostJournalEntryHandler<R>>,
    ) -> Self {
        Self {
            create_handler,
            get_handler,
            list_handler,
            post_handler,
        }
    }
}

#[tonic::async_trait]
impl<R: JournalRepository + 'static> GlJournalEntryService for GlServiceImpl<R> {
    // Associated Type
    type StreamJournalEntriesStream = std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<JournalEntryDetail, Status>> + Send + Sync + 'static>>;

    async fn create_journal_entry(&self, request: Request<CreateJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        let req = request.into_inner();
        
        let header = req.header.ok_or_else(|| Status::invalid_argument("Missing header"))?;
        
        // Basic mapping for MVP
        let posting_date_ts = header.posting_date.ok_or_else(|| Status::invalid_argument("Missing posting_date"))?;
        let posting_date = chrono::DateTime::from_timestamp(posting_date_ts.seconds, 0)
             .ok_or_else(|| Status::invalid_argument("Invalid posting_date"))?
             .naive_utc()
             .date();
             
        let document_date_ts = header.document_date.unwrap_or(posting_date_ts);
        let document_date = chrono::DateTime::from_timestamp(document_date_ts.seconds, 0)
             .unwrap_or(chrono::DateTime::from_timestamp(posting_date_ts.seconds, 0).unwrap())
             .naive_utc()
             .date();

        let lines: Result<Vec<LineItemDTO>, Status> = req.line_items.into_iter().map(|l| {
            let amount_doc = l.amount_in_document_currency.ok_or_else(|| Status::invalid_argument("Missing amount"))?;
            let amount = Decimal::from_str(&amount_doc.value)
                .map_err(|e| Status::invalid_argument(format!("Invalid amount: {}", e)))?;
            
            Ok(LineItemDTO {
                account_id: l.gl_account,
                debit_credit: l.debit_credit_indicator,
                amount,
                cost_center: if l.cost_center.is_empty() { None } else { Some(l.cost_center) },
                profit_center: if l.profit_center.is_empty() { None } else { Some(l.profit_center) },
                text: if l.text.is_empty() { None } else { Some(l.text) },
            })
        }).collect();

        let cmd = CreateJournalEntryCommand {
            company_code: header.company_code,
            fiscal_year: header.fiscal_year,
            posting_date,
            document_date,
            currency: header.currency,
            reference: if header.reference_document.is_empty() { None } else { Some(header.reference_document) },
            lines: lines?,
            post_immediately: req.post_immediately,
        };

        match self.create_handler.handle(cmd).await {
            Ok(entry) => Ok(Response::new(map_to_response(entry))),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_journal_entry(&self, request: Request<GetJournalEntryRequest>) -> Result<Response<JournalEntryDetail>, Status> {
        let req = request.into_inner();
        let id = Uuid::parse_str(&req.journal_entry_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid UUID: {}", e)))?;

        match self.get_handler.handle(GetJournalEntryQuery { id }).await {
            Ok(Some(entry)) => Ok(Response::new(map_to_detail(entry))),
            Ok(None) => Err(Status::not_found("Journal Entry not found")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn list_journal_entries(&self, request: Request<ListJournalEntriesRequest>) -> Result<Response<ListJournalEntriesResponse>, Status> {
        let req = request.into_inner();
        
        let mut status_filter = None;
        if !req.status_filter.is_empty() {
             status_filter = Some(req.status_filter[0].to_string());
        }

        let query = ListJournalEntriesQuery {
            company_code: req.company_code,
            status: status_filter,
            page: req.pagination.as_ref().map(|p| p.page as u64).unwrap_or(1),
            page_size: req.pagination.as_ref().map(|p| p.page_size as u64).unwrap_or(20),
        };

        match self.list_handler.handle(query.clone()).await {
            Ok(result) => {
                let items: Vec<JournalEntrySummary> = result.items.into_iter().map(map_to_summary).collect();
                let total_pages = if result.page_size > 0 {
                    ((result.total_items as u64 + result.page_size - 1) / result.page_size) as i32
                } else {
                    0
                };
                Ok(Response::new(ListJournalEntriesResponse {
                    entries: items,
                    pagination: Some(crate::infrastructure::grpc::common::v1::PaginationResponse {
                        total_items: result.total_items,
                        total_pages,
                        current_page: result.page as i32,
                        page_size: result.page_size as i32,
                    })
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn reverse_journal_entry(&self, _request: Request<ReverseJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
         Err(Status::unimplemented("Not implemented"))
    }
    
    async fn batch_reverse_journal_entries(&self, _request: Request<BatchReverseJournalEntriesRequest>) -> Result<Response<BatchReverseJournalEntriesResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn stream_journal_entries(&self, _request: Request<ListJournalEntriesRequest>) -> Result<Response<Self::StreamJournalEntriesStream>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    
    async fn simulate_journal_entry(&self, _request: Request<SimulateJournalEntryRequest>) -> Result<Response<SimulationResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    async fn update_journal_entry(&self, _request: Request<UpdateJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    async fn delete_journal_entry(&self, _request: Request<DeleteJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    async fn post_journal_entry(&self, _request: Request<PostJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    async fn validate_journal_entry(&self, _request: Request<ValidateJournalEntryRequest>) -> Result<Response<ValidationResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    async fn park_journal_entry(&self, _request: Request<ParkJournalEntryRequest>) -> Result<Response<ParkJournalEntryResponse>, Status> {
         Err(Status::unimplemented("Not implemented"))
    }
    async fn batch_create_journal_entries(&self, _request: Request<BatchCreateJournalEntriesRequest>) -> Result<Response<BatchCreateJournalEntriesResponse>, Status> {
         Err(Status::unimplemented("Not implemented"))
    }
    async fn clear_open_items(&self, _request: Request<ClearOpenItemsRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    async fn revaluate_foreign_currency(&self, _request: Request<RevaluateForeignCurrencyRequest>) -> Result<Response<RevaluationResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    async fn get_parallel_ledger_data(&self, _request: Request<GetParallelLedgerDataRequest>) -> Result<Response<ParallelLedgerDataResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    async fn carry_forward_balances(&self, _request: Request<CarryForwardBalancesRequest>) -> Result<Response<CarryForwardBalancesResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    async fn execute_period_end_close(&self, _request: Request<ExecutePeriodEndCloseRequest>) -> Result<Response<PeriodEndCloseResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    async fn create_batch_input_session(&self, _request: Request<CreateBatchInputSessionRequest>) -> Result<Response<BatchInputSessionResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    async fn get_account_line_items(&self, _request: Request<GetAccountLineItemsRequest>) -> Result<Response<AccountLineItemsResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    async fn generate_print_preview(&self, _request: Request<GeneratePrintPreviewRequest>) -> Result<Response<GeneratePrintPreviewResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
}

// Helpers
fn map_to_response(entry: crate::domain::aggregates::journal_entry::JournalEntry) -> JournalEntryResponse {
    let doc_num = entry.document_number.unwrap_or_default();
    JournalEntryResponse {
        success: true,
        document_reference: Some(crate::infrastructure::grpc::common::v1::SystemDocumentReference {
            document_number: doc_num,
            company_code: entry.company_code,
            fiscal_year: entry.fiscal_year,
            ..Default::default()
        }),
        messages: vec![],
    }
}

fn map_to_summary(entry: crate::domain::aggregates::journal_entry::JournalEntry) -> JournalEntrySummary {
     JournalEntrySummary {
        document_reference: Some(crate::infrastructure::grpc::common::v1::SystemDocumentReference {
            document_number: entry.document_number.unwrap_or_default(),
            company_code: entry.company_code,
            fiscal_year: entry.fiscal_year,
            ..Default::default()
        }),
        document_date: Some(prost_types::Timestamp { seconds: entry.document_date.and_hms_opt(0,0,0).unwrap().and_utc().timestamp(), nanos: 0 }),
        posting_date: Some(prost_types::Timestamp { seconds: entry.posting_date.and_hms_opt(0,0,0).unwrap().and_utc().timestamp(), nanos: 0 }),
        header_text: entry.reference.unwrap_or_default(),
        status: match entry.status {
            crate::domain::aggregates::journal_entry::PostingStatus::Draft => JournalEntryStatus::Draft as i32,
            crate::domain::aggregates::journal_entry::PostingStatus::Posted => JournalEntryStatus::Posted as i32,
            crate::domain::aggregates::journal_entry::PostingStatus::Reversed => JournalEntryStatus::Reversed as i32,
        },
        total_amount: None,
     }
}

fn map_to_detail(entry: crate::domain::aggregates::journal_entry::JournalEntry) -> JournalEntryDetail {
    JournalEntryDetail {
        document_reference: Some(crate::infrastructure::grpc::common::v1::SystemDocumentReference {
            document_number: entry.document_number.clone().unwrap_or_default(),
            company_code: entry.company_code.clone(),
            fiscal_year: entry.fiscal_year,
             ..Default::default()
        }),
        header: Some(JournalEntryHeader {
            company_code: entry.company_code,
            fiscal_year: entry.fiscal_year,
            posting_date: Some(prost_types::Timestamp { seconds: entry.posting_date.and_hms_opt(0,0,0).unwrap().and_utc().timestamp(), nanos: 0 }),
            document_date: Some(prost_types::Timestamp { seconds: entry.document_date.and_hms_opt(0,0,0).unwrap().and_utc().timestamp(), nanos: 0 }),
            currency: entry.currency,
            reference_document: entry.reference.unwrap_or_default(),
            header_text: "".to_string(),
            document_type: "SA".to_string(),
            fiscal_period: 1,
            exchange_rate: "1.0".to_string(),
            origin: DocumentOrigin::Api as i32,
            logical_system: "".to_string(),
            ledger_group: "".to_string(),
            audit: None,
        }),
        line_items: entry.lines.into_iter().map(|l| JournalEntryLineItem {
            line_item_number: l.line_number,
            gl_account: l.account_id,
            debit_credit_indicator: l.debit_credit.as_char().to_string(),
            amount_in_document_currency: Some(crate::infrastructure::grpc::common::v1::MonetaryValue {
                value: l.amount.to_string(),
                currency_code: "CNY".to_string(), 
            }),
            amount_in_local_currency: Some(crate::infrastructure::grpc::common::v1::MonetaryValue {
                 value: l.local_amount.to_string(),
                 currency_code: "CNY".to_string(),
            }),
            cost_center: l.cost_center.unwrap_or_default(),
            profit_center: l.profit_center.unwrap_or_default(),
            text: l.text.unwrap_or_default(),
            // Defaults
            posting_key: "".to_string(),
            account_type: crate::infrastructure::grpc::common::v1::AccountType::Gl as i32,
            business_partner: "".to_string(),
            amount_in_group_currency: None,
            segment: "".to_string(),
            internal_order: "".to_string(),
            wbs_element: "".to_string(),
            assignment_number: "".to_string(),
            tax_code: "".to_string(),
            tax_jurisdiction: "".to_string(),
            clearing_document: "".to_string(),
            clearing_date: None,
            quantity: None,
        }).collect(),
        tax_items: vec![],
        status: match entry.status {
            crate::domain::aggregates::journal_entry::PostingStatus::Draft => JournalEntryStatus::Draft as i32,
            crate::domain::aggregates::journal_entry::PostingStatus::Posted => JournalEntryStatus::Posted as i32,
            crate::domain::aggregates::journal_entry::PostingStatus::Reversed => JournalEntryStatus::Reversed as i32,
        },
        clearing_info: None,
        payment_info: None,
        workflow_info: None,
        attachments: vec![],
    }
}
