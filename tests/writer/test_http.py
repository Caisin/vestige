#!/usr/bin/env python3
"""Real daemon + HTTP/MCP acceptance for writer workflows. Synthetic fixtures only.

Run: python3 tests/writer/test_http.py --binary /path/to/vestige-mcp
No generation provider, user database or network service is required.
"""
import argparse
import concurrent.futures
import io
import json
import os
from pathlib import Path
import secrets
import socket
import sqlite3
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
import zipfile


def available_port():
    with socket.socket() as channel:
        channel.bind(('127.0.0.1', 0))
        return channel.getsockname()[1]


class Mcp:
    def __init__(self, port, token):
        self.url = f'http://127.0.0.1:{port}/mcp'
        self.headers = {'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json', 'Accept': 'application/json, text/event-stream'}
        self.sequence = 0
        result = self.request('initialize', {'protocolVersion': '2025-03-26', 'capabilities': {}, 'clientInfo': {'name': 'writer-acceptance-agent', 'version': '1'}})
        self.headers['MCP-Protocol-Version'] = result['protocolVersion']
        self.request('notifications/initialized', {}, notification=True)

    def request(self, method, params, notification=False):
        self.sequence += 1
        body = {'jsonrpc': '2.0', 'method': method, 'params': params}
        if not notification:
            body['id'] = self.sequence
        request = urllib.request.Request(self.url, data=json.dumps(body).encode(), headers=self.headers)
        with urllib.request.urlopen(request, timeout=90) as response:
            if response.headers.get('Mcp-Session-Id'):
                self.headers['Mcp-Session-Id'] = response.headers['Mcp-Session-Id']
            raw = response.read()
        if not raw:
            return None
        result = json.loads(raw)
        assert 'error' not in result, result
        return result['result']

    def call(self, tool, action, expect_error=None, **arguments):
        result = self.request('tools/call', {'name': tool, 'arguments': {'action': action, **arguments}})
        content = result.get('structuredContent') or json.loads(result['content'][0]['text'])
        if expect_error:
            assert result.get('isError') and content.get('code') == expect_error, content
        else:
            assert not result.get('isError'), content
        return content


def docx(text):
    output = io.BytesIO()
    with zipfile.ZipFile(output, 'w', zipfile.ZIP_DEFLATED) as archive:
        archive.writestr('word/document.xml', f'<w:document xmlns:w="urn:word"><w:body><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:body></w:document>')
    return output.getvalue()


def pdf(text):
    stream = f'BT /F1 14 Tf 50 750 Td ({text}) Tj ET'.encode()
    objects = [b'<< /Type /Catalog /Pages 2 0 R >>', b'<< /Type /Pages /Kids [3 0 R] /Count 1 >>', b'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>', b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>', b'<< /Length ' + str(len(stream)).encode() + b' >>\nstream\n' + stream + b'\nendstream']
    result = b'%PDF-1.4\n'; offsets = [0]
    for index, obj in enumerate(objects, 1):
        offsets.append(len(result)); result += str(index).encode() + b' 0 obj\n' + obj + b'\nendobj\n'
    xref = len(result); result += b'xref\n0 6\n0000000000 65535 f \n'
    for offset in offsets[1:]: result += f'{offset:010d} 00000 n \n'.encode()
    return result + f'trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF'.encode()


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('--binary', required=True); parser.add_argument('--output'); options = parser.parse_args()
    binary = str(Path(options.binary).resolve()); dashboard = available_port(); port = available_port(); token = secrets.token_hex(32)
    with tempfile.TemporaryDirectory(prefix='vestige-writer-acceptance-') as temporary:
        root = Path(temporary); data = root / 'store'; data.mkdir(); log = (root / 'server.log').open('w')
        environment = os.environ.copy(); environment.update({'VESTIGE_DATA_DIR': str(data), 'VESTIGE_AUTH_TOKEN': token, 'VESTIGE_DASHBOARD_ENABLED': '1', 'VESTIGE_DASHBOARD_PORT': str(dashboard), 'VESTIGE_HTTP_PORT': str(port), 'VESTIGE_HTTP_BIND': '127.0.0.1'})
        process = None

        def start(directory):
            env = {**environment, 'VESTIGE_DATA_DIR': str(directory)}
            child = subprocess.Popen([binary, '--daemon', '--http'], stdin=subprocess.DEVNULL, stdout=log, stderr=log, env=env)
            for _ in range(120):
                if child.poll() is not None:
                    raise RuntimeError((root / 'server.log').read_text()[-5000:])
                try:
                    with urllib.request.urlopen(f'http://127.0.0.1:{dashboard}/api/health', timeout=2) as response:
                        assert json.load(response)['writerApiVersion'] == 1
                    return child
                except (OSError, KeyError): time.sleep(0.5)
            child.kill(); child.wait(); raise RuntimeError('Daemon did not become ready')

        def upload(role, name, content, origin=None, expected_status=200):
            query = urllib.parse.urlencode({'role_id': role, 'filename': name, 'title': name.rsplit('.', 1)[0]})
            headers = {'Content-Type': 'application/octet-stream', 'Authorization': 'Bearer ' + token}
            if origin: headers['Origin'] = origin
            request = urllib.request.Request(f'http://127.0.0.1:{dashboard}/api/writer/upload?{query}', data=content, headers=headers)
            try:
                with urllib.request.urlopen(request, timeout=30) as response:
                    assert response.status == expected_status; return json.load(response)
            except urllib.error.HTTPError as error:
                assert error.code == expected_status, error.read(); return json.load(error)

        try:
            process = start(data); agent = Mcp(port, token)
            names = {tool['name'] for tool in agent.request('tools/list', {})['tools']}
            required = {'writer_role', 'writer_source', 'writer_task', 'writer_context', 'writer_project', 'writer_draft', 'writer_review'}
            assert required <= names
            role = agent.call('writer_role', 'create', name='验收用悬疑编剧（合成素材）', description='仅测试外部 Agent 合作协议')['role']['id']
            screenplay = '第1集 第一场 内景 旧车站 夜\n沈禾：如果我把录音交出去，姐姐就回不了家。\n沈禾看着断裂的表带，最终把录音笔放在警察面前。\n第1集 第二场 外景 街口 夜\n灯亮了。她发现录音笔里还有一段自己的声音。'
            imported = upload(role, '测试剧本.txt', screenplay.encode()); source = imported['source']['id']
            assert upload(role, '副本.txt', screenplay.encode())['duplicate']
            formats = [('补充.md', b'# Scene\nA choice with a cost.'), ('场景.fountain', 'INT. 车站 - 夜\n她握住录音笔。'.encode()), ('文档.docx', docx('她把秘密说了出来。')), ('分场.fdx', '<FinalDraft><Content><Paragraph><Text>门外传来了脚步声。</Text></Paragraph></Content></FinalDraft>'.encode()), ('PDF剧本.pdf', pdf('A secret changes the meaning of an earlier clue.'))]
            for name, content in formats: assert upload(role, name, content)['source']['segment_count'] > 0
            upload(role, '不安全.fdx', b'<!DOCTYPE x SYSTEM "file:///fiction"><x/>', expected_status=400)
            upload(role, '错误.docx', b'not a docx', expected_status=400)
            upload(role, '跨站.txt', b'blocked', origin='https://untrusted.invalid', expected_status=403)
            segment = agent.call('writer_source', 'get', role_id=role, source_id=source, limit=1)['segments'][0]
            task = agent.call('writer_task', 'create', role_id=role, kind='extract')['task']['id']

            def claim(index):
                client = Mcp(port, token)
                result = client.request('tools/call', {'name': 'writer_task', 'arguments': {'action': 'claim', 'task_id': task, 'agent_id': f'test-{index}'}})
                return result
            with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor: contenders = list(executor.map(claim, range(2)))
            assert sum(not item.get('isError') for item in contenders) == 1
            winner = next(item for item in contenders if not item.get('isError')); lease = winner['structuredContent']['lease_token']
            rules = [{'id': 'costly-choice', 'category': 'conflict', 'title': '让选择付出代价', 'instruction': '让角色在亲情与公共责任之间做出具体且有代价的选择。', 'applies_to': '人物关系的转折场景', 'exceptions': '不强行用于没有价值冲突的过渡场景', 'evidence': [{'source_id': source, 'segment_id': segment['id'], 'quote': '如果我把录音交出去，姐姐就回不了家。'}]}]
            invalid = json.loads(json.dumps(rules)); invalid[0]['evidence'][0]['quote'] = '不存在的原话'
            agent.call('writer_task', 'complete', task_id=task, lease_token=lease, result={'summary': '伪证据', 'rules': invalid}, expect_error='INVALID_EVIDENCE')
            result = {'summary': '价值选择与集尾重释线索', 'rules': rules}
            agent.call('writer_task', 'heartbeat', task_id=task, lease_token=lease)
            agent.call('writer_task', 'complete', task_id=task, lease_token=lease, result=result)
            assert agent.call('writer_task', 'complete', task_id=task, lease_token=lease, result=result)['idempotent']
            assert agent.call('writer_role', 'get', role_id=role)['role']['active_version'] == 0
            agent.call('writer_role', 'publish', role_id=role, version=1, expected_version=0)
            project = agent.call('writer_project', 'create', role_id=role, name='原创短剧验收', format='short_drama', brief='失物招领员发现一封写给明天的信。', canon={'episode_count': 12, 'episode_minutes': 3, 'characters': '程雨：谨慎，但无法放下父亲失踪的疑问。'})['project']['id']
            chat = agent.call('writer_task', 'create', role_id=role, kind='role_chat', input={'message': '不要用旁白解释，让动作表达犹豫。'})['task']
            claimed = agent.call('writer_task', 'claim', task_id=chat['id'], agent_id='revision-agent')
            rules.append({'id': 'action-not-exposition', 'category': 'preference', 'title': '用动作呈现犹豫', 'instruction': '以行为和停顿呈现人物犹豫，减少解释性旁白。', 'evidence': [{'message_id': chat['input']['message_id'], 'quote': '不要用旁白解释，让动作表达犹豫。'}]})
            agent.call('writer_task', 'complete', task_id=chat['id'], lease_token=claimed['lease_token'], result={'summary': '采纳动作优先的偏好', 'reply': '我将保留有代价的选择，并以人物动作呈现犹豫。请查看候选版本再采纳。', 'rules': rules})
            agent.call('writer_role', 'publish', role_id=role, version=2, expected_version=1)
            context = agent.call('writer_context', 'prepare', role_id=role, project_id=project, query='第一场')
            assert context['role_version'] == 1 and len(context['rules']) == 1
            updated = agent.call('writer_project', 'update', project_id=project, expected_revision=1, role_version=2)['project']; assert updated['revision'] == 2
            other = agent.call('writer_role', 'create', name='其他角色')['role']['id']
            agent.call('writer_context', 'prepare', role_id=other, project_id=project, expect_error='OWNERSHIP_MISMATCH')
            job = agent.call('writer_task', 'create', role_id=role, project_id=project, kind='write', input={'kind': 'scene', 'instruction': '写原创开场'})['task']['id']
            lease = agent.call('writer_task', 'claim', task_id=job, agent_id='writing-agent')['lease_token']
            content = '内景 失物招领处 夜\n程雨把信推到桌沿，手却压着信封上的日期。\n同事：末班车要走了。\n程雨松开手。信封滑落，露出父亲的笔迹。'
            completed = agent.call('writer_task', 'complete', task_id=job, lease_token=lease, result={'kind': 'scene', 'title': '明天的信', 'content': content})
            draft = completed['task']['result']['output']['draft']['id']
            review = agent.call('writer_task', 'create', role_id=role, project_id=project, kind='review', input={'draft_id': draft})['task']['id']
            lease = agent.call('writer_task', 'claim', task_id=review, agent_id='review-agent')['lease_token']
            agent.call('writer_task', 'complete', task_id=review, lease_token=lease, result={'summary': '动作清晰，可进一步交代错过末班车的代价。', 'findings': [{'severity': 'warning', 'quote': '末班车要走了。', 'issue': '离开与留下的代价还不够明确。', 'suggestion': '补充程雨此刻必须赶上的约定。'}]})
            agent.call('writer_source', 'delete', role_id=role, source_id=source, confirm=True, expect_error='SOURCE_IN_USE')
            process.terminate(); process.wait(timeout=30); process = start(data); agent = Mcp(port, token)
            assert agent.call('writer_role', 'get', role_id=role)['role']['active_version'] == 2
            assert agent.call('writer_draft', 'get', draft_id=draft)['draft']['content'] == content
            restored = root / 'restored'; restored.mkdir()
            with sqlite3.connect(str(data / 'vestige.db')) as original, sqlite3.connect(str(restored / 'vestige.db')) as backup:
                original.backup(backup); assert backup.execute('PRAGMA integrity_check').fetchone()[0] == 'ok'
            process.terminate(); process.wait(timeout=30); process = start(restored); agent = Mcp(port, token)
            assert agent.call('writer_draft', 'get', draft_id=draft)['draft']['content'] == content
            assert agent.call('writer_source', 'get', role_id=role, source_id=source)['segments'][0]['text'] == segment['text']
            report = {'transport': 'real HTTP MCP', 'daemonWithClosedStdin': True, 'tools': sorted(required), 'supportedFormats': ['txt', 'md', 'fountain', 'fdx', 'docx', 'pdf'], 'duplicateUpload': True, 'untrustedOriginRejected': True, 'invalidEvidenceRejected': True, 'exclusiveClaim': True, 'idempotentCompletion': True, 'candidatePublishAndConversationRevision': True, 'projectVersionPinning': True, 'crossRoleContextRejected': True, 'originalDraftAndBoundReview': True, 'restartPersistence': True, 'fullDatabaseRestore': True, 'scope': 'Synthetic external-Agent protocol fixture; not a writing-quality benchmark.'}
            if options.output: Path(options.output).write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
            print(json.dumps(report, ensure_ascii=False, indent=2))
        finally:
            if process and process.poll() is None: process.terminate(); process.wait(timeout=30)
            log.close()


if __name__ == '__main__': main()
