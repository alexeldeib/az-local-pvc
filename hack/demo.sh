#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

COLOR='\033[0;36m'
NOCOLOR='\033[0m' # No Color

PLAINTEXT="${PLAINTEXT:=}"

if [ ! -z "$PLAINTEXT" ]; then
    COLOR='\033[0m'
fi

echo ""
echo -e "${COLOR}To avoid colored output, set PLAINTEXT=y${NOCOLOR}"
echo -e "${COLOR}e.g. PLAINTEXT=y ./hack/demo.sh${NOCOLOR}"
echo ""
echo -e "${COLOR}This demo requires an AKS cluster with L-series VMs${NOCOLOR}"
echo -e "${COLOR}The NVMe drives should be unformatted and unmounted${NOCOLOR}"
echo -e "${COLOR}You can use this command to create such a cluster: ${NOCOLOR}"
echo -e "
az aks create
    -g <RESOURCE_GROUP>
    -n <CLUSTER_NAME>
    -l <LOCATION>
    --node-vm-size Standard_L16s_v2
    --node-osdisk-size 1023
    -k 1.17.3
    --network-plugin azure
    --enable-vmss
    --load-balancer-sku standard"
echo -e "${COLOR}${NOCOLOR}"
echo -e "${COLOR}Additionally, you must have kubectl installed.${NOCOLOR}"
echo -e "${COLOR}${NOCOLOR}"

sleep 3

echo -e "${COLOR}Press any key to continue${NOCOLOR}"
# read -n 1 -s/ -r

echo -e "${COLOR}${NOCOLOR}"
echo -e "${COLOR}Let's see what nodes we have${NOCOLOR}"
echo -e "${COLOR}${NOCOLOR}"

sleep 3

kubectl get nodes -o custom-columns="NAME:.metadata.name,SKU:.metadata.labels['node\.kubernetes\.io\/instance-type']"

sleep 3

echo -e "${COLOR}${NOCOLOR}"
echo -e "${COLOR}L-Series VMs have one 1.92 TB NVMe device per 8 CPU cores.${NOCOLOR}"
echo -e "${COLOR}For a Standard_L16s_v2 VM, we expect 2 devices.${NOCOLOR}"
echo -e "${COLOR}${NOCOLOR}"

sleep 3

echo -e "${COLOR}Let's deploy the formatter + provisioner${NOCOLOR}"
echo -e "${COLOR}We'll also deploy a storage class for the local SSDs${NOCOLOR}"
echo -e "${COLOR}${NOCOLOR}"

sleep 3

kubectl apply -f manifests/storage-class.yaml
kubectl apply -f manifests/rbac.yaml
kubectl apply -f manifests/local-storage-provisioner.yaml
kubectl apply -f manifests/local-storage-formatter.yaml

echo -e "${COLOR}${NOCOLOR}"
echo -e "${COLOR}Waiting for those to roll out...${NOCOLOR}"
echo -e "${COLOR}${NOCOLOR}"

kubectl rollout status daemonset/local-storage-provisioner
kubectl rollout status daemonset/local-storage-formatter

sleep 3

echo -e "${COLOR}${NOCOLOR}"
echo -e "${COLOR}We expect to start seeing persistent volumes when these are both deployed${NOCOLOR}"
echo -e "${COLOR}${NOCOLOR}"

sleep 3

kubectl get pv -o wide

sleep 3

echo -e "${COLOR}${NOCOLOR}"
echo -e "${COLOR}Now we can deploy a sample workload${NOCOLOR}"
echo -e "${COLOR}The workload includes a pod and persistent volume claim${NOCOLOR}"
echo -e "${COLOR}It should mount in O(seconds)${NOCOLOR}"
echo -e "${COLOR}${NOCOLOR}"

sleep 3

kubectl apply -f manifests/local-storage-consumer.yaml

echo -e "${COLOR}${NOCOLOR}"
echo -e "${COLOR}Waiting for those to roll out...${NOCOLOR}"
echo -e "${COLOR}${NOCOLOR}"

kubectl rollout status deploy/local-storage-consumer

sleep 3

echo -e "${COLOR}${NOCOLOR}"
echo -e "${COLOR}Now we should see a volume bound to our workload${NOCOLOR}"
echo -e "${COLOR}${NOCOLOR}"

kubectl get pv -o wide

echo -e "${COLOR}${NOCOLOR}"
echo -e "${COLOR}If we delete the pod + volume claim...${NOCOLOR}"
echo -e "${COLOR}${NOCOLOR}"

sleep 3

kubectl delete -f manifests/local-storage-consumer.yaml

sleep 5

echo -e "${COLOR}${NOCOLOR}"
echo -e "${COLOR}...the volume should be released and ready for re-use${NOCOLOR}"
echo -e "${COLOR}The provisioner will recycle it from released back to available after scrubbing.${NOCOLOR}"
echo -e "${COLOR}That can take a few seconds...(this part of the demo goes wrong some times)${NOCOLOR}"
echo -e "${COLOR}${NOCOLOR}"

sleep 15

kubectl get pv -o wide

sleep 3

echo ""
echo -e "${COLOR}Simply delete all the original manifests to cleanup.${NOCOLOR}"
echo -e "${COLOR}NOTE: this will not clean up the disks in any way currently.${NOCOLOR}"
echo ""

sleep 3

kubectl delete -f manifests/local-storage-formatter.yaml
kubectl delete -f manifests/local-storage-provisioner.yaml
kubectl delete -f manifests/rbac.yaml
kubectl delete -f manifests/storage-class.yaml
