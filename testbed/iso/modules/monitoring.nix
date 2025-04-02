{
  pkgs,
  config,
  lib,
  ...
}: let
  influxToken = "xowyTh1iGcNAZsZeydESOHKvENvcyPaWg8hUe3tO4vPOw_buZVwOdUrqG3gwV314aYd9SWKHcxlykcQY_rwYVQ==";

  opentelemetry-config = pkgs.writeText "opentelemetry-collector.yaml" ''
    receivers:
      otlp:
        protocols:
          grpc:
            endpoint: 0.0.0.0:4317

    processors:
      batch:
        timeout: 1s
        send_batch_size: 1024

    exporters:
      influxdb:
        endpoint: "http://localhost:9086"
        org: "faasfog"
        bucket: "faasfog"
        token: "${influxToken}"
        timeout: 5s
        metrics_schema: "telegraf-prometheus-v1"

    service:
      pipelines:
        metrics:
          receivers: [otlp]
          exporters: [influxdb]
        traces:
          receivers: [otlp]
          exporters: [influxdb]
        logs:
          receivers: [otlp]
          exporters: [influxdb]
  '';
in {
  services = {
    influxdb2 = {
      enable = true;
      settings = {
        http-bind-address = "0.0.0.0:9086";
        auth-enabled = false;
        log-enabled = false;
        write-tracing = false;
        pprof-enabled = false;
        https-enabled = false;
      };
    };
    opentelemetry-collector = {
      enable = true;
      package = pkgs.opentelemetry-collector-contrib;
      configFile = opentelemetry-config;
    };
    grafana = {
      enable = false;
      #   addr = "0.0.0.0";
      settings = {
        server.http_port = 9030;
        server.http_addr = "0.0.0.0";
        #   domain = "0.0.0.0:9030";
        "auth.anonymous" = {
          enable = true;
          org_role = "Admin";
          org_name = "Main Org.";
        };
      };
      # security.adminUser = "admin";
      # security.adminPassword = "admin";
      provision = {
        enable = true;
        datasources.settings.datasources = [
          {
            name = "influxdb";
            type = "influxdb";
            access = "proxy";
            #   user = "admin";
            #   secureJsonData = { password = "adminfaasfog"; };
            url = "http://${toString config.services.influxdb2.settings.http-bind-address}";
            isDefault = true;
            jsonData = {
              version = "Flux";
              organization = "faasfog";
              defaultBucket = "faasfog";
              tlsSkipVerify = true;
            };
            secureJsonData = {
              token = influxToken;
            };
          }
        ];
      };
    };
  };
  systemd.services = {
    opentelemetry-collector = {
      serviceConfig = {
        ExecStartPre = "${pkgs.coreutils}/bin/test -f /etc/opentelemetry-config/collector.yaml";
        ExecStart =
          lib.mkForce "${lib.getExe pkgs.opentelemetry-collector-contrib} --config=file:${opentelemetry-config} --config=file:/etc/opentelemetry-config/collector.yaml";
        RestartSec = "3s";
      };
    };

    "init-influxdb2" = {
      wantedBy = ["influxdb2.service"];
      after = ["influxdb2.service"];
      serviceConfig = {
        ExecStart = let
          influxSettings = config.services.influxdb2.settings;
          script = pkgs.writeScript "influxdb2-init" ''
            #!${pkgs.runtimeShell}
            until ${pkgs.curl}/bin/curl -s -f -o /dev/null "http://${toString influxSettings.http-bind-address}"
            do
                sleep 5
            done

            ${pkgs.influxdb2-cli}/bin/influx setup \
                --host http://${toString influxSettings.http-bind-address} \
                --username admin \
                --password adminfaasfog \
                --token ${influxToken} \
                --org faasfog \
                --bucket faasfog \
                --force
          '';
        in "${script} %u";
      };
    };
  };
}
