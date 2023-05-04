{inputs, ...}: {
  imports = [
    inputs.impermanence.nixosModules.impermanence
  ];
  fileSystems."/" = {
    device = "tmpfs";
    fsType = "tmpfs";
    options = ["relatime" "size=2G" "mode=755"]; # mode=755 so only root can write to those files
  };

  fileSystems."/nix" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };

  boot = {
    growPartition = true;
    kernelParams = ["console=ttyS0" "preempt=none"];
    loader.grub = {
      device = "/dev/vda";
    };
    loader.timeout = 0;
  };

  environment.persistence."/nix/persist" = {
    hideMounts = true;
    directories = [
      "/var/lib/chrony"
      "/var/lib/nixos"
      "/var/lib/systemd"
      "/run/k3s/containerd"
      "/var/log"
      "/root"
      # "/boot"
    ];
    # files = [
    #   "/etc/machine-id"
    #   "/etc/ssh/ssh_host_ed25519_key.pub"
    #   "/etc/ssh/ssh_host_ed25519_key"
    #   "/etc/ssh/ssh_host_rsa_key.pub"
    #   "/etc/ssh/ssh_host_rsa_key"
    # ];
  };
}
