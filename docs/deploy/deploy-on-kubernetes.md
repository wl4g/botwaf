# Deploy on Kubernetes

## Install with repo Helm

```bash
helm --kubeconfig ~/.kube/config -n botwaf upgrade --create-namespace -i \
-f ./tooling/deploy/helm/values.yaml \
botwaf \
--set image.repository=registry.cn-shenzhen.aliyuncs.com/wl4g \
--set image.tag=latest \
./tooling/deploy/helm
```
