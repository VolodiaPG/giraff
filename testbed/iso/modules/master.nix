{
  inputs,
  pkgs,
  lib,
  ...
}: {
  services.tailscale.enable = true;
  systemd.services = {
    # useful for debugging
    tailscale-connect = {
      description = "tailscale-connect";
      wantedBy = ["multi-user.target"];
      after = ["mountNfs.service"];
      script = ''
        dir=/nfs
        while ! [ -d "$dir" ] ; do
          sleep 1
        done
        while ! [ -f "$dir/tailscale_authkey" ] ; do
          sleep 1
        done
        AUTH_KEY=$(cat "$dir/tailscale_authkey")

        ${lib.getExe inputs.nixpkgs.legacyPackages."${pkgs.stdenv.system}".tailscale} up --authkey $AUTH_KEY --accept-dns=false --advertise-tags=tag:grid5000
      '';
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = "yes";
        Restart = "on-failure";
        RestartSec = "3";
      };
    };
  };
}
