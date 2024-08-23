{
  pkgs,
  config,
  outputs,
  ...
}: let
  influxSettings = config.services.influxdb2.settings;
  influxToken = "xowyTh1iGcNAZsZeydESOHKvENvcyPaWg8hUe3tO4vPOw_buZVwOdUrqG3gwV314aYd9SWKHcxlykcQY_rwYVQ==";
in {
  systemd.services.proxy = {
    description = "Start the proxy server";
    wantedBy = ["multi-user.target"];
    serviceConfig = {
      Environment = [
        "PORT=3128"
        "INFLUX_ADDRESS=${toString influxSettings.http-bind-address}"
        "INFLUX_ORG=faasfog"
        "INFLUX_BUCKET=faasfog"
        "INFLUX_TOKEN=${toString influxToken}"
        #"DEV=true"
      ];
      ExecStart = "${outputs.packages.${pkgs.system}.proxy}/bin/proxy";
      Restart = "on-failure";
      RestartSec = "3";
    };
  };
}
