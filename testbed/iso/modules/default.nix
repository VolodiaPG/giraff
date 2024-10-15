{
  base = import ./base.nix;
  configuration = import ./configuration.nix;
  filesystem = import ./filesystem.nix;
  init = import ./init.nix;
  monitoring = import ./monitoring.nix;
  proxy = import ./proxy.nix;
  disk = import ./disk.nix;
  registry = import ./registry.nix;
  master = import ./master.nix;

  # tools
  make-disk-image-stateless = import ./make-disk-image-stateless.nix;
}
