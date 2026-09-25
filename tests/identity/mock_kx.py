#!/usr/bin/env python3
"""Loopback-only test identity service. Never configure this provider in production."""
import argparse
import json
import secrets
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlencode, urlsplit


def encode(value):
    data = json.dumps(value, ensure_ascii=False).encode()
    salt = secrets.token_bytes(32)
    return bytes([len(salt)]) + salt + bytes(255 - ((b + salt[i % len(salt)]) % 256) for i, b in enumerate(data))


def decode(data):
    n = data[0]
    return json.loads(bytes((255 - b - data[1 + i % n]) % 256 for i, b in enumerate(data[n + 1:])))


class Handler(BaseHTTPRequestHandler):
    codes = set()

    def log_message(self, *_):
        pass

    def result(self, value, status=200):
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(encode({'code': status, 'result': value, 'msg': 'test fixture'}))

    def do_GET(self):
        url = urlsplit(self.path)
        if url.path == '/auth/dt/apps':
            return self.result([{'app_key': 'test-company', 'app_name': '测试共创组织', 'is_default': True}])
        if url.path == '/auth/user/user_info':
            user = self.headers.get('Authorization', '').removeprefix('Bearer test-only:')
            if user not in ['alice', 'bob', 'carol']:
                return self.result(None, 401)
            return self.result({'id': user, 'name': {'alice': '林编剧', 'bob': '陈编剧', 'carol': '审阅者'}[user], 'enabled': True})
        if url.path.startswith('/auth/dt/login'):
            callback = parse_qs(url.query)['redirect_url'][0]
            code = secrets.token_urlsafe(24)
            self.codes.add(code)
            self.send_response(303)
            self.send_header('Location', callback + '&' + urlencode({'exchange_code': code}))
            self.end_headers()
            return
        self.result(None, 404)

    def do_POST(self):
        data = decode(self.rfile.read(int(self.headers.get('Content-Length', 0))))
        if self.path == '/auth/user/access_token':
            user = data.get('user_name')
            if user not in ['alice', 'bob', 'carol'] or data.get('password') != 'test-only-password':
                return self.result(None, 401)
        elif self.path == '/auth/dt/exchange' and data.get('exchange_code') in self.codes:
            self.codes.remove(data['exchange_code'])
            user = 'alice'
        else:
            return self.result(None, 401)
        self.result({'access_token': 'test-only:' + user, 'exp_at': int(time.time()) + 3600})


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--port', type=int, default=3940)
    parser.add_argument('--allow-test-login', action='store_true', required=True)
    args = parser.parse_args()
    ThreadingHTTPServer(('127.0.0.1', args.port), Handler).serve_forever()
