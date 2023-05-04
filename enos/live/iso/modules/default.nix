{
  configuration = import ./configuration.nix;
  filesystem = import ./filesystem.nix;
  init = import ./init.nix;

  # tools
  make-disk-image-stateless = import ./make-disk-image-stateless.nix;
}
