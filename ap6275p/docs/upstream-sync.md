# Upstream synchronization

`upstream-master` tracks `https://github.com/orangepi-xunlong/firmware.git` `master` exactly. StableKite development happens on `main`.

On every push to StableKite `main`, GitHub Actions fetches upstream and attempts a normal merge. If upstream modifies either removed target (`ap6275p/fw_bcm43752a2_pcie_ag.bin` or `ap6275p/BCM4362A2.hcd`) and Git reports a modify/delete conflict, the workflow fails and prints the conflicted paths. It never chooses a side automatically.

Unrelated upstream firmware is merged normally.
