let
  mountOptions = ["ssd" "compress=zstd:2" "noatime" "discard=async" "space_cache=v2"];
in {
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
            root = {
              size = "100%";
              content = {
                type = "btrfs";
                extraArgs = ["-f"]; # Override existing partition
                # Subvolumes must set a mountpoint in order to be mounted,
                # unless their parent is mounted
                subvolumes = {
                  # Subvolume name is different from mountpoint
                  "/root" = {
                    inherit mountOptions;
                    mountpoint = "/";
                  };
                  # Subvolume name is the same as the mountpoint
                  "/nix" = {
                    inherit mountOptions;
                    mountpoint = "/nix";
                  };
                  # Parent is not mounted so the mountpoint must be set
                  "/persistent" = {
                    mountOptions = ["ssd" "compress=zstd:2" "noatime" "discard=async" "space_cache=v2"];
                    mountpoint = "/persistent";
                  };
                };
              };
            };

            zram-writeback = {
              size = "8G";
            };
          };
        };
      };
    };
  };
}
