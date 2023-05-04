{
  pkgs,
  inputs,
  outputs,
}: let
  inherit (inputs) nixpkgs;
  vm-persistence = nixpkgs.lib.nixosSystem {
    inherit (pkgs) system;
    modules = [
      "${nixpkgs}/nixos/modules/profiles/qemu-guest.nix"
      "${nixpkgs}/nixos/modules/profiles/all-hardware.nix"
      outputs.nixosModules.configuration
      outputs.nixosModules.filesystem
      outputs.nixosModules.init
    ];
    specialArgs = {inherit inputs outputs;};
  };
in {
  vm = let
    binPath = with pkgs;
      lib.strings.makeBinPath (
        [
          util-linux
          nix
          bash
        ]
        ++ (with vm-persistence.config.system.build; [
          nixos-install
          nixos-enter
        ])
        ++ stdenv.initialPath
      );
    directoriesList = vm-persistence.config.environment.persistence."/nix/persist".directories;
    directories = builtins.map (set: "\"" + set.directory + "\"") directoriesList;
  in
    outputs.nixosModules.make-disk-image-stateless {
      inherit pkgs;
      inherit (pkgs) lib;
      inherit (vm-persistence) config;
      diskSize = "auto";
      memSize = 4096; # During build-phase, here, locally
      additionalSpace = "4096M"; # Space added after all the necessary
      format = "qcow2-compressed";
      installBootLoader = false;
      VMMounts = ''
        #!${pkgs.bash}/bin/bash
        set -ex
        export PATH=${binPath}:$PATH

        mount -t tmpfs none $mountPoint

        mkdir -p $mountPoint{/boot,/nix,${builtins.concatStringsSep "," directories}}

        mount /dev/vda1 $mountPoint/nix

        find $mountPoint/nix -mindepth 1 -maxdepth 1 -type d ! -name "nix" -exec rm -rf {} \;
        mv $mountPoint/nix/nix/* $mountPoint/nix

        mkdir -p $mountPoint/nix/boot
        mkdir -p $mountPoint/nix/persist{${builtins.concatStringsSep "," directories}}

        mount -o bind $mountPoint/nix/boot $mountPoint/boot
        ${builtins.concatStringsSep "; " (builtins.map (dir: "mount -o bind $mountPoint/nix/persist" + dir + " $mountPoint" + dir) directories)}
      '';
      inVMScript = ''
        #!${pkgs.bash}/bin/bash
        set -ex
        export PATH=${binPath}:$PATH

        nixos-install --root $mountPoint --no-bootloader --no-root-passwd \
          --system ${vm-persistence.config.system.build.toplevel} \
          --substituters ""

        NIXOS_INSTALL_BOOTLOADER=1 nixos-enter --root $mountPoint -- /nix/var/nix/profiles/system/bin/switch-to-configuration boot
      '';
    };
}
