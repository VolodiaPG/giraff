{
  inputs,
  pkgs,
  lib,
  ...
}: let
  readLines = file: lib.strings.splitString "\n" (builtins.readFile file);
in {
  imports = [
    inputs.impermanence.nixosModules.impermanence
  ];
  fileSystems = {
    "/" = {
      device = "none";
      fsType = "tmpfs";
      options = ["defaults" "relatime" "size=2G" "mode=755"]; # mode=755 so only root can write to those files
    };
    "/nix" = {
      device = "/dev/disk/by-label/nixos";
      fsType = "ext4";
    };
    "/lib/modules" = {
      device = "/run/current-system/kernel-modules/lib/modules";
      options = ["bind" "x-systemd.automount"];
    };
    # Bogus mount to import all tools and kernel extensions
    # However useless this mount is,
    # it still loads all necesary modules for the mounting to manually be invoked inside a script afterwardss
    "/mnt" = {
      device = "nfs:/export";
      fsType = "nfs";
      options = ["x-systemd.automount" "noauto"];
    };
  };

  boot = {
    growPartition = true;
    kernelParams = ["console=ttyS0"]; # "preempt=none"];
    loader.grub = {
      device = "/dev/vda";
    };
    loader.timeout = 0;
  };

  environment = {
    persistence."/nix/persist" = {
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
        "/root/.local/share/fish/fish_history"
      ];
    };
    systemPackages = [pkgs.nfs-utils];
  };

  systemd.services.mountNfs = {
    description = "Mount ssh";
    after = ["network.target"];
    wantedBy = ["multi-user.target"];
    script = ''
      mkdir -p /nfs
      ${pkgs.mount}/bin/mount ${builtins.elemAt (readLines ../config/g5k.nfs.txt) 0} /nfs
      mkdir -p /root/.ssh
      cp /nfs/.ssh/{authorized_keys,config,id_rsa,id_rsa.pub} /root/.ssh
      chmod 700 -R /root/.ssh
      chmod 600 /root/.ssh/*
      chmod 644 /root/.ssh/*.pub
    '';
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = "yes";
      Restart = "on-failure";
      RestartSec = "3";
    };
  };
}
