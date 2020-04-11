# az-local-pvc

[![stability-experimental](https://img.shields.io/badge/stability-experimental-orange.svg)](#experimental)

## Experimental
Code is new and may change or be removed in future versions. Please try it out and provide feedback. If it addresses a use-case that is important to you please open an issue to discuss it further.

The goal of this project is to enable using local SSDs as PVCs in Kubernetes.

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

## Status

The current implementation scans /sys/block for devices containing nvme. It uses blkid to attempt to extract a UUID for a partition on the drive. Unfortunately this errors without useful output when it fails, making it unreliable. If no partition is detected (== failing on blkid), we attempt to format and mount.

We need to avoid shelling to blkid or make our usage more reliable. Ideally, we can read all of this information from udev or sysfs somehow.
