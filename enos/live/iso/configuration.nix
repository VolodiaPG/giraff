{ config, pkgs, ... }:
{
  # Filesystems
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    autoResize = true;
    fsType = "ext4";
  };

  boot = {
    growPartition = true;
    kernelParams = [ "console=ttyS0" ];
    loader.grub = {
      device = "/dev/vda";
    };
    loader.timeout = 0;
  };

  time.timeZone = "Europe/Paris";

  systemd = {
    enableEmergencyMode = false;
    network = {
      wait-online = {
        anyInterface = true;
      };
    };
  };
  systemd.services.systemd-udev-settle.enable = false;

  services.chrony.enable = true;
  services.chrony.servers = [ "ntp.rennes.grid5000.fr" ];

  # declare the gaming user and its fixed password
  users.mutableUsers = false;
  users.users.root = {
    isSystemUser = true;
    shell = pkgs.fish;
    extraGroups = [ "networkmanager" "wheel" "docker" ];
    hashedPassword = "$6$7pE7b8uqvt/XVmgo$Wlznz/v04VkDGxMUCxk9UBERHrMZqtrRlUAqxXYOvck/MKMS1A9FV6oH29qkWpPt/zqiC3Opuhp7QKBDOk62b."; # faas
    openssh.authorizedKeys.keys = [
      "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCpDmkY5OctLdxrPUcnRafndhDvvgw/GNYvgo4I9LrPJ341vGzwgqSi90YRvn725DkYHmEi1bN7i3W2x1AQbuvEBzxMG3BlwtEGtz+/5rIMY+5LRzB4ppN+Ju/ySbPKSD2XpVgVOCegc7ZtZ4XpAevVsi/kyg35RPNGmljEyuN1wIxBVARZXZezsGf1MHzxEqiNogeAEncPCk/P44B6xBRt9qSxshIT/23Cq3M/CpFyvbI0vtdLaVFIPox6ACwlmTgdReC7p05EefKEXaxVe61yhBquzRwLZWf6Y8VESLFFPZ+lEF0Shffk15k97zJICVUmNPF0Wfx1Fn5tQyDeGe2nA5d2aAxHqvl2mJk/fccljzi5K6j6nWNf16pcjWjPqCCOTs8oTo1f7gVXQFCzslPnuPIVUbJItE3Ui+mSTv9KF/Q9oH02FF40mSuKtq5WmntV0kACfokRJLZ6slLabo0LgVzGoixdiGwsuJbWAsNNHURoi3lYb8fMOxZ/2o4GZik= volodia@volodia-msi"
      "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCy2YKtBVJb2Y52birMCUamwfVDVnWsvXbCoj7aipveHOCcBOIbeXV/DoOe/7U9es8FQ7L0bY/XaRzT+dExHsQl6RBYUbAoK5k+sirdGYOSaZd0cHZZL8TztBWrUtxbRHkmr4siLPXSKaTITmEU7LqcwSGusJ8wPjGAuzHVgefeET7oFl495xRLO9FQ1YRJlea/xfxMn4CrEtGDfvZHFkPjgWNcgXhwNNlEQwxLtVmojJy4ugrnKKah6VlqY72zKTgLDelVsmcvtGc+XA1lp8pCUJascTMTdvpca1+3IANkzUSZXLiWez09GVB+lt/2LHS1wQhw50dwSYNboaRkLjSZ Generated passwordless ssh key to move between sites and connect nodes"
    ];
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
    (python3.withPackages (p: with p; [
      requests
      docker
    ]))
    ansible
    fping
    kubernetes-helm

    htop
  ];

  networking.firewall.enable = false;

  # Tell the Nix evaluator to garbage collect more aggressively.
  # This is desirable in memory-constrained environments that don't
  # (yet) have swap set up.
  environment.variables.GC_INITIAL_HEAP_SIZE = "1M";

  # Make the installer more likely to succeed in low memory
  # environments.  The kernel's overcommit heustistics bite us
  # fairly often, preventing processes such as nix-worker or
  # download-using-manifests.pl from forking even if there is
  # plenty of free memory.
  boot.kernel.sysctl."vm.overcommit_memory" = "1";

  system.stateVersion = "22.05"; # Do not change
}
