{
  configuration = import ./configuration.nix;
  filesystem = import ./filesystem.nix;
  init = import ./init.nix;
  monitoring = import ./monitoring.nix;

  # tools
  make-disk-image-stateless = import ./make-disk-image-stateless.nix;
}
