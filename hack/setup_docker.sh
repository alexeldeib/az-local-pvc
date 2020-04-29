#!/usr/bin/env bash
set -o errexit
set -o nounset
set -o pipefail

# apt repo
DISTRO="$(cat /etc/os-release | grep ^ID= | cut -d= -f2)"
CODENAME="$(cat /etc/os-release | grep VERSION_CODENAME= | cut -d= -f2)"

## gpg key
curl -fsSl https://download.docker.com/linux/${DISTRO}/gpg -o gpg.asc
sudo apt-key add gpg.asc

echo "deb [arch=amd64] https://download.docker.com/linux/${DISTRO} ${CODENAME} stable" | sudo tee /etc/apt/sources.list.d/docker.list

# install docker
sudo apt update && sudo apt install -y docker-ce

sudo usermod -aG docker "$(whoami)"

# login
cat ~/.docker_pass | sudo docker login -u "$(cat ~/.docker_user)" --password-stdin
