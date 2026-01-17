// gRPC Server implementation - gRPC 服务实现
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::application::CoaApplicationService;

use crate::infrastructure::grpc::{
    self as proto, chart_of_accounts_service_server::ChartOfAccountsService,
    AccountGroupResponse, AccountHierarchyResponse, BatchCreateGlAccountsRequest,
    BatchCreateGlAccountsResponse, CreateAccountGroupRequest, CreateGlAccountRequest, DeleteGlAccountRequest,
    ExportAccountsRequest, ExportAccountsResponse, GetAccountHierarchyRequest,
    GetAccountPathRequest, GetAccountPathResponse, GetGlAccountRequest, GlAccountDetail,
    GlAccountResponse, ImportAccountsRequest, ImportAccountsResponse, ListAccountGroupsRequest,
    ListAccountGroupsResponse, ListChildAccountsRequest, ListChildAccountsResponse,
    ListGlAccountsRequest, ListGlAccountsResponse,
    UpdateGlAccountRequest, ValidateGlAccountRequest, ValidateGlAccountResponse,
    BatchValidateGlAccountsRequest, BatchValidateGlAccountsResponse,
    BatchUpdateGlAccountsRequest, BatchUpdateGlAccountsResponse,
    CheckAccountPostableRequest, CheckAccountPostableResponse,
};

pub struct CoaGrpcService {
    app_service: Arc<CoaApplicationService>,
}

impl CoaGrpcService {
    pub fn new(app_service: Arc<CoaApplicationService>) -> Self {
        Self { app_service }
    }
}

#[tonic::async_trait]
impl ChartOfAccountsService for CoaGrpcService {
    async fn create_gl_account(
        &self,
        request: Request<CreateGlAccountRequest>,
    ) -> Result<Response<GlAccountResponse>, Status> {
        let req = request.into_inner();
        let account_data = req.account.ok_or_else(|| Status::invalid_argument("account is required"))?;

        // Convert proto to domain model
        let account = crate::domain::GlAccount::new(
            account_data.chart_of_accounts,
            account_data.account_code,
            account_data.account_name,
            convert_account_nature(account_data.account_nature),
            account_data.account_category.to_string(),
        );

        match self.app_service.create_account(account).await {
            Ok(account_code) => Ok(Response::new(GlAccountResponse {
                success: true,
                account_code,
                messages: vec![],
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_gl_account(
        &self,
        request: Request<GetGlAccountRequest>,
    ) -> Result<Response<GlAccountDetail>, Status> {
        let req = request.into_inner();

        match self.app_service.get_account(&req.chart_of_accounts, &req.account_code).await {
            Ok(Some(account)) => {
                // Convert domain model to proto
                let master_data = Some(proto::GlAccountMaster {
                    chart_of_accounts: account.chart_code,
                    account_code: account.account_code,
                    account_name: account.account_name,
                    account_name_long: account.account_name_long.unwrap_or_default(),
                    account_nature: proto_account_nature(&account.account_nature) as i32,
                    account_category: proto_account_category(&account.account_category) as i32,
                    account_group: account.account_group.unwrap_or_default(),
                    account_level: proto_account_level(account.account_level) as i32,
                    parent_account: account.parent_account.unwrap_or_default(),
                    is_leaf_account: account.is_leaf_account,
                    is_postable: account.is_postable,
                    is_cost_element: account.is_cost_element,
                    line_item_display: account.line_item_display,
                    open_item_management: account.open_item_management,
                    balance_indicator: proto_balance_indicator(&account.balance_indicator) as i32,
                    currency: account.currency.unwrap_or_default(),
                    only_local_currency: account.only_local_currency,
                    exchange_rate_diff: account.exchange_rate_diff,
                    tax_relevant: account.tax_relevant,
                    tax_category: account.tax_category.unwrap_or_default(),
                    status: proto_account_status(&account.status) as i32,
                    valid_from: account.valid_from.map(|dt| prost_types::Timestamp {
                        seconds: dt.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                        nanos: 0,
                    }),
                    valid_to: account.valid_to.map(|dt| prost_types::Timestamp {
                        seconds: dt.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                        nanos: 0,
                    }),
                    texts: vec![],
                    controls: vec![],
                    audit: None,
                });

                Ok(Response::new(GlAccountDetail {
                    master_data,
                    company_code_data: vec![],
                    hierarchy_info: None,
                    child_accounts: vec![],
                }))
            }
            Ok(None) => Err(Status::not_found("Account not found")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn update_gl_account(
        &self,
        request: Request<UpdateGlAccountRequest>,
    ) -> Result<Response<GlAccountResponse>, Status> {
        let req = request.into_inner();
        let account_data = req.account.ok_or_else(|| Status::invalid_argument("account is required"))?;

        // 获取现有科目
        let existing = self.app_service
            .get_account(&account_data.chart_of_accounts, &req.account_code)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Account not found"))?;

        // 创建更新后的科目
        let mut account = existing;
        account.account_name = account_data.account_name;
        account.account_name_long = if account_data.account_name_long.is_empty() {
            None
        } else {
            Some(account_data.account_name_long)
        };
        account.account_group = if account_data.account_group.is_empty() {
            None
        } else {
            Some(account_data.account_group)
        };
        account.is_postable = account_data.is_postable;
        account.is_cost_element = account_data.is_cost_element;
        account.line_item_display = account_data.line_item_display;
        account.open_item_management = account_data.open_item_management;
        account.tax_relevant = account_data.tax_relevant;
        account.tax_category = if account_data.tax_category.is_empty() {
            None
        } else {
            Some(account_data.tax_category)
        };
        account.currency = if account_data.currency.is_empty() {
            None
        } else {
            Some(account_data.currency)
        };
        account.only_local_currency = account_data.only_local_currency;
        account.exchange_rate_diff = account_data.exchange_rate_diff;
        account.changed_at = Some(chrono::Utc::now());

        match self.app_service.update_account(account).await {
            Ok(_) => Ok(Response::new(GlAccountResponse {
                success: true,
                account_code: req.account_code,
                messages: vec![],
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn delete_gl_account(
        &self,
        request: Request<DeleteGlAccountRequest>,
    ) -> Result<Response<GlAccountResponse>, Status> {
        let req = request.into_inner();

        match self.app_service.delete_account(&req.chart_of_accounts, &req.account_code, req.soft_delete).await {
            Ok(_) => Ok(Response::new(GlAccountResponse {
                success: true,
                account_code: req.account_code,
                messages: vec![],
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn list_gl_accounts(
        &self,
        request: Request<ListGlAccountsRequest>,
    ) -> Result<Response<ListGlAccountsResponse>, Status> {
        let req = request.into_inner();

        match self.app_service.list_accounts(&req.chart_of_accounts).await {
            Ok(accounts) => {
                let summaries = accounts
                    .into_iter()
                    .map(|a| proto::GlAccountSummary {
                        account_code: a.account_code,
                        account_name: a.account_name,
                        account_nature: proto_account_nature(&a.account_nature) as i32,
                        account_level: proto_account_level(a.account_level) as i32,
                        is_postable: a.is_postable,
                        status: proto_account_status(&a.status) as i32,
                        parent_account: a.parent_account.unwrap_or_default(),
                    })
                    .collect();

                Ok(Response::new(ListGlAccountsResponse {
                    accounts: summaries,
                    pagination: None,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn validate_gl_account(
        &self,
        request: Request<ValidateGlAccountRequest>,
    ) -> Result<Response<ValidateGlAccountResponse>, Status> {
        let req = request.into_inner();
        let posting_date = chrono::Utc::now().naive_utc().date();

        match self
            .app_service
            .validate_account(&req.chart_of_accounts, &req.account_code, posting_date)
            .await
        {
            Ok(result) => Ok(Response::new(ValidateGlAccountResponse {
                result: Some(proto::AccountValidationResult {
                    account_code: req.account_code,
                    is_valid: result.is_valid,
                    exists: result.exists,
                    is_active: result.is_active,
                    is_postable: result.is_postable,
                    status: if result.is_valid {
                        proto::AccountStatus::Active as i32
                    } else if !result.exists {
                        proto::AccountStatus::Inactive as i32
                    } else {
                        proto::AccountStatus::Blocked as i32
                    },
                    messages: result
                        .error_message
                        .map(|msg| vec![proto::common::v1::ApiMessage {
                            r#type: "error".to_string(),
                            code: "VALIDATION_ERROR".to_string(),
                            message: msg,
                            target: String::new(),
                        }])
                        .unwrap_or_default(),
                }),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn batch_validate_gl_accounts(
        &self,
        request: Request<BatchValidateGlAccountsRequest>,
    ) -> Result<Response<BatchValidateGlAccountsResponse>, Status> {
        let req = request.into_inner();
        let posting_date = chrono::Utc::now().naive_utc().date();

        let account_codes: Vec<String> = req.account_codes.clone();

        match self
            .app_service
            .batch_validate_accounts(&req.chart_of_accounts, account_codes, posting_date)
            .await
        {
            Ok(results) => {
                let total_count = results.len() as i32;
                let valid_count = results.iter().filter(|r| r.is_valid).count() as i32;
                let invalid_count = total_count - valid_count;

                let validation_results: Vec<proto::AccountValidationResult> = results
                    .into_iter()
                    .zip(req.account_codes.iter())
                    .map(|(r, code)| proto::AccountValidationResult {
                        account_code: code.clone(),
                        is_valid: r.is_valid,
                        exists: r.exists,
                        is_active: r.is_active,
                        is_postable: r.is_postable,
                        status: if r.is_valid {
                            proto::AccountStatus::Active as i32
                        } else if !r.exists {
                            proto::AccountStatus::Inactive as i32
                        } else {
                            proto::AccountStatus::Blocked as i32
                        },
                        messages: r
                            .error_message
                            .map(|msg| vec![proto::common::v1::ApiMessage {
                                r#type: "error".to_string(),
                                code: "VALIDATION_ERROR".to_string(),
                                message: msg,
                                target: String::new(),
                            }])
                            .unwrap_or_default(),
                    })
                    .collect();

                Ok(Response::new(BatchValidateGlAccountsResponse {
                    results: validation_results,
                    total_count,
                    valid_count,
                    invalid_count,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn check_account_postable(
        &self,
        request: Request<CheckAccountPostableRequest>,
    ) -> Result<Response<CheckAccountPostableResponse>, Status> {
        let req = request.into_inner();

        // 解析过账日期，如果没有提供则使用当前日期
        let posting_date = req.posting_date
            .map(|ts| chrono::NaiveDateTime::from_timestamp_opt(ts.seconds, 0)
                .map(|dt| dt.date())
                .unwrap_or_else(|| chrono::Utc::now().naive_utc().date()))
            .unwrap_or_else(|| chrono::Utc::now().naive_utc().date());

        match self
            .app_service
            .validate_account(&req.chart_of_accounts, &req.account_code, posting_date)
            .await
        {
            Ok(result) => Ok(Response::new(CheckAccountPostableResponse {
                is_postable: result.is_postable,
                reason: result.error_message.clone().unwrap_or_default(),
                messages: result
                    .error_message
                    .map(|msg| vec![proto::common::v1::ApiMessage {
                        r#type: if result.is_postable { "info" } else { "error" }.to_string(),
                        code: if result.is_postable { "OK" } else { "NOT_POSTABLE" }.to_string(),
                        message: msg,
                        target: String::new(),
                    }])
                    .unwrap_or_default(),
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_account_hierarchy(
        &self,
        _request: Request<GetAccountHierarchyRequest>,
    ) -> Result<Response<AccountHierarchyResponse>, Status> {
        Err(Status::unimplemented("Not yet implemented"))
    }

    async fn list_child_accounts(
        &self,
        _request: Request<ListChildAccountsRequest>,
    ) -> Result<Response<ListChildAccountsResponse>, Status> {
        Err(Status::unimplemented("Not yet implemented"))
    }

    async fn get_account_path(
        &self,
        _request: Request<GetAccountPathRequest>,
    ) -> Result<Response<GetAccountPathResponse>, Status> {
        Err(Status::unimplemented("Not yet implemented"))
    }

    async fn create_account_group(
        &self,
        _request: Request<CreateAccountGroupRequest>,
    ) -> Result<Response<AccountGroupResponse>, Status> {
        Err(Status::unimplemented("Not yet implemented"))
    }

    async fn list_account_groups(
        &self,
        _request: Request<ListAccountGroupsRequest>,
    ) -> Result<Response<ListAccountGroupsResponse>, Status> {
        Err(Status::unimplemented("Not yet implemented"))
    }

    async fn batch_create_gl_accounts(
        &self,
        _request: Request<BatchCreateGlAccountsRequest>,
    ) -> Result<Response<BatchCreateGlAccountsResponse>, Status> {
        Err(Status::unimplemented("Not yet implemented"))
    }

    async fn batch_update_gl_accounts(
        &self,
        _request: Request<BatchUpdateGlAccountsRequest>,
    ) -> Result<Response<BatchUpdateGlAccountsResponse>, Status> {
        Err(Status::unimplemented("Not yet implemented"))
    }

    async fn import_accounts(
        &self,
        _request: Request<ImportAccountsRequest>,
    ) -> Result<Response<ImportAccountsResponse>, Status> {
        Err(Status::unimplemented("Not yet implemented"))
    }

    async fn export_accounts(
        &self,
        _request: Request<ExportAccountsRequest>,
    ) -> Result<Response<ExportAccountsResponse>, Status> {
        Err(Status::unimplemented("Not yet implemented"))
    }
}

// Helper conversion functions
fn convert_account_nature(value: i32) -> crate::domain::AccountNature {
    match proto::AccountNature::try_from(value).ok() {
        Some(proto::AccountNature::Asset) => crate::domain::AccountNature::Asset,
        Some(proto::AccountNature::Liability) => crate::domain::AccountNature::Liability,
        Some(proto::AccountNature::Equity) => crate::domain::AccountNature::Equity,
        Some(proto::AccountNature::Revenue) => crate::domain::AccountNature::Revenue,
        Some(proto::AccountNature::Expense) => crate::domain::AccountNature::Expense,
        Some(proto::AccountNature::ProfitLoss) => crate::domain::AccountNature::ProfitLoss,
        _ => crate::domain::AccountNature::Asset,
    }
}

fn proto_account_nature(nature: &crate::domain::AccountNature) -> proto::AccountNature {
    match nature {
        crate::domain::AccountNature::Asset => proto::AccountNature::Asset,
        crate::domain::AccountNature::Liability => proto::AccountNature::Liability,
        crate::domain::AccountNature::Equity => proto::AccountNature::Equity,
        crate::domain::AccountNature::Revenue => proto::AccountNature::Revenue,
        crate::domain::AccountNature::Expense => proto::AccountNature::Expense,
        crate::domain::AccountNature::ProfitLoss => proto::AccountNature::ProfitLoss,
    }
}

fn proto_account_status(status: &crate::domain::AccountStatus) -> proto::AccountStatus {
    match status {
        crate::domain::AccountStatus::Active => proto::AccountStatus::Active,
        crate::domain::AccountStatus::Inactive => proto::AccountStatus::Inactive,
        crate::domain::AccountStatus::Blocked => proto::AccountStatus::Blocked,
        crate::domain::AccountStatus::MarkedForDeletion => proto::AccountStatus::MarkedForDeletion,
    }
}

fn proto_account_level(level: i32) -> proto::AccountLevel {
    match level {
        1 => proto::AccountLevel::AccountLevel1,
        2 => proto::AccountLevel::AccountLevel2,
        3 => proto::AccountLevel::AccountLevel3,
        4 => proto::AccountLevel::AccountLevel4,
        5 => proto::AccountLevel::AccountLevel5,
        _ => proto::AccountLevel::AccountLevel1,
    }
}

fn proto_balance_indicator(indicator: &crate::domain::BalanceIndicator) -> proto::BalanceIndicator {
    match indicator {
        crate::domain::BalanceIndicator::Debit => proto::BalanceIndicator::Debit,
        crate::domain::BalanceIndicator::Credit => proto::BalanceIndicator::Credit,
    }
}

fn proto_account_category(_category: &str) -> proto::AccountCategory {
    proto::AccountCategory::BalanceSheet
}
