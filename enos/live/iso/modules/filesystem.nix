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
      "/var/lib/containers" # podman caches
      "/run/k3s/containerd" # K3S caches
      "/var/log"
      "/root"
    ];
  };
}