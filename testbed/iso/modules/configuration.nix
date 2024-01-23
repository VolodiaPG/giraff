{
  pkgs,
  lib,
  ...
}: let
  # readLines = file: builtins.filter (x: x != "") (lib.strings.splitString "\n" (builtins.readFile file));
  readLines = file: lib.strings.splitString "\n" (builtins.readFile file);
in {
  services = {
    chrony.enable = true;
    chrony.servers = readLines ../config/ntp-servers.txt;
  };

  programs.fish.shellAliases = {
    kubectl = "k3s kubectl";
    k = "kubectl";
    k9 = "k9s --kubeconfig /etc/rancher/k3s/k3s.yaml -A";
  };

  virtualisation.docker.enable = true;

  services.k3s = {
    enable = true;
  };
  # useful packages
  environment.systemPackages = with pkgs; [
    faas-cli
    kubectl
    arkade
    tailscale
    k9s
  ];

  services.tailscale.enable = true;

  system.stateVersion = "22.05"; # Do not change
}
