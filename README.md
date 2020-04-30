[![stability-experimental](https://img.shields.io/badge/stability-experimental-orange.svg)](#experimental)
[![builds.sr.ht status](https://builds.sr.ht/~alexeldeib/az-local-pvc/.build.yml.svg)](https://builds.sr.ht/~alexeldeib/az-local-pvc/.build.yml?)
<!-- [![github actions status](https://github.com/alexeldeib/az-local-pvc/workflows/.github/workflows/main.yaml/badge.svg?branch=master)](https://github.com/alexeldeib/az-local-pvc/actions?query=workflow%3A.github%2Fworkflows%2Fmain.yaml) -->


# az-local-pvc

The goal of this project is to enable using NVMe SSDs e.g. on Azure LSv2 VMs with Kubernetes workloads

The project uses two pods to achieve this:
- bootstrapper to format and mount disks initially
- sig-storage-static-local-provisioner to scan local disks and create PVs for them.

## Experimental
Code is new and may change or be removed in future versions. Please try it out and provide feedback. If it addresses a use-case that is important to you please open an issue to discuss it further.

## Demo 

[![asciicast](https://asciinema.org/a/325049.svg)](https://asciinema.org/a/325049)

## Usage

Under manifests/ there are several raw Kubernetes yaml files as well as a Kustomize manifest. 

Required:
- local-storage-formatter.yaml
- local-storage-provisioner.yaml
- rbac.yaml
- storage-class.yaml

The kustomize manifest directly applies only the required manifests.

`kustomize build manifests/ | kubectl apply -f -`

or 

```bash
kubectl apply -f manifests/local-storage-formatter.yaml
kubectl apply -f manifests/local-storage-provisioner.yaml
kubectl apply -f manifests/rbac.yaml
kubectl apply -f manifests/storage-class.yaml

# Wait for rollout
kubectl rollout status daemonset/local-storage-provisioner
kubectl rollout status daemonset/local-storage-formatter
```

local-storage-consumer.yaml contains a PVC using the newly created storage class and a pod with a claim for that PVC. Apply this and the pod should schedule and run successfully. Deleting that manifest deletes both the pod and the PVC, so the pv status via `kubctl get pv -w` should cycle from bound, to released, to terminated, to available within ~1-2 minutes.

```bash
kubectl apply -f manifests/local-storage-consumer.yaml

# wait for running
kubectl rollout status deploy/local-storage-consumer

# delete it
kubectl delete -f manifests/local-storage-consumer.yaml

# wait for pv to cycle back to available.
kubectl get pv -w

# clean up formatter and provisioner, if desired
kubectl delete -f manifests/local-storage-formatter.yaml
kubectl delete -f manifests/local-storage-provisioner.yaml
kubectl delete -f manifests/rbac.yaml
kubectl delete -f manifests/storage-class.yaml
```

## Mechanics

Disclaimer: these are under rapid change and may not always be accurate.

- Enumerate /sys/block/* for devices that look like nvme (name contains "nvme")
- Get UUID. If populated, disk has been formatted.
- Using known UUID, check for /pv-disks/$UUID. If it doesn't exist, create it.
- Check if /dev/nvme* is mounted by invoking mount.static and reading line by line for /dev/nvme*
  - If it isn't, mount it at /pv-disks/$UUID
  - If it is, but not at /pv-disks/$UUID, error? or unmount, delete old mount point, and remount
  - If it is and it's at /pv-disks/$UUID, do nothing. We are done.
