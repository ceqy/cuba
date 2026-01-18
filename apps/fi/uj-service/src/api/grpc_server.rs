use crate::{
    domain::{
        repositories::{
            PaginationParams, UniversalJournalFilter, UniversalJournalRepository,
        },
        AccountType, SourceModule, UniversalJournalEntry,
    },
    infrastructure::grpc::proto::uj::v1::{
        universal_journal_service_server::UniversalJournalService, AggregateUniversalJournalRequest,
        AggregateUniversalJournalResponse, GetUniversalJournalEntryRequest,
        QueryUniversalJournalRequest, QueryUniversalJournalResponse,
    },
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct UniversalJournalServiceImpl {
    repository: Arc<dyn UniversalJournalRepository>,
}

impl UniversalJournalServiceImpl {
    pub fn new(repository: Arc<dyn UniversalJournalRepository>) -> Self {
        Self { repository }
    }

    /// 将 proto 过滤器转换为领域过滤器
    fn convert_filter(
        &self,
        proto_filter: Option<crate::infrastructure::grpc::proto::uj::v1::UniversalJournalFilter>,
    ) -> UniversalJournalFilter {
        let filter = proto_filter.unwrap_or_default();

        UniversalJournalFilter {
            ledgers: if filter.ledgers.is_empty() {
                None
            } else {
                Some(filter.ledgers)
            },
            company_codes: if filter.company_codes.is_empty() {
                None
            } else {
                Some(filter.company_codes)
            },
            fiscal_year_from: if filter.fiscal_year_from > 0 {
                Some(filter.fiscal_year_from)
            } else {
                None
            },
            fiscal_year_to: if filter.fiscal_year_to > 0 {
                Some(filter.fiscal_year_to)
            } else {
                None
            },
            document_types: if filter.document_types.is_empty() {
                None
            } else {
                Some(filter.document_types)
            },
            posting_date_from: filter
                .posting_date_from
                .and_then(|ts| chrono::NaiveDateTime::from_timestamp_opt(ts.seconds, 0))
                .map(|dt| dt.date()),
            posting_date_to: filter
                .posting_date_to
                .and_then(|ts| chrono::NaiveDateTime::from_timestamp_opt(ts.seconds, 0))
                .map(|dt| dt.date()),
            document_date_from: filter
                .document_date_from
                .and_then(|ts| chrono::NaiveDateTime::from_timestamp_opt(ts.seconds, 0))
                .map(|dt| dt.date()),
            document_date_to: filter
                .document_date_to
                .and_then(|ts| chrono::NaiveDateTime::from_timestamp_opt(ts.seconds, 0))
                .map(|dt| dt.date()),
            gl_accounts: if filter.gl_accounts.is_empty() {
                None
            } else {
                Some(filter.gl_accounts)
            },
            account_types: None, // TODO: 实现账户类型转换
            business_partners: if filter.business_partners.is_empty() {
                None
            } else {
                Some(filter.business_partners)
            },
            cost_centers: if filter.cost_centers.is_empty() {
                None
            } else {
                Some(filter.cost_centers)
            },
            profit_centers: if filter.profit_centers.is_empty() {
                None
            } else {
                Some(filter.profit_centers)
            },
            segments: if filter.segments.is_empty() {
                None
            } else {
                Some(filter.segments)
            },
            business_areas: if filter.business_areas.is_empty() {
                None
            } else {
                Some(filter.business_areas)
            },
            source_modules: if filter.source_modules.is_empty() {
                None
            } else {
                Some(
                    filter
                        .source_modules
                        .into_iter()
                        .map(|m| match m {
                            1 => "GL",
                            2 => "AP",
                            3 => "AR",
                            4 => "AA",
                            5 => "MM",
                            6 => "SD",
                            7 => "CO",
                            8 => "TR",
                            _ => "UNSPECIFIED",
                        })
                        .map(|s| s.to_string())
                        .collect(),
                )
            },
            only_open_items: filter.only_open_items,
            only_cleared_items: filter.only_cleared_items,
            special_gl_indicators: if filter.special_gl_indicators.is_empty() {
                None
            } else {
                Some(filter.special_gl_indicators)
            },
            search_text: if filter.search_text.is_empty() {
                None
            } else {
                Some(filter.search_text)
            },
        }
    }

    /// 将领域模型转换为 proto 模型
    fn convert_to_proto(
        &self,
        entry: &UniversalJournalEntry,
    ) -> crate::infrastructure::grpc::proto::uj::v1::UniversalJournalEntry {
        use crate::infrastructure::grpc::proto::uj::v1;
        use prost_types::Timestamp;

        let source_module = match entry.source_module {
            SourceModule::GL => 1,  // SOURCE_MODULE_GL
            SourceModule::AP => 2,  // SOURCE_MODULE_AP
            SourceModule::AR => 3,  // SOURCE_MODULE_AR
            SourceModule::AA => 4,  // SOURCE_MODULE_AA
            SourceModule::MM => 5,  // SOURCE_MODULE_MM
            SourceModule::SD => 6,  // SOURCE_MODULE_SD
            SourceModule::CO => 7,  // SOURCE_MODULE_CO
            SourceModule::TR => 8,  // SOURCE_MODULE_FI_TR
            _ => 0,  // SOURCE_MODULE_UNSPECIFIED
        };

        let account_type = match entry.account_type {
            AccountType::GL => 1,       // ACCOUNT_TYPE_GL
            AccountType::Customer => 2, // ACCOUNT_TYPE_CUSTOMER
            AccountType::Vendor => 3,   // ACCOUNT_TYPE_VENDOR
            AccountType::Asset => 4,    // ACCOUNT_TYPE_ASSET
            AccountType::Material => 5, // ACCOUNT_TYPE_MATERIAL
            _ => 0,  // ACCOUNT_TYPE_UNSPECIFIED
        };

        v1::UniversalJournalEntry {
            ledger: entry.ledger.clone(),
            company_code: entry.company_code.clone(),
            fiscal_year: entry.fiscal_year,
            document_number: entry.document_number.clone(),
            document_line: entry.document_line,
            document_type: entry.document_type.clone(),
            document_date: Some(Timestamp {
                seconds: entry.document_date.and_hms_opt(0, 0, 0).unwrap().timestamp(),
                nanos: 0,
            }),
            posting_date: Some(Timestamp {
                seconds: entry.posting_date.and_hms_opt(0, 0, 0).unwrap().timestamp(),
                nanos: 0,
            }),
            fiscal_period: entry.fiscal_period,
            reference_document: entry.reference_document.clone().unwrap_or_default(),
            header_text: entry.header_text.clone().unwrap_or_default(),
            document_currency: entry.document_currency.clone(),
            exchange_rate: entry
                .exchange_rate
                .map(|r| r.to_string())
                .unwrap_or_default(),
            logical_system: entry.logical_system.clone().unwrap_or_default(),
            transaction_code: entry.transaction_code.clone().unwrap_or_default(),
            posting_key: entry.posting_key.clone(),
            debit_credit_indicator: entry.debit_credit_indicator.clone(),
            account_type,
            gl_account: entry.gl_account.clone(),
            business_partner: entry.business_partner.clone().unwrap_or_default(),
            material: entry.material.clone().unwrap_or_default(),
            plant: entry.plant.clone().unwrap_or_default(),
            item_text: entry.item_text.clone().unwrap_or_default(),
            assignment_number: entry.assignment_number.clone().unwrap_or_default(),
            amount_in_document_currency: Some(crate::infrastructure::grpc::proto::common::v1::MonetaryValue {
                value: entry.amount_in_document_currency.to_string(),
                currency_code: entry.document_currency.clone(),
            }),
            amount_in_local_currency: Some(crate::infrastructure::grpc::proto::common::v1::MonetaryValue {
                value: entry.amount_in_local_currency.to_string(),
                currency_code: entry.local_currency.clone(),
            }),
            amount_in_group_currency: entry.amount_in_group_currency.map(|amt| {
                crate::infrastructure::grpc::proto::common::v1::MonetaryValue {
                    value: amt.to_string(),
                    currency_code: entry.group_currency.clone().unwrap_or_default(),
                }
            }),
            amount_in_global_currency: entry.amount_in_global_currency.map(|amt| {
                crate::infrastructure::grpc::proto::common::v1::MonetaryValue {
                    value: amt.to_string(),
                    currency_code: entry.global_currency.clone().unwrap_or_default(),
                }
            }),
            amount_in_ledger_currency: entry.amount_in_ledger_currency.map(|amt| {
                crate::infrastructure::grpc::proto::common::v1::MonetaryValue {
                    value: amt.to_string(),
                    currency_code: entry.local_currency.clone(),
                }
            }),
            quantity: entry.quantity.map(|q| {
                crate::infrastructure::grpc::proto::common::v1::QuantityValue {
                    value: q.to_string(),
                    unit_code: entry.quantity_unit.clone().unwrap_or_default(),
                }
            }),
            cost_center: entry.cost_center.clone().unwrap_or_default(),
            profit_center: entry.profit_center.clone().unwrap_or_default(),
            segment: entry.segment.clone().unwrap_or_default(),
            functional_area: entry.functional_area.clone().unwrap_or_default(),
            business_area: entry.business_area.clone().unwrap_or_default(),
            controlling_area: entry.controlling_area.clone().unwrap_or_default(),
            internal_order: entry.internal_order.clone().unwrap_or_default(),
            wbs_element: entry.wbs_element.clone().unwrap_or_default(),
            sales_order: entry.sales_order.clone().unwrap_or_default(),
            sales_order_item: entry.sales_order_item.unwrap_or(0),
            tax_code: entry.tax_code.clone().unwrap_or_default(),
            tax_jurisdiction: entry.tax_jurisdiction.clone().unwrap_or_default(),
            tax_amount: entry.tax_amount.map(|amt| {
                crate::infrastructure::grpc::proto::common::v1::MonetaryValue {
                    value: amt.to_string(),
                    currency_code: entry.document_currency.clone(),
                }
            }),
            clearing_document: entry.clearing_document.clone().unwrap_or_default(),
            clearing_date: entry.clearing_date.map(|date| Timestamp {
                seconds: date.and_hms_opt(0, 0, 0).unwrap().timestamp(),
                nanos: 0,
            }),
            baseline_date: entry.baseline_date.map(|date| Timestamp {
                seconds: date.and_hms_opt(0, 0, 0).unwrap().timestamp(),
                nanos: 0,
            }),
            due_date: entry.due_date.map(|date| Timestamp {
                seconds: date.and_hms_opt(0, 0, 0).unwrap().timestamp(),
                nanos: 0,
            }),
            payment_terms: entry.payment_terms.clone().unwrap_or_default(),
            payment_method: entry.payment_method.clone().unwrap_or_default(),
            payment_block: entry.payment_block.clone().unwrap_or_default(),
            house_bank: entry.house_bank.clone().unwrap_or_default(),
            special_gl_indicator: entry.special_gl_indicator.clone().unwrap_or_default(),
            reference_document_number: entry.reference_document_number.clone().unwrap_or_default(),
            reference_fiscal_year: entry.reference_fiscal_year.unwrap_or(0),
            reference_line_item: entry.reference_line_item.unwrap_or(0),
            reference_document_type: entry.reference_document_type.clone().unwrap_or_default(),
            transaction_type: entry.transaction_type.clone().unwrap_or_default(),
            reference_transaction_type: entry.reference_transaction_type.clone().unwrap_or_default(),
            reference_key_1: entry.reference_key_1.clone().unwrap_or_default(),
            reference_key_2: entry.reference_key_2.clone().unwrap_or_default(),
            reference_key_3: entry.reference_key_3.clone().unwrap_or_default(),
            financial_area: entry.financial_area.clone().unwrap_or_default(),
            consolidation_unit: entry.consolidation_unit.clone().unwrap_or_default(),
            partner_company: entry.partner_company.clone().unwrap_or_default(),
            trading_partner: entry.trading_partner.clone().unwrap_or_default(),
            local_currency: entry.local_currency.clone(),
            group_currency: entry.group_currency.clone().unwrap_or_default(),
            global_currency: entry.global_currency.clone().unwrap_or_default(),
            amount_in_object_currency: entry.amount_in_object_currency.map(|amt| {
                crate::infrastructure::grpc::proto::common::v1::MonetaryValue {
                    value: amt.to_string(),
                    currency_code: entry.local_currency.clone(),
                }
            }),
            amount_in_profit_center_currency: entry.amount_in_profit_center_currency.map(|amt| {
                crate::infrastructure::grpc::proto::common::v1::MonetaryValue {
                    value: amt.to_string(),
                    currency_code: entry.local_currency.clone(),
                }
            }),
            dunning_key: entry.dunning_key.clone().unwrap_or_default(),
            dunning_block: entry.dunning_block.clone().unwrap_or_default(),
            last_dunning_date: entry.last_dunning_date.map(|date| Timestamp {
                seconds: date.and_hms_opt(0, 0, 0).unwrap().timestamp(),
                nanos: 0,
            }),
            dunning_level: entry.dunning_level.unwrap_or(0),
            discount_days_1: entry.discount_days_1.unwrap_or(0),
            discount_days_2: entry.discount_days_2.unwrap_or(0),
            net_payment_days: entry.net_payment_days.unwrap_or(0),
            discount_percent_1: entry
                .discount_percent_1
                .map(|p| p.to_string())
                .unwrap_or_default(),
            discount_percent_2: entry
                .discount_percent_2
                .map(|p| p.to_string())
                .unwrap_or_default(),
            discount_amount: entry.discount_amount.map(|amt| {
                crate::infrastructure::grpc::proto::common::v1::MonetaryValue {
                    value: amt.to_string(),
                    currency_code: entry.document_currency.clone(),
                }
            }),
            sending_cost_center: entry.sending_cost_center.clone().unwrap_or_default(),
            partner_profit_center: entry.partner_profit_center.clone().unwrap_or_default(),
            sending_financial_area: entry.sending_financial_area.clone().unwrap_or_default(),
            account_assignment: entry.account_assignment.clone().unwrap_or_default(),
            local_account: entry.local_account.clone().unwrap_or_default(),
            data_source: entry.data_source.clone().unwrap_or_default(),
            split_method: entry.split_method.clone().unwrap_or_default(),
            manual_split: entry.manual_split,
            created_by: entry.created_by.clone(),
            created_at: Some(Timestamp {
                seconds: entry.created_at.timestamp(),
                nanos: 0,
            }),
            created_time: Some(Timestamp {
                seconds: entry.created_at.timestamp(),
                nanos: 0,
            }),
            changed_by: entry.changed_by.clone().unwrap_or_default(),
            changed_at: entry.changed_at.map(|dt| Timestamp {
                seconds: dt.timestamp(),
                nanos: 0,
            }),
            source_module,
            extension_fields: entry.extension_fields.clone(),
        }
    }
}

#[tonic::async_trait]
impl UniversalJournalService for UniversalJournalServiceImpl {
    async fn query_universal_journal(
        &self,
        request: Request<QueryUniversalJournalRequest>,
    ) -> Result<Response<QueryUniversalJournalResponse>, Status> {
        let req = request.into_inner();

        // 转换过滤器
        let filter = self.convert_filter(req.filter);

        // 转换分页参数
        let pagination = req
            .pagination
            .map(|p| PaginationParams {
                page: p.page as i64,
                page_size: p.page_size as i64,
            })
            .unwrap_or_default();

        // 查询数据
        let (entries, pagination_response) = self
            .repository
            .query(&filter, &pagination, &req.order_by)
            .await
            .map_err(|e| Status::internal(format!("Failed to query: {}", e)))?;

        // 转换为 proto 模型
        let proto_entries: Vec<_> = entries.iter().map(|e| self.convert_to_proto(e)).collect();

        Ok(Response::new(QueryUniversalJournalResponse {
            entries: proto_entries,
            pagination: Some(crate::infrastructure::grpc::proto::common::v1::PaginationResponse {
                total_items: pagination_response.total_count,
                current_page: pagination_response.page as i32,
                page_size: pagination_response.page_size as i32,
                total_pages: pagination_response.total_pages as i32,
            }),
        }))
    }

    type StreamUniversalJournalStream =
        tokio_stream::wrappers::ReceiverStream<Result<crate::infrastructure::grpc::proto::uj::v1::UniversalJournalEntry, Status>>;

    async fn stream_universal_journal(
        &self,
        request: Request<QueryUniversalJournalRequest>,
    ) -> Result<Response<Self::StreamUniversalJournalStream>, Status> {
        let req = request.into_inner();

        // 转换过滤器
        let filter = self.convert_filter(req.filter);

        // 查询数据
        let entries = self
            .repository
            .stream(&filter, &req.order_by)
            .await
            .map_err(|e| Status::internal(format!("Failed to stream: {}", e)))?;

        // 转换为 proto 模型
        let proto_entries: Vec<_> = entries.iter().map(|e| self.convert_to_proto(e)).collect();

        // 创建流
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            for proto_entry in proto_entries {
                if tx.send(Ok(proto_entry)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    async fn get_universal_journal_entry(
        &self,
        request: Request<GetUniversalJournalEntryRequest>,
    ) -> Result<Response<crate::infrastructure::grpc::proto::uj::v1::UniversalJournalEntry>, Status> {
        let req = request.into_inner();

        let entry = self
            .repository
            .get_by_key(
                &req.ledger,
                &req.company_code,
                req.fiscal_year,
                &req.document_number,
                req.document_line,
            )
            .await
            .map_err(|e| Status::internal(format!("Failed to get entry: {}", e)))?
            .ok_or_else(|| Status::not_found("Entry not found"))?;

        Ok(Response::new(self.convert_to_proto(&entry)))
    }

    async fn aggregate_universal_journal(
        &self,
        request: Request<AggregateUniversalJournalRequest>,
    ) -> Result<Response<AggregateUniversalJournalResponse>, Status> {
        let req = request.into_inner();

        // 转换过滤器
        let filter = self.convert_filter(req.filter);

        // 转换聚合维度
        let dimensions: Vec<String> = req
            .dimensions
            .iter()
            .map(|d| format!("{:?}", d))
            .collect();

        // 转换聚合度量
        let measure = format!("{:?}", req.measure);

        // 查询聚合数据
        let results = self
            .repository
            .aggregate(&filter, &dimensions, &measure, &req.measure_field)
            .await
            .map_err(|e| Status::internal(format!("Failed to aggregate: {}", e)))?;

        // 转换为 proto 模型
        let proto_results: Vec<_> = results
            .iter()
            .map(|r| {
                crate::infrastructure::grpc::proto::uj::v1::aggregate_universal_journal_response::AggregationResult {
                    dimension_values: r.dimension_values.clone(),
                    measure_value: r.measure_value.clone(),
                    record_count: r.record_count,
                }
            })
            .collect();

        Ok(Response::new(AggregateUniversalJournalResponse {
            results: proto_results,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::{
        AggregationResult, PaginationParams, PaginationResponse, RepositoryError, UniversalJournalFilter,
        UniversalJournalRepository,
    };
    use async_trait::async_trait;
    use std::sync::Arc;

    struct DummyRepo;

    #[async_trait]
    impl UniversalJournalRepository for DummyRepo {
        async fn query(
            &self,
            _filter: &UniversalJournalFilter,
            _pagination: &PaginationParams,
            _order_by: &[String],
        ) -> Result<(Vec<UniversalJournalEntry>, PaginationResponse), RepositoryError> {
            Ok((Vec::new(), PaginationResponse::new(0, 1, 50)))
        }

        async fn stream(
            &self,
            _filter: &UniversalJournalFilter,
            _order_by: &[String],
        ) -> Result<Vec<UniversalJournalEntry>, RepositoryError> {
            Ok(Vec::new())
        }

        async fn get_by_key(
            &self,
            _ledger: &str,
            _company_code: &str,
            _fiscal_year: i32,
            _document_number: &str,
            _document_line: i32,
        ) -> Result<Option<UniversalJournalEntry>, RepositoryError> {
            Ok(None)
        }

        async fn aggregate(
            &self,
            _filter: &UniversalJournalFilter,
            _dimensions: &[String],
            _measure: &str,
            _measure_field: &str,
        ) -> Result<Vec<AggregationResult>, RepositoryError> {
            Ok(Vec::new())
        }

        async fn save(&self, _entry: &UniversalJournalEntry) -> Result<(), RepositoryError> {
            Ok(())
        }

        async fn batch_save(&self, _entries: &[UniversalJournalEntry]) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    #[test]
    fn convert_filter_maps_source_modules() {
        let svc = UniversalJournalServiceImpl::new(Arc::new(DummyRepo));
        let filter = crate::infrastructure::grpc::proto::uj::v1::UniversalJournalFilter {
            source_modules: vec![2, 3, 7, 8],
            ..Default::default()
        };

        let mapped = svc.convert_filter(Some(filter));
        let modules = mapped.source_modules.unwrap_or_default();
        assert_eq!(modules, vec!["AP", "AR", "CO", "TR"]);
    }
}
