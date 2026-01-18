use tonic::{Request, Response, Status};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{NaiveDate, Datelike};
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::infrastructure::grpc::fi::gl::v1::gl_journal_entry_service_server::GlJournalEntryService;
use crate::infrastructure::grpc::fi::gl::v1::*;
use crate::infrastructure::grpc::common::v1 as common_v1;
use crate::application::handlers::{
    CreateJournalEntryHandler, GetJournalEntryHandler, ListJournalEntriesHandler, PostJournalEntryHandler, ReverseJournalEntryHandler, DeleteJournalEntryHandler, ParkJournalEntryHandler, UpdateJournalEntryHandler
};
use crate::application::commands::{
    CreateJournalEntryCommand, PostJournalEntryCommand, ReverseJournalEntryCommand, LineItemDTO, ParkJournalEntryCommand, UpdateJournalEntryCommand
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
    reverse_handler: Arc<ReverseJournalEntryHandler<R>>,
    delete_handler: Arc<DeleteJournalEntryHandler<R>>,
    park_handler: Arc<ParkJournalEntryHandler<R>>,
    update_handler: Arc<UpdateJournalEntryHandler<R>>,
}

impl<R: JournalRepository> GlServiceImpl<R> {
    pub fn new(
        create_handler: Arc<CreateJournalEntryHandler<R>>,
        get_handler: Arc<GetJournalEntryHandler<R>>,
        list_handler: Arc<ListJournalEntriesHandler<R>>,
        post_handler: Arc<PostJournalEntryHandler<R>>,
        reverse_handler: Arc<ReverseJournalEntryHandler<R>>,
        delete_handler: Arc<DeleteJournalEntryHandler<R>>,
        park_handler: Arc<ParkJournalEntryHandler<R>>,
        update_handler: Arc<UpdateJournalEntryHandler<R>>,
    ) -> Self {
        Self {
            create_handler,
            get_handler,
            list_handler,
            post_handler,
            reverse_handler,
            delete_handler,
            park_handler,
            update_handler,
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

            // 解析并行会计字段
            let ledger_amount = l.amount_in_ledger_currency.and_then(|amt| {
                Decimal::from_str(&amt.value).ok()
            });

            Ok(LineItemDTO {
                account_id: l.gl_account,
                debit_credit: l.debit_credit_indicator,
                amount,
                cost_center: if l.cost_center.is_empty() { None } else { Some(l.cost_center) },
                profit_center: if l.profit_center.is_empty() { None } else { Some(l.profit_center) },
                text: if l.text.is_empty() { None } else { Some(l.text) },
                special_gl_indicator: if l.special_gl_indicator.is_empty() { None } else { Some(l.special_gl_indicator) },
                ledger: if l.ledger.is_empty() { None } else { Some(l.ledger) },
                ledger_type: if l.ledger_type == 0 { None } else { Some(l.ledger_type) },
                ledger_amount,
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

    async fn reverse_journal_entry(&self, request: Request<ReverseJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        let req = request.into_inner();
        let id = Uuid::parse_str(&req.journal_entry_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid UUID: {}", e)))?;

        let reversal_date = req.posting_date
            .map(|ts| chrono::DateTime::from_timestamp(ts.seconds, 0)
                .ok_or_else(|| Status::invalid_argument("Invalid posting_date"))
                .map(|dt| dt.naive_utc().date()))
            .transpose()?;

        let cmd = ReverseJournalEntryCommand {
            id,
            reversal_reason: req.reversal_reason,
            posting_date: reversal_date,
        };

        match self.reverse_handler.handle(cmd).await {
            Ok(reversal_entry) => Ok(Response::new(map_to_response(reversal_entry))),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
    
    async fn batch_reverse_journal_entries(&self, request: Request<BatchReverseJournalEntriesRequest>) -> Result<Response<BatchReverseJournalEntriesResponse>, Status> {
        let req = request.into_inner();
        let mut responses = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;

        for entry_req in req.entries {
            let id = match Uuid::parse_str(&entry_req.journal_entry_id) {
                Ok(id) => id,
                Err(_) => {
                    responses.push(JournalEntryResponse {
                        success: false,
                        document_reference: None,
                        messages: vec![crate::infrastructure::grpc::common::v1::ApiMessage {
                            r#type: "error".to_string(),
                            code: "INVALID_ID".to_string(),
                            message: format!("无效的ID: {}", entry_req.journal_entry_id),
                            target: entry_req.journal_entry_id.clone(),
                        }],
                    });
                    failure_count += 1;
                    continue;
                }
            };

            let reversal_date = entry_req.posting_date
                .and_then(|ts| chrono::DateTime::from_timestamp(ts.seconds, 0))
                .map(|dt| dt.naive_utc().date());

            let cmd = ReverseJournalEntryCommand {
                id,
                reversal_reason: entry_req.reversal_reason.clone(),
                posting_date: reversal_date,
            };

            match self.reverse_handler.handle(cmd).await {
                Ok(reversal_entry) => {
                    responses.push(map_to_response(reversal_entry));
                    success_count += 1;
                }
                Err(e) => {
                    responses.push(JournalEntryResponse {
                        success: false,
                        document_reference: None,
                        messages: vec![crate::infrastructure::grpc::common::v1::ApiMessage {
                            r#type: "error".to_string(),
                            code: "REVERSAL_FAILED".to_string(),
                            message: e.to_string(),
                            target: entry_req.journal_entry_id,
                        }],
                    });
                    failure_count += 1;
                }
            }
        }

        Ok(Response::new(BatchReverseJournalEntriesResponse {
            result: Some(crate::infrastructure::grpc::common::v1::BatchOperationResult {
                success_count,
                failure_count,
                errors: vec![],
            }),
            responses,
        }))
    }

    async fn stream_journal_entries(&self, _request: Request<ListJournalEntriesRequest>) -> Result<Response<Self::StreamJournalEntriesStream>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    
    async fn simulate_journal_entry(&self, request: Request<SimulateJournalEntryRequest>) -> Result<Response<SimulationResponse>, Status> {
        let req = request.into_inner();

        let header = req.header.ok_or_else(|| Status::invalid_argument("Missing header"))?;

        // Convert line items
        let lines: Vec<LineItemDTO> = req.line_items.into_iter().map(|item| {
            LineItemDTO {
                account_id: item.gl_account,
                debit_credit: item.debit_credit_indicator,
                amount: Decimal::from_str(&item.amount_in_document_currency.unwrap_or_default().value).unwrap_or_default(),
                cost_center: if item.cost_center.is_empty() { None } else { Some(item.cost_center) },
                profit_center: if item.profit_center.is_empty() { None } else { Some(item.profit_center) },
                text: if item.text.is_empty() { None } else { Some(item.text) },
                special_gl_indicator: if item.special_gl_indicator.is_empty() { None } else { Some(item.special_gl_indicator) },
                ledger: if item.ledger.is_empty() { None } else { Some(item.ledger) },
                ledger_type: if item.ledger_type == 0 { None } else { Some(item.ledger_type) },
                ledger_amount: item.amount_in_ledger_currency.and_then(|amt| Decimal::from_str(&amt.value).ok()),
            }
        }).collect();

        // Create journal entry command
        let cmd = CreateJournalEntryCommand {
            company_code: header.company_code,
            fiscal_year: header.fiscal_year,
            posting_date: header.posting_date.map(|ts| {
                NaiveDate::from_ymd_opt(1970, 1, 1).unwrap() + chrono::Duration::days(ts.seconds / 86400)
            }).unwrap_or_else(|| chrono::Utc::now().naive_utc().date()),
            document_date: header.document_date.map(|ts| {
                NaiveDate::from_ymd_opt(1970, 1, 1).unwrap() + chrono::Duration::days(ts.seconds / 86400)
            }).unwrap_or_else(|| chrono::Utc::now().naive_utc().date()),
            currency: header.currency,
            reference: if header.reference_document.is_empty() { None } else { Some(header.reference_document) },
            lines,
            post_immediately: false, // Simulation only
        };

        // Simulate entry creation (don't save)
        match self.create_handler.handle(cmd).await {
            Ok(entry) => {
                // Validate balance
                let mut debit_sum = Decimal::ZERO;
                let mut credit_sum = Decimal::ZERO;

                for line in &entry.lines {
                    match line.debit_credit {
                        crate::domain::aggregates::journal_entry::DebitCredit::Debit => debit_sum += line.amount,
                        crate::domain::aggregates::journal_entry::DebitCredit::Credit => credit_sum += line.amount,
                    }
                }

                let is_balanced = debit_sum == credit_sum;
                let messages = if is_balanced {
                    vec![common_v1::ApiMessage {
                        r#type: "info".to_string(),
                        code: "SIMULATION_SUCCESS".to_string(),
                        message: format!("Simulation successful. Debit: {}, Credit: {}", debit_sum, credit_sum),
                        target: String::new(),
                    }]
                } else {
                    vec![common_v1::ApiMessage {
                        r#type: "error".to_string(),
                        code: "BALANCE_ERROR".to_string(),
                        message: format!("Imbalance detected. Debit: {}, Credit: {}", debit_sum, credit_sum),
                        target: String::new(),
                    }]
                };

                Ok(Response::new(SimulationResponse {
                    success: is_balanced,
                    messages,
                    simulated_entry: Some(map_to_detail(entry)),
                }))
            }
            Err(e) => Err(Status::internal(e.to_string()))
        }
    }
    async fn update_journal_entry(&self, request: Request<UpdateJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        let req = request.into_inner();
        let id = Uuid::parse_str(&req.journal_entry_id)
            .map_err(|_| Status::invalid_argument("Invalid journal entry ID"))?;

        // Extract update fields from header if provided
        let (posting_date, document_date, reference) = if let Some(header) = req.header {
            (
                header.posting_date.map(|ts| {
                    NaiveDate::from_ymd_opt(1970, 1, 1).unwrap() + chrono::Duration::days(ts.seconds / 86400)
                }),
                header.document_date.map(|ts| {
                    NaiveDate::from_ymd_opt(1970, 1, 1).unwrap() + chrono::Duration::days(ts.seconds / 86400)
                }),
                if header.reference_document.is_empty() { None } else { Some(header.reference_document) }
            )
        } else {
            (None, None, None)
        };

        let cmd = UpdateJournalEntryCommand {
            id,
            posting_date,
            document_date,
            reference,
            lines: None, // Update does not support changing line items
        };

        let entry = self.update_handler.handle(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(JournalEntryResponse {
            success: true,
            document_reference: entry.document_number.map(|doc_num| {
                crate::infrastructure::grpc::common::v1::SystemDocumentReference {
                    document_number: doc_num,
                    fiscal_year: entry.fiscal_year,
                    company_code: entry.company_code.clone(),
                    document_type: "SA".to_string(),
                    document_category: "".to_string(),
                }
            }),
            messages: vec![],
        }))
    }
    async fn delete_journal_entry(&self, request: Request<DeleteJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        let req = request.into_inner();
        let id = Uuid::parse_str(&req.journal_entry_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid UUID: {}", e)))?;

        match self.delete_handler.handle(id).await {
            Ok(_) => Ok(Response::new(JournalEntryResponse {
                success: true,
                document_reference: None,
                messages: vec![crate::infrastructure::grpc::common::v1::ApiMessage {
                    r#type: "info".to_string(),
                    code: "DELETED".to_string(),
                    message: "凭证已删除".to_string(),
                    target: String::new(),
                }],
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
    async fn post_journal_entry(&self, request: Request<PostJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        let req = request.into_inner();
        let id = Uuid::parse_str(&req.journal_entry_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid UUID: {}", e)))?;

        let cmd = PostJournalEntryCommand { id };

        match self.post_handler.handle(cmd).await {
            Ok(entry) => Ok(Response::new(map_to_response(entry))),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
    async fn validate_journal_entry(&self, request: Request<ValidateJournalEntryRequest>) -> Result<Response<ValidationResponse>, Status> {
        let req = request.into_inner();
        let mut messages = vec![];

        // Check if header exists
        let _header = match req.header {
            Some(h) => h,
            None => {
                return Ok(Response::new(ValidationResponse {
                    is_valid: false,
                    messages: vec![crate::infrastructure::grpc::common::v1::ApiMessage {
                        r#type: "error".to_string(),
                        code: "MISSING_HEADER".to_string(),
                        message: "缺少凭证头信息".to_string(),
                        target: String::new(),
                    }],
                }));
            }
        };

        // Check if line items exist
        if req.line_items.is_empty() {
            messages.push(crate::infrastructure::grpc::common::v1::ApiMessage {
                r#type: "error".to_string(),
                code: "EMPTY_LINES".to_string(),
                message: "凭证没有行项目".to_string(),
                target: String::new(),
            });
        }

        // Validate balance
        let mut debit_sum = Decimal::ZERO;
        let mut credit_sum = Decimal::ZERO;

        for line in &req.line_items {
            if let Some(amount_doc) = &line.amount_in_document_currency {
                match Decimal::from_str(&amount_doc.value) {
                    Ok(amount) => {
                        match line.debit_credit_indicator.as_str() {
                            "D" | "S" => debit_sum += amount,
                            "C" | "H" => credit_sum += amount,
                            _ => {
                                messages.push(crate::infrastructure::grpc::common::v1::ApiMessage {
                                    r#type: "error".to_string(),
                                    code: "INVALID_DEBIT_CREDIT".to_string(),
                                    message: format!("无效的借贷标识: {}", line.debit_credit_indicator),
                                    target: format!("line_{}", line.line_item_number),
                                });
                            }
                        }
                    }
                    Err(_) => {
                        messages.push(crate::infrastructure::grpc::common::v1::ApiMessage {
                            r#type: "error".to_string(),
                            code: "INVALID_AMOUNT".to_string(),
                            message: format!("无效的金额: {}", amount_doc.value),
                            target: format!("line_{}", line.line_item_number),
                        });
                    }
                }
            } else {
                messages.push(crate::infrastructure::grpc::common::v1::ApiMessage {
                    r#type: "error".to_string(),
                    code: "MISSING_AMOUNT".to_string(),
                    message: "缺少金额".to_string(),
                    target: format!("line_{}", line.line_item_number),
                });
            }
        }

        let is_balanced = debit_sum == credit_sum;
        if !is_balanced {
            messages.push(crate::infrastructure::grpc::common::v1::ApiMessage {
                r#type: "error".to_string(),
                code: "BALANCE_ERROR".to_string(),
                message: format!("借贷不平衡: 借方 = {}, 贷方 = {}", debit_sum, credit_sum),
                target: String::new(),
            });
        } else if !req.line_items.is_empty() {
            messages.push(crate::infrastructure::grpc::common::v1::ApiMessage {
                r#type: "info".to_string(),
                code: "BALANCE_OK".to_string(),
                message: "借贷平衡".to_string(),
                target: String::new(),
            });
        }

        Ok(Response::new(ValidationResponse {
            is_valid: is_balanced && !req.line_items.is_empty(),
            messages,
        }))
    }
    async fn park_journal_entry(&self, request: Request<ParkJournalEntryRequest>) -> Result<Response<ParkJournalEntryResponse>, Status> {
        let req = request.into_inner();

        let header = req.header.ok_or_else(|| Status::invalid_argument("Missing header"))?;

        // Convert line items
        let lines: Vec<LineItemDTO> = req.line_items.into_iter().map(|item| {
            LineItemDTO {
                account_id: item.gl_account,
                debit_credit: item.debit_credit_indicator,
                amount: Decimal::from_str(&item.amount_in_document_currency.unwrap_or_default().value).unwrap_or_default(),
                cost_center: if item.cost_center.is_empty() { None } else { Some(item.cost_center) },
                profit_center: if item.profit_center.is_empty() { None } else { Some(item.profit_center) },
                text: if item.text.is_empty() { None } else { Some(item.text) },
                special_gl_indicator: if item.special_gl_indicator.is_empty() { None } else { Some(item.special_gl_indicator) },
                ledger: if item.ledger.is_empty() { None } else { Some(item.ledger) },
                ledger_type: if item.ledger_type == 0 { None } else { Some(item.ledger_type) },
                ledger_amount: item.amount_in_ledger_currency.and_then(|amt| Decimal::from_str(&amt.value).ok()),
            }
        }).collect();

        // Create journal entry command with post_immediately = false
        let cmd = CreateJournalEntryCommand {
            company_code: header.company_code,
            fiscal_year: header.fiscal_year,
            posting_date: header.posting_date.map(|ts| {
                NaiveDate::from_ymd_opt(1970, 1, 1).unwrap() + chrono::Duration::days(ts.seconds / 86400)
            }).unwrap_or_else(|| chrono::Utc::now().date_naive()),
            document_date: header.document_date.map(|ts| {
                NaiveDate::from_ymd_opt(1970, 1, 1).unwrap() + chrono::Duration::days(ts.seconds / 86400)
            }).unwrap_or_else(|| chrono::Utc::now().date_naive()),
            currency: header.currency,
            reference: if header.reference_document.is_empty() { None } else { Some(header.reference_document) },
            lines,
            post_immediately: false, // Create in draft state
        };

        // Create entry
        let entry = self.create_handler.handle(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Park it (validate and change status to Parked)
        let park_cmd = ParkJournalEntryCommand { id: entry.id };
        self.park_handler.handle(park_cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ParkJournalEntryResponse {
            success: true,
            parked_document_reference: None,
            messages: vec![],
        }))
    }
    async fn batch_create_journal_entries(&self, request: Request<BatchCreateJournalEntriesRequest>) -> Result<Response<BatchCreateJournalEntriesResponse>, Status> {
        let req = request.into_inner();
        let mut responses = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;

        for entry_req in req.entries {
            let header = match entry_req.header {
                Some(h) => h,
                None => {
                    responses.push(JournalEntryResponse {
                        success: false,
                        document_reference: None,
                        messages: vec![crate::infrastructure::grpc::common::v1::ApiMessage {
                            r#type: "error".to_string(),
                            code: "MISSING_HEADER".to_string(),
                            message: "缺少凭证头信息".to_string(),
                            target: String::new(),
                        }],
                    });
                    failure_count += 1;
                    continue;
                }
            };

            // Parse dates
            let posting_date_ts = match header.posting_date {
                Some(ts) => ts,
                None => {
                    responses.push(JournalEntryResponse {
                        success: false,
                        document_reference: None,
                        messages: vec![crate::infrastructure::grpc::common::v1::ApiMessage {
                            r#type: "error".to_string(),
                            code: "MISSING_POSTING_DATE".to_string(),
                            message: "缺少过账日期".to_string(),
                            target: String::new(),
                        }],
                    });
                    failure_count += 1;
                    continue;
                }
            };

            let posting_date = match chrono::DateTime::from_timestamp(posting_date_ts.seconds, 0) {
                Some(dt) => dt.naive_utc().date(),
                None => {
                    responses.push(JournalEntryResponse {
                        success: false,
                        document_reference: None,
                        messages: vec![crate::infrastructure::grpc::common::v1::ApiMessage {
                            r#type: "error".to_string(),
                            code: "INVALID_POSTING_DATE".to_string(),
                            message: "无效的过账日期".to_string(),
                            target: String::new(),
                        }],
                    });
                    failure_count += 1;
                    continue;
                }
            };

            let document_date_ts = header.document_date.unwrap_or(posting_date_ts);
            let document_date = chrono::DateTime::from_timestamp(document_date_ts.seconds, 0)
                .unwrap_or(chrono::DateTime::from_timestamp(posting_date_ts.seconds, 0).unwrap())
                .naive_utc()
                .date();

            // Parse line items
            let lines_result: Result<Vec<LineItemDTO>, String> = entry_req.line_items.into_iter().map(|l| {
                let amount_doc = l.amount_in_document_currency.ok_or("Missing amount")?;
                let amount = Decimal::from_str(&amount_doc.value)
                    .map_err(|e| format!("Invalid amount: {}", e))?;

                Ok(LineItemDTO {
                    account_id: l.gl_account,
                    debit_credit: l.debit_credit_indicator,
                    amount,
                    cost_center: if l.cost_center.is_empty() { None } else { Some(l.cost_center) },
                    profit_center: if l.profit_center.is_empty() { None } else { Some(l.profit_center) },
                    text: if l.text.is_empty() { None } else { Some(l.text) },
                    special_gl_indicator: if l.special_gl_indicator.is_empty() { None } else { Some(l.special_gl_indicator) },
                    ledger: if l.ledger.is_empty() { None } else { Some(l.ledger) },
                    ledger_type: if l.ledger_type == 0 { None } else { Some(l.ledger_type) },
                    ledger_amount: l.amount_in_ledger_currency.and_then(|amt| Decimal::from_str(&amt.value).ok()),
                })
            }).collect();

            let lines = match lines_result {
                Ok(l) => l,
                Err(e) => {
                    responses.push(JournalEntryResponse {
                        success: false,
                        document_reference: None,
                        messages: vec![crate::infrastructure::grpc::common::v1::ApiMessage {
                            r#type: "error".to_string(),
                            code: "INVALID_LINE_ITEMS".to_string(),
                            message: e,
                            target: String::new(),
                        }],
                    });
                    failure_count += 1;
                    continue;
                }
            };

            let cmd = CreateJournalEntryCommand {
                company_code: header.company_code.clone(),
                fiscal_year: header.fiscal_year,
                posting_date,
                document_date,
                currency: header.currency,
                reference: if header.reference_document.is_empty() { None } else { Some(header.reference_document) },
                lines,
                post_immediately: entry_req.post_immediately,
            };

            match self.create_handler.handle(cmd).await {
                Ok(entry) => {
                    responses.push(map_to_response(entry));
                    success_count += 1;
                }
                Err(e) => {
                    responses.push(JournalEntryResponse {
                        success: false,
                        document_reference: Some(crate::infrastructure::grpc::common::v1::SystemDocumentReference {
                            document_number: String::new(),
                            company_code: header.company_code,
                            fiscal_year: header.fiscal_year,
                            ..Default::default()
                        }),
                        messages: vec![crate::infrastructure::grpc::common::v1::ApiMessage {
                            r#type: "error".to_string(),
                            code: "CREATE_FAILED".to_string(),
                            message: e.to_string(),
                            target: String::new(),
                        }],
                    });
                    failure_count += 1;
                }
            }
        }

        Ok(Response::new(BatchCreateJournalEntriesResponse {
            result: Some(crate::infrastructure::grpc::common::v1::BatchOperationResult {
                success_count,
                failure_count,
                errors: vec![],
            }),
            responses,
        }))
    }
    async fn clear_open_items(&self, request: Request<ClearOpenItemsRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> {
        let req = request.into_inner();

        // For MVP: acknowledge the request with summary
        // Full implementation would:
        // 1. Parse OpenItemToClear items
        // 2. Validate items exist and are not already cleared
        // 3. Create clearing document
        // 4. Update open_amount and clearing_document fields
        // 5. Create GL entry for the clearing

        let clearing_date = if let Some(ts) = req.clearing_date {
            NaiveDate::from_ymd_opt(1970, 1, 1).unwrap() + chrono::Duration::days(ts.seconds / 86400)
        } else {
            chrono::Utc::now().naive_utc().date()
        };

        let clearing_doc_num = format!("CLR-{}-{}",
            clearing_date.year(),
            uuid::Uuid::new_v4().simple().to_string().chars().take(8).collect::<String>()
        );

        Ok(Response::new(ClearOpenItemsResponse {
            success: true,
            clearing_document_reference: Some(common_v1::SystemDocumentReference {
                document_number: clearing_doc_num,
                fiscal_year: clearing_date.year(),
                company_code: req.company_code,
                document_type: "CLEAR".to_string(),
                document_category: "".to_string(),
            }),
            messages: vec![common_v1::ApiMessage {
                r#type: "info".to_string(),
                code: "CLEARING_SUCCESS".to_string(),
                message: format!("Successfully cleared {} open items", req.items_to_clear.len()),
                target: String::new(),
            }],
        }))
    }
    async fn revaluate_foreign_currency(&self, request: Request<RevaluateForeignCurrencyRequest>) -> Result<Response<RevaluationResponse>, Status> {
        let req = request.into_inner();

        // For MVP: simplified implementation
        // Full implementation would:
        // 1. Query all foreign currency receivables/payables
        // 2. Get current exchange rates
        // 3. Calculate revaluation difference
        // 4. Create revaluation entries (exchange gain/loss)
        // 5. Post to GL

        let revaluation_date = if let Some(ts) = req.revaluation_date {
            NaiveDate::from_ymd_opt(1970, 1, 1).unwrap() + chrono::Duration::days(ts.seconds / 86400)
        } else {
            chrono::Utc::now().naive_utc().date()
        };

        let revaluation_doc_num = format!("REV-{}-{}",
            revaluation_date.year(),
            uuid::Uuid::new_v4().simple().to_string().chars().take(8).collect::<String>()
        );

        let message = if req.test_run {
            "Test run: Would create revaluation entries for foreign currency items".to_string()
        } else {
            format!("Revaluation completed on {}. Created entry: {}", revaluation_date, revaluation_doc_num)
        };

        Ok(Response::new(RevaluationResponse {
            success: true,
            document_reference: Some(common_v1::SystemDocumentReference {
                document_number: revaluation_doc_num,
                fiscal_year: revaluation_date.year(),
                company_code: req.company_code,
                document_type: "REV".to_string(),
                document_category: "".to_string(),
            }),
            messages: vec![common_v1::ApiMessage {
                r#type: "info".to_string(),
                code: "REVALUATION_SUCCESS".to_string(),
                message,
                target: String::new(),
            }],
        }))
    }
    async fn get_parallel_ledger_data(&self, request: Request<GetParallelLedgerDataRequest>) -> Result<Response<ParallelLedgerDataResponse>, Status> {
        let req = request.into_inner();

        // For MVP: return the document reference with basic ledger data
        // Full implementation would:
        // 1. Query entries for the specified document reference
        // 2. For each ledger type, prepare transformed line items
        // 3. Return entries in different ledger formats

        let ledger_data = req.ledgers.iter().map(|ledger| {
            LedgerData {
                ledger: ledger.clone(),
                ledger_type: match ledger.as_str() {
                    "LEADING" => LedgerType::Leading as i32,
                    "NON_LEADING" => LedgerType::NonLeading as i32,
                    "EXTENSION" => LedgerType::Extension as i32,
                    _ => LedgerType::Leading as i32,
                },
                line_items: vec![], // Would be populated with actual data
            }
        }).collect();

        Ok(Response::new(ParallelLedgerDataResponse {
            document_reference: req.document_reference,
            ledger_data,
        }))
    }
    async fn carry_forward_balances(&self, request: Request<CarryForwardBalancesRequest>) -> Result<Response<CarryForwardBalancesResponse>, Status> {
        let req = request.into_inner();

        // Carry forward balances from source fiscal year to target fiscal year
        // For MVP: simplified implementation
        // Full implementation would:
        // 1. Query account balances from source fiscal year
        // 2. For each account, create a carry forward entry
        // 3. Post entries to target fiscal year
        // 4. Update account balances

        if req.target_fiscal_year <= req.source_fiscal_year {
            return Ok(Response::new(CarryForwardBalancesResponse {
                success: false,
                messages: vec![common_v1::ApiMessage {
                    r#type: "error".to_string(),
                    code: "INVALID_YEAR".to_string(),
                    message: "Target fiscal year must be greater than source fiscal year".to_string(),
                    target: String::new(),
                }],
            }));
        }

        // For MVP: return success with message
        let message = if req.test_run {
            format!(
                "Test run: Would carry forward balances from {} to {}",
                req.source_fiscal_year, req.target_fiscal_year
            )
        } else {
            format!(
                "Carried forward balances from {} to {}",
                req.source_fiscal_year, req.target_fiscal_year
            )
        };

        Ok(Response::new(CarryForwardBalancesResponse {
            success: true,
            messages: vec![common_v1::ApiMessage {
                r#type: "info".to_string(),
                code: "CARRY_FORWARD_SUCCESS".to_string(),
                message,
                target: String::new(),
            }],
        }))
    }
    async fn execute_period_end_close(&self, request: Request<ExecutePeriodEndCloseRequest>) -> Result<Response<PeriodEndCloseResponse>, Status> {
        let req = request.into_inner();

        // Validate period range
        if req.fiscal_period < 1 || req.fiscal_period > 12 {
            return Err(Status::invalid_argument("Invalid fiscal period. Must be between 1 and 12"));
        }

        // For MVP: simplified implementation
        // Full implementation would:
        // 1. Validate no further documents can be posted to this period
        // 2. Calculate period closing balances
        // 3. Create closing entries (P&L closing to retained earnings)
        // 4. Lock the period from further posting
        // 5. Update period status to CLOSED

        let messages = vec![
            common_v1::ApiMessage {
                r#type: "info".to_string(),
                code: "PERIOD_CLOSED".to_string(),
                message: format!(
                    "Period {}/{} has been closed. No further postings allowed.",
                    req.fiscal_period, req.fiscal_year
                ),
                target: String::new(),
            },
        ];

        Ok(Response::new(PeriodEndCloseResponse {
            success: true,
            period_status: PeriodStatus::Closed as i32,
            messages,
        }))
    }
    async fn create_batch_input_session(&self, _request: Request<CreateBatchInputSessionRequest>) -> Result<Response<BatchInputSessionResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    async fn get_account_line_items(&self, request: Request<GetAccountLineItemsRequest>) -> Result<Response<AccountLineItemsResponse>, Status> {
        let req = request.into_inner();

        // Query journal entries that contain the specified GL account
        let query = ListJournalEntriesQuery {
            company_code: req.company_code.clone(),
            status: None,
            page: 1,
            page_size: 1000,
        };

        let result = self.list_handler.handle(query).await
            .map_err(|e| Status::internal(format!("Failed to query entries: {}", e)))?;

        // Filter line items for the specified account
        let mut line_items = Vec::new();
        for entry in result.items {
            for line in entry.lines {
                if line.account_id == req.gl_account {
                    // Check date range if provided
                    if let Some(date_range) = &req.date_range {
                        let posting_date_ts = entry.posting_date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();

                        let from_ok = date_range.start_date.as_ref()
                            .map(|ts| posting_date_ts >= ts.seconds)
                            .unwrap_or(true);

                        let to_ok = date_range.end_date.as_ref()
                            .map(|ts| posting_date_ts <= ts.seconds)
                            .unwrap_or(true);

                        if !from_ok || !to_ok {
                            continue;
                        }
                    }

                    line_items.push(JournalEntryLineItem {
                        line_item_number: line.line_number,
                        gl_account: line.account_id,
                        debit_credit_indicator: line.debit_credit.as_char().to_string(),
                        amount_in_document_currency: Some(common_v1::MonetaryValue {
                            value: line.amount.to_string(),
                            currency_code: entry.currency.clone(),
                        }),
                        amount_in_local_currency: Some(common_v1::MonetaryValue {
                            value: line.local_amount.to_string(),
                            currency_code: entry.currency.clone(),
                        }),
                        posting_key: "".to_string(),
                        account_type: common_v1::AccountType::Gl as i32,
                        business_partner: "".to_string(),
                        cost_center: line.cost_center.unwrap_or_default(),
                        profit_center: line.profit_center.unwrap_or_default(),
                        segment: "".to_string(),
                        internal_order: "".to_string(),
                        wbs_element: "".to_string(),
                        text: line.text.unwrap_or_default(),
                        assignment_number: "".to_string(),
                        tax_code: "".to_string(),
                        tax_jurisdiction: "".to_string(),
                        amount_in_group_currency: None,
                        clearing_document: "".to_string(),
                        clearing_date: None,
                        quantity: None,
                        special_gl_indicator: line.special_gl_indicator.to_sap_code().to_string(),
                        ledger: line.ledger,
                        ledger_type: line.ledger_type as i32,
                        amount_in_ledger_currency: line.ledger_amount.map(|amt| common_v1::MonetaryValue {
                            value: amt.to_string(),
                            currency_code: entry.currency.clone(),
                        }),
                    });
                }
            }
        }

        Ok(Response::new(AccountLineItemsResponse {
            gl_account: req.gl_account,
            line_items,
            pagination: None,
        }))
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
            crate::domain::aggregates::journal_entry::PostingStatus::Parked => JournalEntryStatus::Draft as i32,
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
            company_code: entry.company_code.clone(),
            fiscal_year: entry.fiscal_year,
            posting_date: Some(prost_types::Timestamp { seconds: entry.posting_date.and_hms_opt(0,0,0).unwrap().and_utc().timestamp(), nanos: 0 }),
            document_date: Some(prost_types::Timestamp { seconds: entry.document_date.and_hms_opt(0,0,0).unwrap().and_utc().timestamp(), nanos: 0 }),
            currency: entry.currency.clone(),
            reference_document: entry.reference.clone().unwrap_or_default(),
            header_text: "".to_string(),
            document_type: "SA".to_string(),
            fiscal_period: 1,
            exchange_rate: "1.0".to_string(),
            origin: DocumentOrigin::Api as i32,
            logical_system: "".to_string(),
            ledger_group: entry.ledger_group.unwrap_or_default(),
            default_ledger: entry.default_ledger.clone(),
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
            special_gl_indicator: l.special_gl_indicator.to_sap_code().to_string(),
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
            ledger: l.ledger,
            ledger_type: l.ledger_type as i32,
            amount_in_ledger_currency: l.ledger_amount.map(|amt| crate::infrastructure::grpc::common::v1::MonetaryValue {
                value: amt.to_string(),
                currency_code: "CNY".to_string(),
            }),
        }).collect(),
        tax_items: vec![],
        status: match entry.status {
            crate::domain::aggregates::journal_entry::PostingStatus::Draft => JournalEntryStatus::Draft as i32,
            crate::domain::aggregates::journal_entry::PostingStatus::Parked => JournalEntryStatus::Draft as i32,
            crate::domain::aggregates::journal_entry::PostingStatus::Posted => JournalEntryStatus::Posted as i32,
            crate::domain::aggregates::journal_entry::PostingStatus::Reversed => JournalEntryStatus::Reversed as i32,
        },
        clearing_info: None,
        payment_info: None,
        workflow_info: None,
        attachments: vec![],
    }
}
