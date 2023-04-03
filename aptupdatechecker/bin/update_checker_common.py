from os.path import dirname, join, split
from os import environ
import subprocess

icon_ok = join(split(dirname(__file__))[0], "icons/software-update-available.svg")
icon_error = join(split(dirname(__file__))[0], "icons/software-update-error.svg")

def set_dbus_addr():
  ### replace with https://askubuntu.com/questions/879066/what-is-the-function-of-sytemds-execstartpre-directive
  # https://superuser.com/questions/1555754/why-isnt-systemd-running-my-execstartpre-script
  environ['DBUS_SESSION_BUS_ADDRESS'] = "unix:path=/run/user/1000/bus"

def error_notification (title, errorstr, app, icon=icon_error):
  set_dbus_addr()
  try:
    subprocess.check_call(" ".join(['notify-send', '-t', '30000', '--app-name', '"{0}"'.format(app), '--icon', '"{0}"'.format(icon), '-u', 'critical', '"{0}"'.format(title), '"{0}"'.format(str(errorstr))]), shell=True)
  except subprocess.CalledProcessError as e:
    print("Notification failed")

def update_notifier(app, title, msg, icon=icon_ok):
  set_dbus_addr()
  try:
    subprocess.check_call(" ".join(['notify-send', '-t', '30000', '--app-name', '"{0}"'.format(app), '--icon', '"{0}"'.format(icon), '-u', 'normal', '"{0}"'.format(title), '"{0}"'.format(msg)]), shell=True)
  except subprocess.CalledProcessError as e:
    print("Notification failed")