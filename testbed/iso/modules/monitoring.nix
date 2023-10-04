{
  pkgs,
  config,
  ...
}: let
  influxToken = "xowyTh1iGcNAZsZeydESOHKvENvcyPaWg8hUe3tO4vPOw_buZVwOdUrqG3gwV314aYd9SWKHcxlykcQY_rwYVQ==";
in {
  services.influxdb2 = {
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

  systemd.services = {
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

  services.grafana = {
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
}
