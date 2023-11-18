{
  pkgs,
  lib,
  ...
}: {
  programs.fish.enable = true;

  # declare the gaming user and its fixed password
  users.mutableUsers = false;
  users.users.root = {
    isSystemUser = true;
    shell = pkgs.fish;
    extraGroups = ["networkmanager" "wheel"];
    password = "faas";
    # hashedPassword = "$6$qi8XAsi7E.eVCsQK$7xIDTcn0g3h9iRGU3IMBBq7e53oTC5dDSa3qn/2EmIjO.nvNvfDq2OiEBiw8aDWLxkAiuuo.BcBdCtAK6p6Y71"; # faas
    # openssh.authorizedKeys.keys = readLines ../config/id_rsa.pub;# Acutally mounted by enos
  };
  programs.vim.defaultEditor = false;
  security.sudo.enable = false;
  services = {
    openssh = {
      enable = true;
      settings = {
        PermitRootLogin = "yes";
        X11Forwarding = false;
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
        ]))
      ansible
      fping
      kubernetes-helm

      htop
    ];

    # Set caching for g5k registry
    etc."rancher/k3s/registries.yaml".text = ''
      mirrors:
        docker.io:
          endpoint:
            - "http://docker-cache.grid5000.fr"
          configs:
            "http://docker-cache.grid5000.fr":
              tls:
                insecure_skip_verify: true
    '';

    # Tell the Nix evaluator to garbage collect morke aggressively.
    # This is desirable in memory-constrained environments that don't
    # (yet) have swap set up.
    variables.GC_INITIAL_HEAP_SIZE = "1M";
  };

  networking = {
    firewall = {
      enable = lib.mkForce false;
      allowPing = true;
    };
    useNetworkd = false;
    useDHCP = true;
  };

  system.stateVersion = "22.05"; # Do not change
}
