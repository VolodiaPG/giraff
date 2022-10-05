
# Tips
To connect to ssh in the vm, before starting it, do:
```sh
export QEMU_NET_OPTS="hostfwd=tcp::2221-:22,hostfwd=tcp::8080-:80"
```
and then:
```sh
ssh -oUserKnownHostsFile=/dev/null -oStrictHostKeyChecking=no nixos@localhost -p 2221
```

# To build the iso

Builds the iso and rsync it to grid5000
```sh
nix build .#nixosConfigurations.isoimage.config.system.build.isoImage
sync --perms --chmod=u+rwx,g+rwx,o+rwx result/iso/nixos.iso rennes.g5k:~/faas-fog.iso
```