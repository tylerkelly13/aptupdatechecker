from os.path import dirname, join, split
import subprocess

icon_ok = join(split(dirname(__file__))[0], "icons/software-update-available.svg")
icon_error = join(split(dirname(__file__))[0], "icons/software-update-error.svg")

def get_user():
  username = subprocess.check_output("who | grep -P '^\w+' | awk '{print $1}'", shell=True).decode('utf-8').split('\n')[0]
  uid = subprocess.check_output(" ".join(["id", "-u", '"{0}"'.format(username)]), shell=True).decode('utf-8').split("\n")[0]
  return [
    uid,
    username
  ]

def get_display():
  return subprocess.check_output("who | grep -m1 -P '^\w+' | awk '{print $5}' | sed 's/[(|)]//g'", shell=True).decode('utf-8').split("\n")[0]


def set_envs(user = get_user(), display = get_display()):
  uid, username = user
  return (
    " ".join(
    [
      "sudo -u {0}".format(username),
      "DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/{0}/bus".format(uid),
      "XDG_RUNTIME_DIR=/run/user/{0}".format(uid),
      "DISPLAY={0}".format(display),
      "UID={0}".format(uid),
    ]))

def error_notification (title, errorstr, app, icon=icon_error):
  try:
    env = set_envs()
    command = " ".join([env, 'notify-send', '-t', '30000', '--app-name', '"{0}"'.format(app), '--icon', '"{0}"'.format(icon), '-u', 'critical', '"{0}"'.format(title), '"{0}"'.format(str(errorstr))])
    subprocess.check_call(command, shell=True)
  except subprocess.CalledProcessError as e:
    print("Notification failed:", e.stderr)

def update_notifier(app, title, msg, icon=icon_ok):
  try:
    env = set_envs()
    command = " ".join([env, 'notify-send', '-t', '30000', '--app-name', '"{0}"'.format(app), '--icon', '"{0}"'.format(icon), '-u', 'normal', '"{0}"'.format(title), '"{0}"'.format(msg)])
    subprocess.check_call(command, shell=True)
  except subprocess.CalledProcessError as e:
    print("Notification failed:", e.stderr)