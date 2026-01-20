window.onload = function() {
  //<editor-fold desc="Changeable Configuration Block">

  // the following lines will be replaced by docker/configurator, when it runs in a docker-container
  window.ui = SwaggerUIBundle({
    urls: [
      {
        url: "/specs/auth-service.yaml",
        name: "Auth Service"
      },
      {
        url: "/specs/rbac-service.yaml",
        name: "RBAC Service"
      },
      {
        url: "/specs/gl-service.yaml",
        name: "GL Service"
      },
      {
        url: "/specs/ap-service.yaml",
        name: "AP Service"
      },
      {
        url: "/specs/ar-service.yaml",
        name: "AR Service"
      },
      {
        url: "/specs/coa-service.yaml",
        name: "COA Service"
      }
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
    layout: "StandaloneLayout"
  });

  //</editor-fold>
};
