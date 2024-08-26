{
  lib,
  config,
  ...
}: {
  disko.extraRootModules = ["zfs"];
  disko.devices = {
    disk = {
      sda = {
        type = "disk";
        device = " /dev/disk/by-diskseq/1";
        content = {
          type = "gpt";
          partitions = {
            MBR = {
              size = "1M";
              type = "EF02"; # for grub MBR
              priority = 1; # Needs to be first partition
            };
            ESP = {
              size = "100M";
              type = "EF00";
              content = {
                type = "filesystem";
                format = "vfat";
                mountpoint = "/boot";
              };
            };
            plainSwap = {
              size = "4G";
              content = {
                type = "swap";
              };
            };
            rpool = {
              size = "100%";
              content = {
                type = "zfs";
                pool = "rpool";
              };
            };
          };
        };
      };
    };
    zpool = {
      "rpool" = {
        type = "zpool";
        rootFsOptions = {
          acltype = "posixacl";
          dnodesize = "auto";
          canmount = "off";
          xattr = "sa";
          atime = "off";
          relatime = "on";
          normalization = "formD";
          mountpoint = "none";
          compression = "zstd";
          "com.sun:auto-snapshot" = "false";
          devices = "off";
        };
        options = {
          ashift = "12";
          autotrim = "on";
        };

        datasets = {
          local = {
            type = "zfs_fs";
            options.mountpoint = "none";
          };
          safe = {
            type = "zfs_fs";
            options.mountpoint = "none";
          };
          "local/root" = {
            type = "zfs_fs";
            mountpoint = "/";
            options.mountpoint = "legacy";
            postCreateHook = ''
              zfs snapshot rpool/local/root@blank
            '';
          };
          "local/nix" = {
            type = "zfs_fs";
            mountpoint = "/nix";
            options = {
              atime = "off";
              canmount = "on";
              mountpoint = "legacy";
            };
          };
          "safe/persistent" = {
            type = "zfs_fs";
            mountpoint = "/persistent";
            options = {
              mountpoint = "legacy";
            };

            # ${builtins.concatStringsSep "; " (builtins.map (dir: "mkdir -p /" + dir) directoriesToBind)}
            postMountHook = with lib; let
              directoriesList = config.environment.persistence."/persistent".directories;
              directories = builtins.map (set: "\"" + set.directory + "\"") directoriesList;

              dirname = path: let
                components = strings.splitString "/" path;
                length = builtins.length components;
                dirname = builtins.concatStringsSep "/" (lists.take (length - 1) components);
              in
                dirname;
              filesList = map (set: set.file) config.environment.persistence."/persistent".files;
              files = builtins.map dirname filesList;

              directoriesToBind = directories ++ files;
            in ''
              ${builtins.concatStringsSep "; " (builtins.map (dir: "mkdir -p /mnt/persistent" + dir) directoriesToBind)}
              ${builtins.concatStringsSep "; " (builtins.map (dir: "touch /mnt/persistent" + dir) filesList)}
            '';
          };
        };
      };
    };
  };
}
