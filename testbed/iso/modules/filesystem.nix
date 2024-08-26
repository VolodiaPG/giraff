{
  inputs,
  pkgs,
  lib,
  config,
  ...
}: let
  readLines = file: lib.strings.splitString "\n" (builtins.readFile file);
in {
  boot = {
    zfs.extraPools = ["rpool"];
    kernelPackages = config.boot.zfs.package.latestCompatibleLinuxPackages;
    kernelParams = ["console=ttyS0"]; # "preempt=none"];
    loader.grub = {
      device = "nodev";
    };
    loader.timeout = 0;
    supportedFilesystems = ["zfs"];
    zfs.devNodes = "/dev/disk/by-partuuid";
    initrd.enable = true;
    initrd.postDeviceCommands = lib.mkAfter ''
      zfs rollback -r rpool/local/root@blank
    '';
  };
  systemd = {
    enableEmergencyMode = false;

    # Explicitly disable ZFS mount service since we rely on legacy mounts
    services.zfs-mount.enable = false;

    extraConfig = ''
      DefaultTimeoutStartSec=20s
      DefaultTimeoutStopSec=10s
    '';
  };
  services.zfs = {
    trim.enable = true;
    autoScrub = {
      enable = true;
      pools = ["rpool"];
    };
  };

  fileSystems = lib.mkMerge [
    {
      "/var/lib/rancher" = {
        device = "none";
        fsType = "tmpfs";
        options = ["defaults" "size=50%" "mode=755"];
      };
      "/persistent" = {
        neededForBoot = true;
      };
      "/lib/modules" = {
        device = "/run/current-system/kernel-modules/lib/modules";
        options = ["bind" "x-systemd.automount"];
      };
      # Bogus mount to import all tools and kernel extensions
      # However useless this mount is,
      # it still loads all necessary modules for the mounting to manually be invoked inside a script afterwardss
      "/mnt" = {
        device = "nfs:/export";
        fsType = "nfs";
        options = ["x-systemd.automount" "noauto"];
      };
    }
  ];
  environment = {
    persistence."/persistent" = {
      hideMounts = true;
      directories = [
        "/var/lib/chrony"
        "/var/lib/nixos"
        "/var/lib/systemd"
        "/var/lib/containers" # podman caches
        "/run/k3s/containerd" # K3S caches
        "/var/lib/rancher/k3s/agent/containerd"
        "/var/lib/docker/overlay2"
        "/var/lib/docker/image"
        "/var/lib/docker/containerd"
        "/var/log"
        "/root"
        #          "/etc/ssh"
      ];
      files = [
        # Preserve influxdb login information as created initially in Nix
        "/etc/influxdb/influxdb.conf"
        "/var/lib/influxdb2/influxd.bolt"
        "/var/lib/influxdb2/influxd.sqlite"
        "/root/.local/share/fish/fish_history"
      ];
    };
    systemPackages = [pkgs.nfs-utils];
  };

  # useful for debugging
  systemd.services = {
    sshx = {
      description = "sshx";
      wantedBy = ["multi-user.target"];
      script = ''
        dir=$(mktemp -d)
        ${pkgs.mount}/bin/mount ${builtins.elemAt (readLines ../config/g5k.nfs.txt) 0} $dir

        while ! [ -f "/my_group" ] ; do
          sleep 1
        done
        while ! [ -f "/my_name" ] ; do
          sleep 1
        done
        mkdir -p "$dir/sshx/$(cat /my_group)"
        export PATH=/run/current-system/sw/bin:$PATH
        export SHELL="fish"
        ${pkgs.lib.getExe inputs.nixpkgs.legacyPackages."${pkgs.stdenv.system}".sshx} -q | while IFS= read -r line; do printf "%-15s %s\n" "$(cat /my_name)" "$line"; done >> "$dir/sshx/$(cat /my_group)/$(cat /my_name).sshx"
      '';
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = "yes";
        Restart = "on-failure";
        RestartSec = "3";
      };
    };

    # useful for debugging
    tailscale-connect = {
      description = "tailscale-connect";
      wantedBy = ["multi-user.target"];
      script = ''
        dir=$(mktemp -d)
        ${pkgs.mount}/bin/mount ${builtins.elemAt (readLines ../config/g5k.nfs.txt) 0} $dir

        while ! [ -f "/my_name" ] ; do
          sleep 1
        done
        if [[ $(cat "/my_name") == "iot_emulation" ]]; then
          while ! [ -f "$dir/tailscale_authkey" ] ; do
            sleep 1
          done
          AUTH_KEY=$(cat "$dir/tailscale_authkey")

          ${pkgs.lib.getExe inputs.nixpkgs.legacyPackages."${pkgs.stdenv.system}".tailscale} up --authkey $AUTH_KEY --advertise-tags=tag:grid5000
        fi
      '';
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = "yes";
        Restart = "on-failure";
        RestartSec = "3";
      };
    };

    mountNfs = {
      description = "Mount ssh";
      wantedBy = ["multi-user.target"];
      script = ''
        mkdir -p /nfs
        ${pkgs.mount}/bin/mount ${builtins.elemAt (readLines ../config/g5k.nfs.txt) 0} /nfs

        mkdir -p /etc/ssh/authorized_keys.d
        cat /nfs/.ssh/authorized_keys > /etc/ssh/authorized_keys.d/root
        chmod 644 /etc/ssh/authorized_keys.d/root

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
  };
}
