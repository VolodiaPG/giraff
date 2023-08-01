{pkgs, ...}: let
  binPath = with pkgs;
    lib.strings.makeBinPath (
      [
        bash
        kubernetes-helm
        k3s
      ]
      ++ stdenv.initialPath
    );
in {
  systemd.services.start-openfaas = {
    description = "Starts openfaas on k3s";
    after = ["k3s.service"];
    wantedBy = ["multi-user.target"];
    script = ''
      #!${pkgs.bash}/bin/bash
      export PATH=${binPath}:$PATH
      export KUBECONFIG=/etc/rancher/k3s/k3s.yaml

      k3s kubectl apply -f https://raw.githubusercontent.com/openfaas/faas-netes/master/namespaces.yml
      helm repo add openfaas https://openfaas.github.io/faas-netes/
      helm repo update
      helm upgrade openfaas --install openfaas/openfaas --version 14.0.6 \
        --namespace openfaas \
        --set functionNamespace=openfaas-fn \
        --set generateBasicAuth=true
        # --set prometheus.image=ghcr.io/volodiapg/prometheus:v2.42.0 \
        # --set alertmanager.image=ghcr.io/volodiapg/alertmanager:v0.25.0 \
        # --set stan.image=ghcr.io/volodiapg/nats-streaming:0.25.3 \
        # --set nats.image=ghcr.io/volodiapg/nats:2.9.14
    '';
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = "yes";
    };
  };
}
