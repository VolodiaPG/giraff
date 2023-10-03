{
  base = import ./base.nix;
  configuration = import ./configuration.nix;
  filesystem = import ./filesystem.nix;
  init = import ./init.nix;
  monitoring = import ./monitoring.nix;
  proxy = import ./proxy.nix;

  # tools
  make-disk-image-stateless = import ./make-disk-image-stateless.nix;
}
