# Installs Longhorn
rm -rf /tmp/longhorn

git clone https://github.com/longhorn/longhorn.git --depth 1 /tmp/longhorn
kubectl create namespace longhorn-system
helm install longhorn /tmp/longhorn/chart/ --namespace longhorn-system --kubeconfig $HOME/.kube/config --set defaultSettings.defaultDataPath="/storage"
kubectl patch storageclass local-path -p '{"metadata": {"annotations":{"storageclass.kubernetes.io/is-default-class":"false"}}}'

rm -rf /tmp/longhorn
