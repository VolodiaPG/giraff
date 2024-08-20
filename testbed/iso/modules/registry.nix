let
  listenAddress = "0.0.0.0";
  port = 5555;
in {
  services.dockerRegistry = {
    enable = true;
    inherit port;
    inherit listenAddress;
  };
  networking.firewall.allowedTCPPorts = [port];
}
