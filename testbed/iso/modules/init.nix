{
  pkgs,
  outputs,
  ...
}: {
  systemd.services.start-openfaas = {
    description = "Starts openfaas on k3s";
    after = ["k3s.service"];
    wants = ["k3s.service"];
    wantedBy = ["multi-user.target"];
    script = ''
      ${pkgs.k3s}/bin/k3s kubectl apply -f /etc/namespaces.yaml
      ${pkgs.k3s}/bin/k3s kubectl apply -f /etc/kubenix.json
    '';
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = "yes";
      Restart = "on-failure";
      RestartSec = "3";
    };
  };

  environment.etc."kubenix.json".source = outputs.packages.${pkgs.system}.openfaas;
  environment.etc."namespaces.yaml".text = ''
    apiVersion: v1
    kind: Namespace
    metadata:
      name: openfaas
      annotations:
        linkerd.io/inject: enabled
        config.linkerd.io/skip-inbound-ports: "4222"
        config.linkerd.io/skip-outbound-ports: "4222"
      labels:
        role: openfaas-system
        access: openfaas-system
        istio-injection: enabled
    ---
    apiVersion: v1
    kind: Namespace
    metadata:
      name: openfaas-fn
      annotations:
        linkerd.io/inject: enabled
        config.linkerd.io/skip-inbound-ports: "4222"
        config.linkerd.io/skip-outbound-ports: "4222"
      labels:
        istio-injection: enabled
        role: openfaas-fn
  '';

  system.stateVersion = "22.05"; # Do not change
}
