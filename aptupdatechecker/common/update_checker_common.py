from os.path import dirname
from plyer import notification

icon_ok = dirname(__file__) + "/icons/software-update-available.svg"
icon_error = dirname(__file__) + "/icons/software-update-error.svg"

def error_notification (title, errorstr, app, icon=icon_error):
  notification.notify(title=title, message=str(errorstr), timeout=30, app_name=app, app_icon=icon, hints={"bgcolor":"#f00000", "fgcolor":"fcfcae", "frcolor":"ff0000"})

def update_notifier(app, title, msg, icon=icon_ok):
    notification.notify(title=title, message=msg, timeout=30, app_name=app, app_icon=icon)
