from os.path import dirname, join, split
from grp import getgrnam
from pwd import getpwnam
import subprocess

icon_ok = join(split(dirname(__file__))[0], "icons/software-update-available.svg")
icon_error = join(split(dirname(__file__))[0], "icons/software-update-error.svg")

def get_users():
  sudo_users = getgrnam('sudo').gr_mem
  sudoers = []
  for user in sudo_users:
    uid = getpwnam(user).pw_uid
    username = user
    sudoers.append({uid, username})
  return sudoers

def get_display():
  try:
    return subprocess.check_output("who | grep -m1 -P '^\w+' | awk '{print $5}' | sed 's/[(|)]//g'", shell=True).decode('utf-8').split("\n")[0]
  except:
    return ":0"

def set_envs(users = get_users(), display = get_display()):
  sudo_user_set_envs = []
  for user in users:
    uid, username = user
    sudo_user_set_envs.append(
      " ".join(
      [
        "sudo -u {0}".format(username),
        "DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/{0}/bus".format(uid),
        "XDG_RUNTIME_DIR=/run/user/{0}".format(uid),
        "DISPLAY={0}".format(display),
        "WAYLAND_DISPLAY={0}".format("wayland-1"),
        "UID={0}".format(uid),
      ]))
  return sudo_user_set_envs

def error_notification (title, errorstr, app, icon=icon_error):
  for user in set_envs():
    try:
      command = " ".join([user, 'notify-send', '-t', '30000', '--app-name', '"{0}"'.format(app), '--icon', '"{0}"'.format(icon), '-u', 'critical', '"{0}"'.format(title), '"{0}"'.format(str(errorstr))])
      subprocess.check_call(command, shell=True)
    except subprocess.CalledProcessError as e:
      print("Notification failed:", e.stderr)

def update_notifier(app, title, msg, icon=icon_ok):
  for user in set_envs():
    try:
      command = " ".join([user, 'notify-send', '-t', '30000', '--app-name', '"{0}"'.format(app), '--icon', '"{0}"'.format(icon), '-u', 'normal', '"{0}"'.format(title), '"{0}"'.format(msg)])
      subprocess.check_call(command, shell=True)
    except subprocess.CalledProcessError as e:
      print("Notification failed:", e.stderr)