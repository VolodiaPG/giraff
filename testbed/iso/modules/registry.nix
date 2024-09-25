{pkgs, ...}: let
  listenAddress = "0.0.0.0";
  port = 5555;
  certificate = "${pkgs.path}/nixos/tests/common/acme/server/acme.test.cert.pem";
  key = "${pkgs.path}/nixos/tests/common/acme/server/acme.test.key.pem";
in {
  services.dockerRegistry = {
    enable = true;
    inherit port;
    inherit listenAddress;
    extraConfig = {
      http.tls = {
        inherit certificate key;
      };
      storage = {
        filesystem = {
          rootdirectory = "/var/lib/docker-registry";
        };
      };
    };
  };
  networking.firewall.allowedTCPPorts = [port];
  environment.etc."containers/policy.json".text = ''
    {
        "default": [
            {
                "type": "insecureAcceptAnything"
            }
        ]
    }
  '';
}
