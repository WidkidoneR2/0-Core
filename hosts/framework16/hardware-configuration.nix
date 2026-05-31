# PLACEHOLDER for pre-install validation ONLY.
# At install this is REPLACED by the real hardware scan:
#   nixos-generate-config --no-filesystems --root /mnt
{ config, lib, pkgs, modulesPath, ... }:
{
  imports = [ (modulesPath + "/installer/scan/not-detected.nix") ];

  # Minimal initrd modules so the encrypted NVMe is reachable to unlock at boot.
  # The real scan fills in the accurate set for your Framework.
  boot.initrd.availableKernelModules = [ "nvme" "xhci_pci" "usb_storage" "sd_mod" ];
  boot.initrd.kernelModules = [ ];
  boot.kernelModules = [ ];
  boot.extraModulePackages = [ ];
}
