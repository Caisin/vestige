#!/usr/bin/env python3
"""Back up identity/workspace databases without resurrecting old login sessions on restore."""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import sqlite3
import tempfile


def snapshot(source, destination):
    destination.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    with sqlite3.connect(source.as_uri() + '?mode=ro', uri=True) as src:
        with sqlite3.connect(destination) as dst:
            src.backup(dst, pages=256)
            if dst.execute('PRAGMA integrity_check').fetchone()[0] != 'ok':
                raise RuntimeError('Snapshot integrity check failed')


def backup_identity(data, destination):
    data, destination = Path(data).resolve(), Path(destination).resolve()
    if destination.exists():
        raise ValueError('Destination already exists')
    destination.parent.mkdir(parents=True, exist_ok=True)
    temp = Path(tempfile.mkdtemp(prefix='.identity-backup-', dir=destination.parent))
    try:
        identity = data / 'identity'
        snapshot(identity / 'identity.db', temp / 'identity' / 'identity.db')
        with sqlite3.connect(temp / 'identity' / 'identity.db') as db:
            spaces = [row[0] for row in db.execute("SELECT id FROM workspaces WHERE kind!='legacy'")]
            # Backups restore users and permissions, never bearer credentials or pending grants.
            for table in ['credentials', 'flows', 'invitations']:
                db.execute('DELETE FROM ' + table)
        db.execute('PRAGMA wal_checkpoint(TRUNCATE)')
        db.close()
        files = 0
        for space in spaces:
            if not re.fullmatch(r'[a-f0-9]{64}', space):
                raise ValueError('Invalid workspace ID in identity catalog')
            source = identity / 'spaces' / space / 'vestige.db'
            if source.exists():
                snapshot(source, temp / 'identity' / 'spaces' / space / 'vestige.db')
                files += 1
        shutil.copy2(identity / 'session.key', temp / 'identity' / 'session.key')
        if (data / 'auth.json').is_file():
            shutil.copy2(data / 'auth.json', temp / 'auth.json')
        (temp / 'manifest.json').write_text(json.dumps({'format': 1, 'workspaceDatabases': files, 'credentialsRemoved': True, 'consistency': 'per-database SQLite snapshots; pause writes for a common point in time'}, indent=2))
        temp.rename(destination)
        return files
    except BaseException:
        shutil.rmtree(temp, ignore_errors=True)
        raise


if __name__ == '__main__':
    os.umask(0o077)
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('data_dir', type=Path)
    p.add_argument('destination', type=Path)
    args = p.parse_args()
    print(json.dumps({'workspaceDatabases': backup_identity(args.data_dir, args.destination), 'credentialsRemoved': True}))
