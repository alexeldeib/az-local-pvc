# Design

- Enumerate /sys/block/* for devices that look like nvme (name contains "nvme")
- Get UUID. If populated, disk has been formatted.
- Using known UUID, check for /mnt/pv-disks/$UUID. If it doesn't exist, create it.
- Check if /dev/nvme* is mounted by invoking mount.static and reading line by line for /dev/nvme*
  - If it isn't, mount it at /mnt/pv-disks/$UUID
  - If it is, but not at /mnt/pv-disks/$UUID, error? or unmount, delete old mount point, and remount
  - If it is and it's at /mnt/pv-disks/$UUID, do nothing. We are done.

