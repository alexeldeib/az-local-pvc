[![stability-experimental](https://img.shields.io/badge/stability-experimental-orange.svg)](#experimental)
[![builds.sr.ht status](https://builds.sr.ht/~alexeldeib/az-local-pvc/.build.yml.svg)](https://builds.sr.ht/~alexeldeib/az-local-pvc/.build.yml?)
<!-- [![github actions status](https://github.com/alexeldeib/az-local-pvc/workflows/.github/workflows/main.yaml/badge.svg?branch=master)](https://github.com/alexeldeib/az-local-pvc/actions?query=workflow%3A.github%2Fworkflows%2Fmain.yaml) -->


# az-local-pvc

The goal of this project is to enable using NVMe SSDs e.g. on Azure LSv2 VMs with Kubernetes workloads.

## Experimental
Code is new and may change or be removed in future versions. Please try it out and provide feedback. If it addresses a use-case that is important to you please open an issue to discuss it further.


The project uses two pods to achieve this:
- bootstrapper to format and mount disks initially
- sig-storage-static-local-provisioner to scan local disks and create PVs for them.

src/main.rs scans for unmounted, unformatted nvme devices. It formats and mounts them, handing off control to the provisioner for the lifecycle of drive usage in Kubernetes.

## Usage

Under manifests/ there are several raw Kubernetes yaml files as well as a Kustomize manifest. 

Required:
- local-storage-formatted.yaml
- local-storage-provisioner.yaml
- rbac.yaml
- storage-class.yaml

The kustomize manifest directly applies only the required manifests.

`kustomize build manifests/ | kubectl apply -f -`

or 

```bash
kubectl apply -f manifests/local-storage-formatted.yaml
kubectl apply -f manifests/local-storage-provisioner.yaml
kubectl apply -f manifests/rbac.yaml
kubectl apply -f manifests/storage-class.yaml
```

local-storage-consumer.yaml contains a PVC using the newly created storage class and a pod with a claim for that PVC. Apply this to test the pod should schedule and run successfully. Deleting that manifest deletes both the pod and the PVC, so the pv status via `kubctl get pv -w` should cycle from bound, to released, to terminated, to available withing ~1-2 minutes.

```bash
kubectl apply -f manifests/local-storage-consumer.yaml
kubectl get pod -w # wait for running
kubectl delete -f manifests/local-storage-consumer.yaml
kubectl get pv -w # wait for pv to cycle back to available. unreliable currently with blkid, see below.
```

## Mechanics

- Enumerate /sys/block/* for devices that look like nvme (name contains "nvme")
- Get UUID. If populated, disk has been formatted.
- Using known UUID, check for /mnt/pv-disks/$UUID. If it doesn't exist, create it.
- Check if /dev/nvme* is mounted by invoking mount.static and reading line by line for /dev/nvme*
  - If it isn't, mount it at /mnt/pv-disks/$UUID
  - If it is, but not at /mnt/pv-disks/$UUID, error? or unmount, delete old mount point, and remount
  - If it is and it's at /mnt/pv-disks/$UUID, do nothing. We are done.
