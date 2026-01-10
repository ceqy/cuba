# ERP核心表字段映射参考（脱敏版）

本文档记录了从ERP核心系统底表到微服务Proto字段的映射关系，用于确保数据模型的一致性和完整性。

## 1. 财务会计 (FI) 领域

### 1.1 会计凭证抬头 (原表: BKPF)

| 原字段名 | 脱敏后字段名 | 中文含义 | Proto类型 |
|---------|-------------|---------|----------|
| BUKRS | company_code | 公司代码 | string |
| BELNR | document_number | 凭证编号 | string |
| GJAHR | fiscal_year | 会计年度 | int32 |
| BLART | document_type | 凭证类型 | string |
| BLDAT | document_date | 凭证日期 | google.protobuf.Timestamp |
| BUDAT | posting_date | 过账日期 | google.protobuf.Timestamp |
| MONAT | fiscal_period | 会计期间 | int32 |
| CPUDT | entry_date | 录入日期 | google.protobuf.Timestamp |
| CPUTM | entry_time | 录入时间 | string |
| USNAM | created_by | 创建人 | string |
| TCODE | transaction_code | 事务代码 | string |
| BVORG | cross_company_number | 跨公司编号 | string |
| XBLNR | reference_document | 参考凭证号 | string |
| DBBLG | recurring_entry_doc | 周期性凭证号 | string |
| STBLG | reversal_document | 冲销凭证号 | string |
| STJAH | reversal_year | 冲销年度 | int32 |
| BKTXT | header_text | 凭证抬头文本 | string |
| WAERS | currency | 货币代码 | string |
| KURSF | exchange_rate | 汇率 | string |
| AWTYP | reference_transaction | 参考事务类型 | string |
| AWKEY | reference_key | 参考关键字 | string |
| LDGRP | ledger_group | 分类账组 | string |

### 1.2 会计凭证行项目 (原表: BSEG)

| 原字段名 | 脱敏后字段名 | 中文含义 | Proto类型 |
|---------|-------------|---------|----------|
| BUKRS | company_code | 公司代码 | string |
| BELNR | document_number | 凭证编号 | string |
| GJAHR | fiscal_year | 会计年度 | int32 |
| BUZEI | line_item_number | 行项目号 | int32 |
| BUZID | line_item_type | 行项目类型 | string |
| AUGDT | clearing_date | 清账日期 | google.protobuf.Timestamp |
| AUGBL | clearing_document | 清账凭证 | string |
| BSCHL | posting_key | 记账码 | string |
| KOART | account_type | 科目类型 | string |
| SHKZG | debit_credit_indicator | 借贷标识 | string |
| GSBER | business_area | 业务范围 | string |
| PRCTR | profit_center | 利润中心 | string |
| KOSTL | cost_center | 成本中心 | string |
| HKONT | gl_account | 总账科目 | string |
| KUNNR | customer | 客户编号 | string |
| LIFNR | supplier | 供应商编号 | string |
| DMBTR | amount_local | 本位币金额 | string |
| WRBTR | amount_doc | 凭证货币金额 | string |
| MWSKZ | tax_code | 税码 | string |
| MWSTS | tax_amount | 税额 | string |
| SGTXT | item_text | 行项目文本 | string |
| ZUONR | assignment | 分配号 | string |
| VBUND | trading_partner | 贸易伙伴 | string |
| BEESSION | segment | 分部 | string |
| FKBER | functional_area | 功能范围 | string |

### 1.3 客户主数据 (原表: KNA1/KNB1)

| 原字段名 | 脱敏后字段名 | 中文含义 | Proto类型 |
|---------|-------------|---------|----------|
| KUNNR | customer_id | 客户编号 | string |
| NAME1 | name | 客户名称 | string |
| NAME2 | name_2 | 客户名称2 | string |
| LAND1 | country | 国家代码 | string |
| ORT01 | city | 城市 | string |
| PSTLZ | postal_code | 邮政编码 | string |
| STRAS | street | 街道地址 | string |
| TELF1 | telephone | 电话号码 | string |
| TELFX | fax | 传真号码 | string |
| KTOKD | account_group | 账户组 | string |
| AKONT | reconciliation_account | 统驭科目 | string |
| ZTERM | payment_terms | 付款条件 | string |
| ZWELS | payment_methods | 付款方式 | string |
| MAHNA | dunning_procedure | 催款程序 | string |
| BUSAB | accounting_clerk | 会计文员 | string |

### 1.4 供应商主数据 (原表: LFA1/LFB1)

| 原字段名 | 脱敏后字段名 | 中文含义 | Proto类型 |
|---------|-------------|---------|----------|
| LIFNR | supplier_id | 供应商编号 | string |
| NAME1 | name | 供应商名称 | string |
| NAME2 | name_2 | 供应商名称2 | string |
| LAND1 | country | 国家代码 | string |
| ORT01 | city | 城市 | string |
| PSTLZ | postal_code | 邮政编码 | string |
| STRAS | street | 街道地址 | string |
| TELF1 | telephone | 电话号码 | string |
| KTOKK | account_group | 账户组 | string |
| AKONT | reconciliation_account | 统驭科目 | string |
| ZTERM | payment_terms | 付款条件 | string |
| ZWELS | payment_methods | 付款方式 | string |
| REPRF | check_double_invoice | 重复发票检查 | bool |
| TOGRU | tolerance_group | 容差组 | string |

## 2. 采购 (MM) 领域

### 2.1 采购订单抬头 (原表: EKKO)

| 原字段名 | 脱敏后字段名 | 中文含义 | Proto类型 |
|---------|-------------|---------|----------|
| EBELN | order_number | 采购订单号 | string |
| BUKRS | company_code | 公司代码 | string |
| BSTYP | document_category | 凭证类别 | string |
| BSART | document_type | 凭证类型 | string |
| LOEKZ | deletion_indicator | 删除标识 | bool |
| STATU | status | 状态 | string |
| AEDAT | change_date | 修改日期 | google.protobuf.Timestamp |
| ERNAM | created_by | 创建人 | string |
| LIFNR | supplier | 供应商 | string |
| ZTERM | payment_terms | 付款条件 | string |
| EKORG | purchasing_org | 采购组织 | string |
| EKGRP | purchasing_group | 采购组 | string |
| WAERS | currency | 货币代码 | string |
| WKURS | exchange_rate | 汇率 | string |
| BEDAT | order_date | 订单日期 | google.protobuf.Timestamp |
| KDATB | validity_start | 有效期开始 | google.protobuf.Timestamp |
| KDATE | validity_end | 有效期结束 | google.protobuf.Timestamp |
| KONNR | contract_number | 合同编号 | string |
| AUTLF | complete_delivery | 完整交货 | bool |
| RESWK | supplying_plant | 供应工厂 | string |
| FRGKE | release_indicator | 审批标识 | string |
| FRGZU | release_status | 审批状态 | string |
| PROCSTAT | processing_status | 处理状态 | string |

### 2.2 采购订单行项目 (原表: EKPO)

| 原字段名 | 脱敏后字段名 | 中文含义 | Proto类型 |
|---------|-------------|---------|----------|
| EBELN | order_number | 采购订单号 | string |
| EBELP | item_number | 行项目号 | int32 |
| LOEKZ | deletion_indicator | 删除标识 | bool |
| STATU | status | 状态 | string |
| AEDAT | change_date | 修改日期 | google.protobuf.Timestamp |
| TXZ01 | short_text | 短文本 | string |
| MATNR | material | 物料编号 | string |
| EMATN | supplier_material | 供应商物料号 | string |
| BUKRS | company_code | 公司代码 | string |
| WERKS | plant | 工厂 | string |
| LGORT | storage_location | 存储地点 | string |
| MATKL | material_group | 物料组 | string |
| INFNR | info_record | 信息记录 | string |
| MENGE | quantity | 数量 | string |
| MEINS | unit | 单位 | string |
| NETPR | net_price | 净价 | string |
| PEINH | price_unit | 价格单位 | int32 |
| NETWR | net_value | 净值 | string |
| BRTWR | gross_value | 总值 | string |
| AGDAT | delivery_date | 交货日期 | google.protobuf.Timestamp |
| WEBAZ | gr_processing_days | 收货处理天数 | int32 |
| MWSKZ | tax_code | 税码 | string |
| BONUS | volume_rebate_group | 批量折扣组 | string |
| INSMK | quality_inspection | 质检标识 | bool |
| SPINF | update_info_record | 更新信息记录 | bool |
| PRSDR | print_price | 打印价格 | bool |
| SCHPR | estimated_price | 估计价格 | bool |
| BWTAR | valuation_type | 评估类型 | string |
| BWTTY | valuation_category | 评估类别 | string |
| PSTYP | item_category | 项目类别 | string |
| KNTTP | account_assignment | 账户分配类别 | string |
| KZVBR | consumption_posting | 消耗过账 | string |
| VRTKZ | distribution_indicator | 分配标识 | string |
| TWRKZ | partial_invoice | 部分发票 | bool |
| WEPOS | goods_receipt | 收货标识 | bool |
| WEUNB | unplanned_delivery | 非计划交货 | bool |
| REPOS | invoice_receipt | 发票收据 | bool |
| WEBRE | gr_based_iv | 基于收货的发票校验 | bool |
| KZABS | order_acknowledgment | 订单确认 | bool |
| LABNR | schedule_line_number | 计划行号 | string |
| KONNR | contract_number | 合同编号 | string |
| KTPNR | contract_item | 合同项目 | int32 |
| ANFNR | rfq_number | 询价单号 | string |
| ANFPS | rfq_item | 询价项目 | int32 |
| BANFN | requisition_number | 采购申请号 | string |
| BNFPO | requisition_item | 采购申请项目 | int32 |

### 2.3 采购计划行 (原表: EKET)

| 原字段名 | 脱敏后字段名 | 中文含义 | Proto类型 |
|---------|-------------|---------|----------|
| EBELN | order_number | 采购订单号 | string |
| EBELP | item_number | 行项目号 | int32 |
| ETENR | schedule_line | 计划行号 | int32 |
| EINDT | delivery_date | 交货日期 | google.protobuf.Timestamp |
| SLFDT | statistics_date | 统计日期 | google.protobuf.Timestamp |
| LPEIN | delivery_date_category | 交货日期类别 | string |
| MENGE | scheduled_quantity | 计划数量 | string |
| AMENG | previous_quantity | 先前数量 | string |
| WEMNG | goods_receipt_quantity | 收货数量 | string |
| WAMNG | issued_quantity | 发出数量 | string |
| UZEIT | delivery_time | 交货时间 | string |
| BANFN | requisition_number | 采购申请号 | string |
| BNFPO | requisition_item | 采购申请项目 | int32 |
| ESTKZ | creation_indicator | 创建标识 | string |

## 3. 销售 (SD) 领域

### 3.1 销售订单抬头 (原表: VBAK)

| 原字段名 | 脱敏后字段名 | 中文含义 | Proto类型 |
|---------|-------------|---------|----------|
| VBELN | order_number | 销售订单号 | string |
| ERDAT | creation_date | 创建日期 | google.protobuf.Timestamp |
| ERZET | creation_time | 创建时间 | string |
| ERNAM | created_by | 创建人 | string |
| ANGDT | quotation_valid_from | 报价有效期从 | google.protobuf.Timestamp |
| BNDDT | quotation_valid_to | 报价有效期至 | google.protobuf.Timestamp |
| AUDAT | document_date | 凭证日期 | google.protobuf.Timestamp |
| VBTYP | document_category | 凭证类别 | string |
| AUART | order_type | 订单类型 | string |
| AUGRU | order_reason | 订单原因 | string |
| GWLDT | warranty_date | 保修日期 | google.protobuf.Timestamp |
| SUBMI | collective_number | 集合编号 | string |
| LIFSK | delivery_block | 交货冻结 | string |
| FAKSK | billing_block | 开票冻结 | string |
| NETWR | net_value | 净值 | string |
| WAERK | currency | 货币代码 | string |
| VKORG | sales_org | 销售组织 | string |
| VTWEG | distribution_channel | 分销渠道 | string |
| SPART | division | 产品组 | string |
| VKGRP | sales_group | 销售组 | string |
| VKBUR | sales_office | 销售办事处 | string |
| GSBER | business_area | 业务范围 | string |
| GESSION | segment | 分部 | string |
| KUNNR | sold_to_party | 售达方 | string |
| KVGR1 | customer_group_1 | 客户组1 | string |
| KVGR2 | customer_group_2 | 客户组2 | string |
| KVGR3 | customer_group_3 | 客户组3 | string |
| KVGR4 | customer_group_4 | 客户组4 | string |
| KVGR5 | customer_group_5 | 客户组5 | string |
| KNUMV | condition_number | 条件编号 | string |
| VDATU | requested_delivery_date | 请求交货日期 | google.protobuf.Timestamp |
| VPRGR | proposed_date_type | 建议日期类型 | string |
| AUTLF | complete_delivery | 完整交货 | bool |
| VBKLA | original_system | 原始系统 | string |
| VBKLT | document_indicator | 凭证标识 | string |
| KALSM | pricing_procedure | 定价程序 | string |
| VSBED | shipping_conditions | 装运条件 | string |
| FKARA | billing_type | 开票类型 | string |
| AWAHR | order_probability | 订单概率 | int32 |
| BSTNK | customer_po | 客户采购订单号 | string |
| BSTDK | customer_po_date | 客户采购订单日期 | google.protobuf.Timestamp |
| TELF1 | telephone | 电话号码 | string |
| MAHZA | number_of_reminders | 催款次数 | int32 |
| MAHDT | last_reminder_date | 最后催款日期 | google.protobuf.Timestamp |
| KTEXT | search_term | 搜索词 | string |
| ABHESSION | unloading_point | 卸货点 | string |
| INCO1 | incoterms_1 | 国际贸易条款1 | string |
| INCO2 | incoterms_2 | 国际贸易条款2 | string |
| ZESSION | incoterms_location | 国际贸易条款地点 | string |
| LCNUM | letter_of_credit | 信用证号 | string |
| ABSSC | payment_guarantee | 付款担保程序 | string |
| VGBEL | reference_document | 参考凭证 | string |
| OBJNR | object_number | 对象编号 | string |
| BUKRS_VF | billing_company_code | 开票公司代码 | string |
| TAXK1 | tax_classification_1 | 税分类1 | string |
| TAXK2 | tax_classification_2 | 税分类2 | string |
| TAXK3 | tax_classification_3 | 税分类3 | string |
| TAXK4 | tax_classification_4 | 税分类4 | string |

### 3.2 销售订单行项目 (原表: VBAP)

| 原字段名 | 脱敏后字段名 | 中文含义 | Proto类型 |
|---------|-------------|---------|----------|
| VBELN | order_number | 销售订单号 | string |
| POSNR | item_number | 行项目号 | int32 |
| MATNR | material | 物料编号 | string |
| ARKTX | item_description | 项目描述 | string |
| PSTYV | item_category | 项目类别 | string |
| CHARG | batch | 批次 | string |
| MATKL | material_group | 物料组 | string |
| WERKS | plant | 工厂 | string |
| LGORT | storage_location | 存储地点 | string |
| VSTEL | shipping_point | 装运点 | string |
| ROUTE | route | 路线 | string |
| STKEY | bom_origin | BOM来源 | string |
| STDAT | bom_date | BOM日期 | google.protobuf.Timestamp |
| STLNR | bom_number | BOM编号 | string |
| AWAHR | order_probability | 订单概率 | int32 |
| ERDAT | creation_date | 创建日期 | google.protobuf.Timestamp |
| ERNAM | created_by | 创建人 | string |
| ERZET | creation_time | 创建时间 | string |
| ABGRU | rejection_reason | 拒绝原因 | string |
| UEPOS | higher_level_item | 上级项目 | int32 |
| GRPOS | alternative_item | 替代项目 | int32 |
| FKREL | billing_relevance | 开票相关性 | string |
| KMEIN | condition_unit | 条件单位 | string |
| KPEIN | condition_price_unit | 条件价格单位 | int32 |
| KWMENG | order_quantity | 订单数量 | string |
| LSMENG | required_quantity | 需求数量 | string |
| KBMENG | cumulative_confirmed_qty | 累计确认数量 | string |
| KLMENG | cumulative_ordered_qty | 累计订购数量 | string |
| VRKME | sales_unit | 销售单位 | string |
| UMVKZ | conversion_numerator | 换算分子 | int32 |
| UMVKN | conversion_denominator | 换算分母 | int32 |
| MEINS | base_unit | 基本单位 | string |
| NETWR | net_value | 净值 | string |
| WAERK | currency | 货币代码 | string |
| NETPR | net_price | 净价 | string |
| MWSBP | tax_amount | 税额 | string |
| PRSDT | pricing_date | 定价日期 | google.protobuf.Timestamp |
| NTGEW | net_weight | 净重 | string |
| GEWEI | weight_unit | 重量单位 | string |
| BRGEW | gross_weight | 毛重 | string |
| VOLUM | volume | 体积 | string |
| VOLEH | volume_unit | 体积单位 | string |
| VGBEL | reference_document | 参考凭证 | string |
| VGPOS | reference_item | 参考项目 | int32 |
| VOESSION | reference_preceding_doc | 前置参考凭证 | string |
| VOESSION_POS | reference_preceding_item | 前置参考项目 | int32 |
| UPFLU | update_indicator | 更新标识 | string |
| SPART | division | 产品组 | string |
| MVGR1 | material_group_1 | 物料组1 | string |
| MVGR2 | material_group_2 | 物料组2 | string |
| MVGR3 | material_group_3 | 物料组3 | string |
| MVGR4 | material_group_4 | 物料组4 | string |
| MVGR5 | material_group_5 | 物料组5 | string |
| KONDM | material_pricing_group | 物料定价组 | string |
| KTGRM | account_assignment_group | 账户分配组 | string |
| BONUS | volume_rebate_group | 批量折扣组 | string |
| PROVG | commission_group | 佣金组 | string |
| PMATN | pricing_reference_material | 定价参考物料 | string |
| KNUMH | condition_record_number | 条件记录号 | string |
| OBJNR | object_number | 对象编号 | string |
| SHKZG | returns_item | 退货项目 | string |
| SKTOF | cash_discount | 现金折扣 | bool |
| PRSOK | pricing_ok | 定价确认 | bool |
| MTVFP | availability_check | 可用性检查 | string |
| SUMBD | requirements_type | 需求类型 | string |
| BEDAE | requirements_class | 需求类别 | string |
| KBVER | allowed_deviation | 允许偏差 | string |
| KEVER | days_deviation | 偏差天数 | int32 |
| VBEAF | fixed_date_quantity | 固定日期数量 | string |
| VBEAV | fixed_date_value | 固定日期值 | string |
| STCUR | exchange_rate_stats | 统计汇率 | string |
| KOWRR | statistical_value | 统计值 | bool |
| STADAT | statistics_date | 统计日期 | google.protobuf.Timestamp |
| EXART | business_transaction | 业务事务类型 | string |
| PRESSION | pricing_type | 定价类型 | string |
| KNPRS | pricing_indicator | 定价标识 | string |
| KZVBR | consumption_posting | 消耗过账 | string |
| VGTYP | preceding_doc_category | 前置凭证类别 | string |
| RFPNT | reference_point | 参考点 | string |
| ABLFZ | rounding_quantity | 舍入数量 | string |
| VPZUO | allocation_indicator | 分配标识 | string |
| PRBME | base_unit_pricing | 基本单位定价 | string |
| UMREF | conversion_factor | 换算因子 | string |
| SSESSION | special_stock_indicator | 特殊库存标识 | string |
| SOBESSION | special_stock_number | 特殊库存编号 | string |
| VPMAT | planning_material | 计划物料 | string |
| VPWRK | planning_plant | 计划工厂 | string |
| PAOESSION | profitability_segment | 获利能力段 | string |
| PS_PSP_PNR | wbs_element | WBS元素 | string |
| AUFNR | order | 订单编号 | string |
| UPMAT | cross_plant_material | 跨工厂物料 | string |
| UKONM | cross_plant_condition | 跨工厂条件 | string |
| MFRGR | material_freight_group | 物料运费组 | string |
| PLAVO | planning_delivery_sched | 计划交货计划 | string |
| KANNR | kanban_sequence | 看板序列 | string |
| CMPRE | credit_price | 信用价格 | string |
| CMPNT | credit_percentage | 信用百分比 | string |
| CMTFG | credit_function | 信用功能 | string |
| CMPRE_FLT | credit_price_float | 信用价格浮动 | string |
| CMKUA | credit_exchange_rate | 信用汇率 | string |
| CUESSION | configuration_number | 配置编号 | string |
| CUOBJ | configuration_object | 配置对象 | string |
| CUOBJ_CH | configuration_change | 配置变更 | string |
| CEPOK | expected_price_ok | 预期价格确认 | bool |
| KOUPD | condition_update | 条件更新 | string |
| SERAIL | serial_number_profile | 序列号配置文件 | string |
| ANESSION | serial_number | 序列号 | string |
| NACHL | subsequent_delivery | 后续交货 | bool |
| MAESSION | manual_completion | 手动完成 | bool |
| SESSION | completion_rule | 完成规则 | string |
| ABFOR | form_of_payment_guarantee | 付款担保形式 | string |
| ABGES | guaranteed_amount | 担保金额 | string |
| J_1BCFOP | cfop_code | CFOP代码 | string |
| J_1BTAXLW1 | tax_law_1 | 税法1 | string |
| J_1BTAXLW2 | tax_law_2 | 税法2 | string |
| J_1BTXSDC | tax_code_determination | 税码确定 | string |

## 4. 生产 (PP) 领域

### 4.1 生产订单抬头 (原表: AUFK)

| 原字段名 | 脱敏后字段名 | 中文含义 | Proto类型 |
|---------|-------------|---------|----------|
| AUFNR | order_number | 订单编号 | string |
| AUART | order_type | 订单类型 | string |
| AUTYP | order_category | 订单类别 | string |
| REFNR | reference_order | 参考订单 | string |
| ERNAM | created_by | 创建人 | string |
| ERDAT | creation_date | 创建日期 | google.protobuf.Timestamp |
| AENAM | changed_by | 修改人 | string |
| AEDAT | change_date | 修改日期 | google.protobuf.Timestamp |
| KTEXT | description | 描述 | string |
| LTEXT | long_text | 长文本 | string |
| BUKRS | company_code | 公司代码 | string |
| WERKS | plant | 工厂 | string |
| GSBER | business_area | 业务范围 | string |
| KOKRS | controlling_area | 控制范围 | string |
| KOSTV | responsible_cost_center | 责任成本中心 | string |
| STORT | location | 位置 | string |
| SOWRK | location_plant | 位置工厂 | string |
| ASESSION | settlement_rule | 结算规则 | string |
| PRCTR | profit_center | 利润中心 | string |
| FUNC_AREA | functional_area | 功能范围 | string |
| CYCLE | settlement_period | 结算期间 | string |
| SESSION | segment | 分部 | string |
| OBJNR | object_number | 对象编号 | string |
| PHAS0 | created_status | 已创建状态 | bool |
| PHAS1 | released_status | 已下达状态 | bool |
| PHAS2 | confirmed_status | 已确认状态 | bool |
| PHAS3 | delivered_status | 已交货状态 | bool |
| LOESSION | deletion_flag | 删除标识 | bool |
| IESSION | settlement_flag | 结算标识 | bool |
| ABKRS | settlement_period_unit | 结算期间单位 | string |

### 4.2 生产订单组件 (原表: RESB)

| 原字段名 | 脱敏后字段名 | 中文含义 | Proto类型 |
|---------|-------------|---------|----------|
| RSNUM | reservation_number | 预留编号 | string |
| RSPOS | reservation_item | 预留项目 | int32 |
| RSART | record_type | 记录类型 | string |
| AUFNR | order_number | 订单编号 | string |
| AUFPL | routing_number | 工艺路线编号 | string |
| APLZL | routing_counter | 工艺路线计数器 | int32 |
| VORNR | operation_number | 工序编号 | string |
| MATNR | material | 物料编号 | string |
| WERKS | plant | 工厂 | string |
| LGORT | storage_location | 存储地点 | string |
| BDMNG | requirement_quantity | 需求数量 | string |
| MEINS | unit | 单位 | string |
| ENMNG | withdrawn_quantity | 已领用数量 | string |
| BDTER | requirement_date | 需求日期 | google.protobuf.Timestamp |
| BWART | movement_type | 移动类型 | string |
| SOBESSION | special_stock_indicator | 特殊库存标识 | string |
| KZEAR | final_issue | 最终发料 | bool |
| SHKZG | debit_credit_indicator | 借贷标识 | string |
| WAESSION | goods_movement_status | 货物移动状态 | string |
| DBESSION | direct_procurement | 直接采购 | string |
| XLOESSION | deletion_indicator | 删除标识 | bool |
| KZESSION | backflush_indicator | 倒冲标识 | bool |
| DUMPS | phantom_item | 虚拟项目 | bool |
| POSTP | item_category | 项目类别 | string |
| POESSION | position | 位置 | string |
| POTX1 | item_text_1 | 项目文本1 | string |
| POTX2 | item_text_2 | 项目文本2 | string |
| SORTF | sort_string | 排序字符串 | string |
| RGESSION | reference_indicator | 参考标识 | bool |
| BEESSION | procurement_indicator | 采购标识 | string |
| NOMNG | scrap_quantity | 报废数量 | string |
| ALPGR | alternative_group | 替代组 | string |
| ALPRF | priority | 优先级 | string |
| ALPST | strategy | 策略 | string |
| EWAHR | usage_probability | 使用概率 | string |
| CHARG | batch | 批次 | string |
| CLESSION | class | 类别 | string |
| CUOBJ | configuration_object | 配置对象 | string |
| CUESSION | configuration_number | 配置编号 | string |
| KNTTP | account_assignment | 账户分配类别 | string |
| SOBKZ | special_stock | 特殊库存 | string |
| LIFNR | supplier | 供应商 | string |
| LIFZT | planned_delivery_days | 计划交货天数 | int32 |
| PLIFZ | planned_delivery_time | 计划交货时间 | int32 |
| WEBAZ | gr_processing_days | 收货处理天数 | int32 |
| PEINH | price_unit | 价格单位 | int32 |
| PREIS | price | 价格 | string |
| SAKTO | gl_account | 总账科目 | string |
| PRCTR | profit_center | 利润中心 | string |
| FISTL | funds_center | 资金中心 | string |
| GESSION | commitment_item | 承诺项目 | string |
| FKBER | functional_area | 功能范围 | string |
| GRANT_NBR | grant_number | 拨款编号 | string |
| BUDGET_PD | budget_period | 预算期间 | string |
| KSTRG | cost_object | 成本对象 | string |
| PAOESSION | profitability_segment | 获利能力段 | string |
| PS_PSP_PNR | wbs_element | WBS元素 | string |
| NPLNR | network | 网络 | string |
| AUFPL_ORD | order_routing | 订单工艺路线 | string |
| APLZL_ORD | order_routing_counter | 订单工艺路线计数器 | int32 |
| VORNR_ORD | order_operation | 订单工序 | string |
| OBJNR | object_number | 对象编号 | string |
| AUFNR_ORG | original_order | 原始订单 | string |
| RSNUM_ORG | original_reservation | 原始预留 | string |
| RSPOS_ORG | original_reservation_item | 原始预留项目 | int32 |
| UMESSION | conversion_indicator | 换算标识 | string |
| UMREZ | conversion_numerator | 换算分子 | int32 |
| UMREN | conversion_denominator | 换算分母 | int32 |
| KZAUS | discontinuation_indicator | 停产标识 | string |
| AUSDT | discontinuation_date | 停产日期 | google.protobuf.Timestamp |
| NFESSION | follow_up_material | 后续物料 | string |
| KZBED | requirements_indicator | 需求标识 | string |
| KZEAR_ORIG | original_final_issue | 原始最终发料 | bool |
| ESSION | segment | 分部 | string |

## 5. 库存管理 (MM-IM) 领域

### 5.1 物料凭证抬头 (原表: MKPF)

| 原字段名 | 脱敏后字段名 | 中文含义 | Proto类型 |
|---------|-------------|---------|----------|
| MBLNR | document_number | 物料凭证号 | string |
| MJAHR | document_year | 凭证年度 | int32 |
| VGART | transaction_type | 事务类型 | string |
| BLART | document_type | 凭证类型 | string |
| BLAUM | document_type_original | 原始凭证类型 | string |
| BLDAT | document_date | 凭证日期 | google.protobuf.Timestamp |
| BUDAT | posting_date | 过账日期 | google.protobuf.Timestamp |
| CPUDT | entry_date | 录入日期 | google.protobuf.Timestamp |
| CPUTM | entry_time | 录入时间 | string |
| AEDAT | change_date | 修改日期 | google.protobuf.Timestamp |
| USNAM | created_by | 创建人 | string |
| TCODE | transaction_code | 事务代码 | string |
| XBLNR | reference_document | 参考凭证号 | string |
| BKTXT | header_text | 凭证抬头文本 | string |
| FESSION | reversal_indicator | 冲销标识 | string |
| WESSION | goods_movement_status | 货物移动状态 | string |
| XABLN | goods_receipt_slip | 收货单号 | string |
| BWESSION | goods_movement_type | 货物移动类型 | string |
| EXNUM | external_number | 外部编号 | string |
| BESSION | stock_management_indicator | 库存管理标识 | string |
| XBLESSION | blocked_stock_indicator | 冻结库存标识 | bool |
| PRESSION | print_indicator | 打印标识 | string |
| LESSION | warehouse_number | 仓库编号 | string |
| SESSION | special_stock_indicator | 特殊库存标识 | string |

### 5.2 物料凭证行项目 (原表: MSEG)

| 原字段名 | 脱敏后字段名 | 中文含义 | Proto类型 |
|---------|-------------|---------|----------|
| MBLNR | document_number | 物料凭证号 | string |
| MJAHR | document_year | 凭证年度 | int32 |
| ZEESSION | item_number | 行项目号 | int32 |
| BWART | movement_type | 移动类型 | string |
| XAUTO | auto_created | 自动创建 | bool |
| MATNR | material | 物料编号 | string |
| WERKS | plant | 工厂 | string |
| LGORT | storage_location | 存储地点 | string |
| CHARG | batch | 批次 | string |
| INSMK | stock_type | 库存类型 | string |
| ZUESSION | receiving_storage_location | 接收存储地点 | string |
| ZESSION | receiving_batch | 接收批次 | string |
| SOBESSION | special_stock_indicator | 特殊库存标识 | string |
| KZBEW | movement_indicator | 移动标识 | string |
| KZVBR | consumption_indicator | 消耗标识 | string |
| KZZUG | receipt_indicator | 收货标识 | string |
| SHKZG | debit_credit_indicator | 借贷标识 | string |
| WAESSION | goods_movement_status | 货物移动状态 | string |
| BWTAR | valuation_type | 评估类型 | string |
| MENGE | quantity | 数量 | string |
| MEINS | unit | 单位 | string |
| ERFMG | entry_quantity | 录入数量 | string |
| ERFME | entry_unit | 录入单位 | string |
| DMBTR | amount_local | 本位币金额 | string |
| WAERS | currency | 货币代码 | string |
| BNBTR | purchase_value | 采购价值 | string |
| BUALT | amount_posting | 过账金额 | string |
| EXBWR | external_amount | 外部金额 | string |
| EXVKW | external_value | 外部价值 | string |
| SALK3 | total_stock_value | 库存总值 | string |
| VPRSV | price_control | 价格控制 | string |
| VERPR | moving_average_price | 移动平均价 | string |
| STPRS | standard_price | 标准价格 | string |
| PEINH | price_unit | 价格单位 | int32 |
| LBESSION | warehouse_number | 仓库编号 | string |
| LGTYP | storage_type | 存储类型 | string |
| LGPLA | storage_bin | 存储仓位 | string |
| BESTQ | stock_category | 库存类别 | string |
| SOBKZ | special_stock | 特殊库存 | string |
| VBELN | delivery_number | 交货单号 | string |
| VBELP | delivery_item | 交货项目 | int32 |
| VGART | transaction_type | 事务类型 | string |
| LGNUM | warehouse_complex | 仓库复合体 | string |
| TBNUM | transfer_requirement | 转储需求 | string |
| TBPOS | transfer_requirement_item | 转储需求项目 | int32 |
| TESSION | transfer_order | 转储订单 | string |
| WEESSION | transfer_order_item | 转储订单项目 | int32 |
| EBELN | purchase_order | 采购订单号 | string |
| EBELP | purchase_order_item | 采购订单项目 | int32 |
| LFBJA | delivery_year | 交货年度 | int32 |
| LFBNR | delivery_document | 交货凭证 | string |
| LFPOS | delivery_document_item | 交货凭证项目 | int32 |
| SJAHR | fiscal_year | 会计年度 | int32 |
| SMBLN | material_document_reversed | 被冲销物料凭证 | string |
| SMBLP | material_document_item_reversed | 被冲销物料凭证项目 | int32 |
| ELIKZ | delivery_completed | 交货完成 | bool |
| SGTXT | item_text | 行项目文本 | string |
| EQUNR | equipment | 设备编号 | string |
| AUFNR | order | 订单编号 | string |
| KOSTL | cost_center | 成本中心 | string |
| PRCTR | profit_center | 利润中心 | string |
| PS_PSP_PNR | wbs_element | WBS元素 | string |
| NPLNR | network | 网络 | string |
| AUFPL | routing_number | 工艺路线编号 | string |
| APLZL | routing_counter | 工艺路线计数器 | int32 |
| RSNUM | reservation_number | 预留编号 | string |
| RSPOS | reservation_item | 预留项目 | int32 |
| UMWRK | receiving_plant | 接收工厂 | string |
| UMLGO | receiving_storage_loc | 接收存储地点 | string |
| UMCHA | receiving_batch | 接收批次 | string |
| UMBAR | receiving_valuation_type | 接收评估类型 | string |
| UMSOK | receiving_special_stock | 接收特殊库存 | string |
| KUNNR | customer | 客户编号 | string |
| LIFNR | supplier | 供应商编号 | string |
| KDAUF | sales_order | 销售订单号 | string |
| KDPOS | sales_order_item | 销售订单项目 | int32 |
| KDEIN | sales_order_schedule | 销售订单计划行 | int32 |
| PLPLA | planning_plant | 计划工厂 | string |
| MESSION | segment | 分部 | string |
| GRANT_NBR | grant_number | 拨款编号 | string |
| BUDGET_PD | budget_period | 预算期间 | string |
| KSTRG | cost_object | 成本对象 | string |
| PAOESSION | profitability_segment | 获利能力段 | string |
| GSBER | business_area | 业务范围 | string |
| FKBER | functional_area | 功能范围 | string |
| GESSION | commitment_item | 承诺项目 | string |
| FISTL | funds_center | 资金中心 | string |
| FIESSION | fund | 资金 | string |
| KBLNR | document_number_earmarked | 预留凭证号 | string |
| KBLPOS | item_number_earmarked | 预留项目号 | int32 |

## 6. 关键字脱敏映射表

以下是本设计中使用的关键字脱敏映射：

| 原始术语 | 脱敏后术语 | 说明 |
|---------|-----------|------|
| SAP | ERP | 企业资源计划系统 |
| S/4HANA | Core ERP | 核心ERP系统 |
| BAPI | API | 应用程序接口 |
| RFC | RPC | 远程过程调用 |
| IDoc | EDI Message | 电子数据交换消息 |
| ABAP | Backend Logic | 后端逻辑 |
| Fiori | Web UI | Web用户界面 |
| BTP | Cloud Platform | 云平台 |
| CDS View | Data View | 数据视图 |
| AMDP | Stored Procedure | 存储过程 |
| OData | REST API | RESTful接口 |
| HANA | In-Memory DB | 内存数据库 |
| Customizing | Configuration | 配置 |
| IMG | Config Guide | 配置指南 |
| SPRO | System Config | 系统配置 |
| Tcodes | Transactions | 事务 |
| Enhancement | Extension | 扩展 |
| BAdI | Extension Point | 扩展点 |
| User Exit | Custom Hook | 自定义钩子 |
| CMOD/SMOD | Extension Framework | 扩展框架 |
| Smartforms | Print Template | 打印模板 |
| Adobe Forms | PDF Template | PDF模板 |
| ALV | Data Grid | 数据网格 |
| Dynpro | Screen | 屏幕 |
| Selection Screen | Filter Screen | 筛选屏幕 |
| Variant | Saved Filter | 保存的筛选条件 |
| Authorization Object | Permission | 权限 |
| Profile | Permission Set | 权限集 |
| Role | Access Role | 访问角色 |
| Transport Request | Change Request | 变更请求 |
| Workbench | Development | 开发 |
| Customizing Request | Config Change | 配置变更 |
| Client | Tenant | 租户 |
| Mandant | Tenant ID | 租户ID |
| Company Code | Legal Entity | 法人实体 |
| Controlling Area | Management Unit | 管理单元 |
| Plant | Site | 站点/工厂 |
| Storage Location | Warehouse Zone | 仓库区域 |
| Profit Center | Profit Unit | 利润单元 |
| Cost Center | Cost Unit | 成本单元 |
| Internal Order | Cost Collector | 成本收集器 |
| WBS Element | Project Element | 项目元素 |
| Network | Project Network | 项目网络 |
| Activity | Project Activity | 项目活动 |
| Milestone | Project Milestone | 项目里程碑 |
| Material Master | Product Master | 产品主数据 |
| Vendor Master | Supplier Master | 供应商主数据 |
| Customer Master | Customer Master | 客户主数据 |
| G/L Account | Ledger Account | 分类账科目 |
| Reconciliation Account | Control Account | 统驭科目 |
| Document Type | Posting Type | 过账类型 |
| Posting Key | Entry Key | 录入码 |
| Movement Type | Stock Movement | 库存移动类型 |
| Valuation Class | Asset Class | 资产类别 |
| Depreciation Area | Depreciation Book | 折旧账簿 |
| Asset Class | Fixed Asset Type | 固定资产类型 |
