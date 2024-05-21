{
  pkgs,
  inputs,
  ...
}: {
  programs.fish.shellAliases = {
    kubectl = "k3s kubectl";
    k = "kubectl";
    k9 = "k9s --kubeconfig /etc/rancher/k3s/k3s.yaml -A";
  };

  virtualisation.docker.enable = true;

  services.k3s = {
    enable = true;
  };
  # useful packages
  environment.systemPackages = with pkgs; [
    faas-cli
    kubectl
    arkade
    tailscale
    k9s
    inputs.ebpf-netem.packages.${pkgs.system}.ebpf-netem
  ];

  services.tailscale.enable = true;

  systemd.services.rebootwatcher = {
    description = "reboot the computer when /nfs/enosvm/do_restart/xx.xx.xx is created";
    wantedBy = ["multi-user.target"];
    path = [pkgs.busybox];
    script = ''
      reboot_path() {
        local IP_GROUP_CMD=$(ip -f inet addr show ens3 | sed -En -e 's/.*inet ([0-9]+.[0-9]+.[0-9]+).*/\1/p')
        local DO_REBOOT_PATH="/nfs/enosvm/do_reboot/$IP_GROUP_CMD"
        echo "$DO_REBOOT_PATH"
      }

      PREV_BOOTID_PATH=/persistent/previous_bootid
      BOOTID=$(cat /proc/sys/kernel/random/boot_id)
      PREV_BOOTID=$BOOTID
      if [ -f $PREV_BOOTID_PATH ]; then
        PREV_BOOTID=$(cat $PREV_BOOTID_PATH)
      fi
      echo $PREV_BOOTID > /prevbootid
      echo $BOOTID > $(echo $PREV_BOOTID_PATH)
      while [[ ! -d "$(reboot_path)" || -f "$(reboot_path)/$PREV_BOOTID" ]]; do
        sleep 2
      done

      touch "$(reboot_path)/$BOOTID"
      reboot -ff
    '';
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = "yes";
      Restart = "on-failure";
      RestartSec = "3";
    };
  };

  system.stateVersion = "22.05"; # Do not change
}
