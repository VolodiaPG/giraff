{
  pkgs,
  outputs,
  ...
}: {
  systemd.services.proxy = {
    description = "Start the proxy server";
    after = ["network.target"];
    wantedBy = ["multi-user.target"];
    serviceConfig = {
      Environment = "PORT=3128";
      ExecStart = "${outputs.packages.${pkgs.system}.proxy}/bin/proxy";
    };
  };
}
