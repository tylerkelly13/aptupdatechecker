#!/usr/bin/env python3
from subprocess import SubprocessError, check_output, run
import aptupdatechecker.lib.update_checker_common as common
import re

updated_regex = re.compile('(\d+)\s+to\supgrade,')


def apt_update():
  try:
    run(["apt-get", "update"], capture_output=True)
  except SubprocessError:
    common.error_notification("Failed to retrieve package update lists", SubprocessError)
    raise

def apt_installable():
  try:
    upgradeable = updated_regex.search(check_output(["apt-get", "dist-upgrade", "--simulate", "--purge", "--auto-remove"]).decode('utf-8')).group(1)
  except SubprocessError:
    common.error_notification("Failed to determine number of upgrades", SubprocessError)
    raise
  return upgradeable

def main():
  apt_update()
  sw_upgrades = apt_installable()

  software_str = "Software upgrade available" if int(sw_upgrades) == 1 else (sw_upgrades) + " software upgrades available\nRun `aptupdater` to install"

  common.update_notifier("Software update checker", common.icon, "Software updates available!", software_str)

if __name__ == "__main__":
    main()