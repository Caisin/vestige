PRAGMA foreign_keys=ON;
PRAGMA journal_mode=WAL;
CREATE TABLE IF NOT EXISTS users(id TEXT PRIMARY KEY, issuer TEXT NOT NULL, subject TEXT NOT NULL, name TEXT NOT NULL, UNIQUE(issuer,subject));
CREATE TABLE IF NOT EXISTS workspaces(id TEXT PRIMARY KEY, name TEXT NOT NULL, kind TEXT NOT NULL CHECK(kind IN ('personal','shared','legacy')), owner TEXT NOT NULL REFERENCES users(id));
CREATE TABLE IF NOT EXISTS members(workspace TEXT NOT NULL REFERENCES workspaces(id), user_id TEXT NOT NULL REFERENCES users(id), role TEXT NOT NULL CHECK(role IN ('owner','editor','viewer')), PRIMARY KEY(workspace,user_id));
CREATE TABLE IF NOT EXISTS credentials(id TEXT PRIMARY KEY, hash TEXT NOT NULL UNIQUE, user_id TEXT NOT NULL REFERENCES users(id), workspace TEXT NOT NULL REFERENCES workspaces(id), kind TEXT NOT NULL CHECK(kind IN ('session','agent')), permission TEXT NOT NULL, label TEXT NOT NULL, provider TEXT NOT NULL, upstream BLOB NOT NULL, expires INTEGER NOT NULL, checked INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS flows(state TEXT PRIMARY KEY, binding TEXT NOT NULL, provider TEXT NOT NULL, verifier TEXT NOT NULL, expires INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS invitations(hash TEXT PRIMARY KEY, workspace TEXT NOT NULL REFERENCES workspaces(id), role TEXT NOT NULL, expires INTEGER NOT NULL, creator TEXT NOT NULL REFERENCES users(id));
CREATE TABLE IF NOT EXISTS audit(id TEXT PRIMARY KEY, user_id TEXT NOT NULL, workspace TEXT NOT NULL, action TEXT NOT NULL, created INTEGER NOT NULL);
-- One active browser login authorizes the loopback MCP bridge. It is never
-- accepted from a remote listener and is removed on logout.
CREATE TABLE IF NOT EXISTS local_bindings(singleton INTEGER PRIMARY KEY CHECK(singleton = 1), credential TEXT NOT NULL REFERENCES credentials(id) ON DELETE CASCADE, updated INTEGER NOT NULL);
