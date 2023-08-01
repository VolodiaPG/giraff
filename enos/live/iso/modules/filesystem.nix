{inputs, ...}: {
  imports = [
    inputs.impermanence.nixosModules.impermanence
  ];
  fileSystems."/" = {
    device = "none";
    fsType = "tmpfs";
    options = ["defaults" "relatime" "size=2G" "mode=755"]; # mode=755 so only root can write to those files
  };

  fileSystems."/nix" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };

  boot = {
    growPartition = true;
    kernelParams = ["console=ttyS0"]; # "preempt=none"];
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
      "/var/lib/containers" # podman caches
      "/run/k3s/containerd" # K3S caches
      "/var/log"
      "/root"
    ];
    files = [
      # Preserve influxdb login informations as created initially in Nix
      "/etc/influxdb/influxdb.conf"
      "/var/lib/influxdb2/influxd.bolt"
      "/var/lib/influxdb2/influxd.sqlite"
      # Preserve ssh info
      # "/etc/ssh/ssh_host_rsa_key"
      # "/etc/ssh/ssh_host_rsa_key.pub"
      # "/etc/ssh/ssh_host_ed25519_key"
      # "/etc/ssh/ssh_host_ed25519_key.pub"
    ];
  };
}
