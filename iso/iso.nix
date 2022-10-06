{ config, pkgs, ... }:
{

  # compress 6x faster than default
  # but iso is 15% bigger
  # tradeoff acceptable because we don't want to distribute
  # default is xz which is very slow
  isoImage.squashfsCompression = "zstd -Xcompression-level 6";

  # my azerty keyboard
  i18n.defaultLocale = "fr_FR.UTF-8";
  services.xserver.layout = "fr";
  console = {
    keyMap = "fr";
  };

  # xanmod kernel for better performance
  # see https://xanmod.org/
  # boot.kernelPackages = pkgs.linux_hardened;

  # getting IP from dhcp
  # no network manager
  networking.dhcpcd.enable = true;
  # networking.networkmanager.enable = true; # Easiest to use and most distros use this by default.
  #networking.hostName = "faas-fog"; # Define your hostname.

  services.xserver.xkbOptions = "eurosign:e";

  time.timeZone = "Europe/Paris";

  # declare the gaming user and its fixed password
  users.mutableUsers = false;
  #users.users.nixos = {
  #  isNormalUser = true;
  #  shell = pkgs.fish;
  #  extraGroups = [ "networkmanager" "wheel" ];
  #  hashedPassword = "$6$7pE7b8uqvt/XVmgo$Wlznz/v04VkDGxMUCxk9UBERHrMZqtrRlUAqxXYOvck/MKMS1A9FV6oH29qkWpPt/zqiC3Opuhp7QKBDOk62b."; # faas
  #  openssh.authorizedKeys.keys = [ "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCpDmkY5OctLdxrPUcnRafndhDvvgw/GNYvgo4I9LrPJ341vGzwgqSi90YRvn725DkYHmEi1bN7i3W2x1AQbuvEBzxMG3BlwtEGtz+/5rIMY+5LRzB4ppN+Ju/ySbPKSD2XpVgVOCegc7ZtZ4XpAevVsi/kyg35RPNGmljEyuN1wIxBVARZXZezsGf1MHzxEqiNogeAEncPCk/P44B6xBRt9qSxshIT/23Cq3M/CpFyvbI0vtdLaVFIPox6ACwlmTgdReC7p05EefKEXaxVe61yhBquzRwLZWf6Y8VESLFFPZ+lEF0Shffk15k97zJICVUmNPF0Wfx1Fn5tQyDeGe2nA5d2aAxHqvl2mJk/fccljzi5K6j6nWNf16pcjWjPqCCOTs8oTo1f7gVXQFCzslPnuPIVUbJItE3Ui+mSTv9KF/Q9oH02FF40mSuKtq5WmntV0kACfokRJLZ6slLabo0LgVzGoixdiGwsuJbWAsNNHURoi3lYb8fMOxZ/2o4GZik= volodia@volodia-msi" ];
  #};
  users.users.root = {
    isSystemUser = true;
    shell = pkgs.bash;
    extraGroups = [ "networkmanager" "wheel" ];
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

  services.qemuGuest.enable = true;

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
  ];
  
  # services.grafana = {
  #     enable = true;
  #     # Listening address and TCP port
  #     addr = "127.0.0.1";
  #     port = 3000;
  #     # Grafana needs to know on which domain and URL it's running:
  #     domain = "0.0.0.0";
  #     #rootUrl = "http://your.domain/grafana/"; # Not needed if it is `https://your.domain/`
  # };
  # services.telegraf.enable = true;

  # Forward faas-cli
  # systemd.services.openfaas-forward = {
  #   description = "OpenFaaS port-forward";
  #   wantedBy = [ "multi-user.target" ];
  #   serviceConfig = {
  #     Type = "simple";
  #     Restart = "always";
  #     ExecStart = "env KUBECONFIG=/etc/rancher/k3s/k3s.yaml ${pkgs.k3s}/bin/k3s kubectl port-forward -n openfaas svc/gateway 8080:8080";
  #   };
  # };

  environment.etc."modprobe.d/floppy.blacklist.conf".text = ''
    blacklist floppy
  '';
  
  networking.firewall.enable = false;
  
  system.stateVersion = "22.05"; # Do not change
}
