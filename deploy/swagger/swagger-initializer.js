window.onload = function() {
  window.ui = SwaggerUIBundle({
    urls: [
      { url: "./specs/auth.json", name: "身份认证 (IAM)" },
      { url: "./specs/finance.json", name: "财务 (Finance)" },
      { url: "./specs/procurement.json", name: "采购 (Procurement)" },
      { url: "./specs/sales.json", name: "销售 (Sales)" },
      { url: "./specs/supplychain.json", name: "供应链 (Supply Chain)" },
      { url: "./specs/asset.json", name: "资产管理 (Asset)" },
      { url: "./specs/manufacturing.json", name: "制造 (Manufacturing)" },
      { url: "./specs/service.json", name: "客户服务 (Service)" },
      { url: "./specs/rd.json", name: "研发 (R&D)" },
      { url: "./specs/hr.json", name: "人力资源 (HR)" }
    ],
    dom_id: '#swagger-ui',
    deepLinking: true,
    presets: [
      SwaggerUIBundle.presets.apis,
      SwaggerUIStandalonePreset
    ],
    plugins: [
      SwaggerUIBundle.plugins.DownloadUrl
    ],
    layout: "StandaloneLayout",
    docExpansion: "none",
    defaultModelsExpandDepth: -1,
    persistAuthorization: true
  });
};
