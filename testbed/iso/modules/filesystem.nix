{
  inputs,
  pkgs,
  lib,
  config,
  ...
}: let
  readLines = file: lib.strings.splitString "\n" (builtins.readFile file);
  rootVolume = "disk/by-partlabel/disk-sda-root";
in {
  boot = {
    kernelPackages = pkgs.linuxPackages_latest;
    kernelParams = ["console=ttyS0"]; # "preempt=none"];
    loader.grub = {
      device = "nodev";
    };
    loader.timeout = 0;
    supportedFilesystems = ["btrfs"];
    initrd.enable = true;
    initrd.postDeviceCommands = let
      directoriesList = config.environment.persistence."/persistent".directories;
      directories = builtins.map (set: "\"" + set.directory + "\"") directoriesList;

      dirname = path: let
        components = lib.strings.splitString "/" path;
        length = builtins.length components;
        dirname = builtins.concatStringsSep "/" (lib.lists.take (length - 1) components);
      in
        dirname;
      filesList = map (set: set.file) config.environment.persistence."/persistent".files;
      files = builtins.map dirname filesList;

      directoriesToBind = directories ++ files;
    in
      lib.mkAfter ''
        mkdir /btrfs_tmp
        diskpath=$(realpath /dev/${rootVolume})
        mount -t btrfs $diskpath /btrfs_tmp

        delete_subvolume_recursively() {
              IFS=$'\n'
              for i in $(btrfs subvolume list -o "$1" | cut -f 9- -d ' '); do
                  delete_subvolume_recursively "/btrfs_tmp/$i"
              done
              btrfs subvolume delete "$1"
          }

          delete_subvolume_recursively /btrfs_tmp/root

          btrfs subvolume create /btrfs_tmp/root

          mkdir -p /btrfs_tmp/root/boot
          mkdir -p /btrfs_tmp/root/nix
          mkdir -p /btrfs_tmp/root/persistent
          ${builtins.concatStringsSep "; " (builtins.map (dir: "mkdir -p /btrfs_tmp/root" + dir) directoriesToBind)}
          ${builtins.concatStringsSep "; " (builtins.map (dir: "mkdir -p /btrfs_tmp/persistent" + dir) directoriesToBind)}
          ${builtins.concatStringsSep "; " (builtins.map (dir: "touch /btrfs_tmp/persistent" + dir) filesList)}

          umount /btrfs_tmp
      '';
  };

  zramSwap = {
    enable = true;
    writebackDevice = "/dev/disk/by-partlabel/disk-sda-zram-writeback";
  };

  fileSystems = lib.mkMerge [
    {
      #"/var/lib/rancher" = {
      #  device = "none";
      #  fsType = "tmpfs";
      #  options = ["defaults" "size=50%" "mode=755"];
      #};
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
        #"/var/lib/nixos"
        #"/var/lib/systemd"
        #"/var/lib/containers" # podman caches
        "/run/k3s/containerd" # K3S caches
        "/var/lib/rancher/k3s/agent/containerd"
        #"/var/lib/docker/overlay2"
        #"/var/lib/docker/image"
        #"/var/lib/docker/containerd"
        "/var/log"
        "/root"
        {
          directory = "/var/lib/docker-registry";
          user = "docker-registry";
          group = "docker-registry";
        }
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
    # sshx = {
    #   description = "sshx";
    #   wantedBy = ["multi-user.target"];
    #   script = ''
    #     dir=$(mktemp -d)
    #     ${pkgs.mount}/bin/mount ${builtins.elemAt (readLines ../config/g5k.nfs.txt) 0} $dir

    #     while ! [ -f "/my_group" ] ; do
    #       sleep 1
    #     done
    #     while ! [ -f "/my_name" ] ; do
    #       sleep 1
    #     done
    #     mkdir -p "$dir/sshx/$(cat /my_group)"
    #     export PATH=/run/current-system/sw/bin:$PATH
    #     export SHELL="fish"
    #     ${pkgs.lib.getExe inputs.nixpkgs.legacyPackages."${pkgs.stdenv.system}".sshx} -q | while IFS= read -r line; do printf "%-15s %s\n" "$(cat /my_name)" "$line"; done >> "$dir/sshx/$(cat /my_group)/$(cat /my_name).sshx"
    #   '';
    #   serviceConfig = {
    #     Type = "oneshot";
    #     RemainAfterExit = "yes";
    #     Restart = "on-failure";
    #     RestartSec = "3";
    #   };
    # };

    # # useful for debugging
    tailscale-connect-iot = {
      description = "tailscale-connect-iot";
      wantedBy = ["multi-user.target"];
      after = ["mountNfs.service"];
      script = ''
        dir=/nfs
        while ! [ -e "$dir" ] ; do
          sleep 1
        done
        while ! [ -f "/my_name" ] ; do
          sleep 1
        done
        if [[ $(cat "/my_name") == "iot_emulation" ]]; then
          while ! [ -f "$dir/tailscale_authkey" ] ; do
            sleep 1
          done
          AUTH_KEY=$(cat "$dir/tailscale_authkey")

          ${pkgs.lib.getExe inputs.nixpkgs.legacyPackages."${pkgs.stdenv.system}".tailscale} up --authkey $AUTH_KEY --accept-dns=false --advertise-tags=tag:grid5000
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
  services.tailscale.enable = true;
}
