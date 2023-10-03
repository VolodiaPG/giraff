{
  pkgs,
  outputs,
  ...
}: {
  #   services.squid.enable = true;
  #   services.squid.extraConfig = ''
  #     cache deny all
  #     http_access allow all
  #     tcp_outgoing_address 0.0.0.0 all
  #     max_filedescriptors 4096
  #     cache_mem 256 MB

  #     icap_enable on
  #     icap_preview_enable on
  #     icap_preview_size 4096
  #     icap_service service_req reqmod_precache bypass=0 icap://127.0.0.1:11344/request
  #     adaptation_access service_req allow all
  #   '';

  #   systemd.services.icap-wait = {
  #     description = "Wait for the icap server";
  #     after = [ "icap.service" ];
  #     wantedBy = [ "squid.service" ];
  #     before = ["squid.service" ];
  #     script = ''
  #       until ${pkgs.netcat}/bin/nc -z -v -w1 127.0.0.1 11344 2>/dev/null; do
  #         echo Waiting for port 11344 to open
  #         sleep 2
  #       done
  #       sleep 2
  #     '';
  #     serviceConfig = {
  #       Type = "oneshot";
  #     };
  #   };

  # # curl -X POST -x http://localhost:3128 10.0.2.2:8000 --data "toto"

  #   systemd.services.icap = {
  #     description = "Start the icap server";
  #     after = [ "network.target" ];
  #     wantedBy = [ "squid.service" ];
  #     serviceConfig = {
  #       User = "squid";
  #       ExecStart = "${outputs.packages.${pkgs.system}.icap}/bin/icap";
  #     };
  #   };
  systemd.services.proxy = {
    description = "Start the proxy server";
    after = ["network.target"];
    wantedBy = ["multi-user.target"];
    serviceConfig = {
      # User = "squid";
      Environment = "PORT=3128";
      ExecStart = "${outputs.packages.${pkgs.system}.proxy}/bin/proxy";
    };
  };
}
