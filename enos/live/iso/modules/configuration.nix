{
  pkgs,
  lib,
  ...
}: let
  # readLines = file: builtins.filter (x: x != "") (lib.strings.splitString "\n" (builtins.readFile file));
  readLines = file: lib.strings.splitString "\n" (builtins.readFile file);
in {
  time.timeZone = "Europe/Paris";

  services.chrony.enable = true;
  services.chrony.servers = readLines ../config/ntp-servers.txt;

  # declare the gaming user and its fixed password
  users.mutableUsers = false;
  users.users.root = {
    isSystemUser = true;
    shell = pkgs.fish;
    extraGroups = ["networkmanager" "wheel"];
    password = "faas";
    # hashedPassword = "$6$qi8XAsi7E.eVCsQK$7xIDTcn0g3h9iRGU3IMBBq7e53oTC5dDSa3qn/2EmIjO.nvNvfDq2OiEBiw8aDWLxkAiuuo.BcBdCtAK6p6Y71"; # faas
    openssh.authorizedKeys.keys = readLines ../config/id_rsa.pub;
  };

  services.openssh = {
    enable = true;
    permitRootLogin = "yes";
  };
  services.fwupd.enable = true;

  services.k3s = {
    enable = true;
  };

  virtualisation.docker.enable = true;

  # useful packages
  environment.systemPackages = with pkgs; [
    faas-cli
    kubectl
    arkade

    # enoslib necessities
    (python3.withPackages (p:
      with p; [
        requests
      ]))
    ansible
    fping
    kubernetes-helm

    htop
  ];

  # Set caching for g5k registry
  environment.etc."rancher/k3s/registries.yaml".text = ''
    mirrors:
      docker.io:
        endpoint:
          - "http://docker-cache.grid5000.fr"
        configs:
          "http://docker-cache.grid5000.fr":
            tls:
              insecure_skip_verify: true
  '';

  networking.firewall.enable = false;

  # Tell the Nix evaluator to garbage collect morke aggressively.
  # This is desirable in memory-constrained environments that don't
  # (yet) have swap set up.
  environment.variables.GC_INITIAL_HEAP_SIZE = "1M";

  boot.kernel.sysctl."net.ipv4.tcp_congestion_control" = "bbr2";

  system.stateVersion = "22.05"; # Do not change
}
