#!/usr/bin/env python

from __future__ import print_function

import os
import toml

_pwd = os.path.dirname(os.path.abspath(__file__))
cargo = toml.loads(open(os.path.join(_pwd, 'Cargo.toml'), 'r').read())

for section in ['dependencies', 'dev-dependencies']:
    for dep, version in cargo[section].items():
        print('cargo build -p %s' % dep)
