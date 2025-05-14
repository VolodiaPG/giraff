{
  pkgs,
  lib,
  ...
}: let
  readLines = file: lib.strings.splitString "\n" (builtins.readFile file);
in {
  programs.fish.enable = true;

  # declare the gaming user and its fixed password
  users.mutableUsers = false;
  users.users.root = {
    isSystemUser = true;
    shell = pkgs.fish;
    extraGroups = ["networkmanager" "wheel"];
    password = "giraff";
    # hashedPassword = "$6$qi8XAsi7E.eVCsQK$7xIDTcn0g3h9iRGU3IMBBq7e53oTC5dDSa3qn/2EmIjO.nvNvfDq2OiEBiw8aDWLxkAiuuo.BcBdCtAK6p6Y71"; # faas
    # openssh.authorizedKeys.keys = readLines ../config/id_rsa.pub;# Actually mounted by enos
  };
  programs.vim.defaultEditor = false;
  security.sudo.enable = false;
  services = {
    openssh = {
      enable = true;
      allowSFTP = true;
      settings = {
        PermitRootLogin = "yes";
        X11Forwarding = false;
        KbdInteractiveAuthentication = lib.mkForce true;
        PasswordAuthentication = lib.mkForce true;
      };
    };
    fwupd.enable = true;
    resolved.enable = false;
  };

  # useful packages
  environment = {
    systemPackages = with pkgs; [
      # enoslib necessities
      (python3.withPackages (p:
        with p; [
          requests
          mitogen
        ]))
      fping
      kubernetes-helm

      htop
    ];

    # Set caching for g5k registry
    etc."rancher/k3s/registries.yaml".text = ''
      mirrors:
        "docker.io":
          endpoint:
            - "http://docker-cache.grid5000.fr"
        "*":
          endpoint:
            - "https://127.0.0.1:5555"
    '';

    # etc."containerd/certs.d/local".text = ''
    #   server = "https://127.0.0.1:5555"
    #
    #   [host."https://127.0.0.1:5555"]
    #     capabilities = ["pull", "resolve"]
    #     skip_verify = true
    #   '';
    # # Tell the Nix evaluator to garbage collect morke aggressively.
    # This is desirable in memory-constrained environments that don't
    # (yet) have swap set up.
    variables.GC_INITIAL_HEAP_SIZE = "1M";
  };

  virtualisation.containerd = {
    enable = true;
    settings = {
      config_path = "/etc/containerd/certs.d";
    };
  };

  networking = {
    firewall = {
      enable = lib.mkForce false;
      # allowPing = true;
    };
    useNetworkd = false;
    useDHCP = true;
  };

  # systemd.services.NetworkManager-wait-online.enable = true;
  # systemd.network.wait-online.enable = true;
  boot.kernel.sysctl = {
    "kernel.threads-max" = 2000000;
    "kernel.pid-max" = 2000000;
    "fs.file-max" = 204708;
    "vm.max_map_count" = 6000000;
    "net.core.default_qdisc" = lib.mkForce "cake"; #fq_codel also works but is older
    "net.ipv4.tcp_ecn" = 1;
    "net.ipv4.tcp_sack" = 1;
    "net.ipv4.tcp_dsack" = 1;
  };

  services = {
    chrony.enable = true;
    chrony.servers = readLines ../config/ntp-servers.txt;
  };

  system.stateVersion = "22.05"; # Do not change
}
