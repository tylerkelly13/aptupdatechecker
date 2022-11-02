from os.path import dirname
from subprocess import run

icon = dirname(__file__) + "/icons/software-update-available-svgrepo-com.svg"

def error_notification (msg, errorstr):
  run(["notify-send", "--urgency=critical", "--expire-time=30000", "-a", "APT update checker", "-i", icon, msg, str(errorstr)])

def update_notifier(app, icon, title, msg):
    run(["notify-send", "--urgency=normal", "--expire-time=30000", "-a", app, "-i", icon, title, msg])