#! /bin/bash

dpkg-buildpackage -b -uc -us
dpkg -b debian/aptupdatechecker
mv debian/aptupdatechecker.deb aptupdatechecker-0.1.1.deb