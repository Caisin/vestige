export interface WriterRole { id: string; name: string; description: string; active_version: number; source_count: number; created_at: string; updated_at: string }
export interface Evidence { source_id?: string; segment_id?: string; message_id?: string; quote: string }
export interface WriterRule { id: string; category: string; title: string; instruction: string; rationale?: string; applies_to?: string; exceptions?: string; evidence: Evidence[] }
export interface WriterVersion { role_id: string; version: number; base_version: number; status: 'draft' | 'published'; rules: WriterRule[]; summary: string; task_id?: string; created_at: string }
export interface WriterSource { id: string; role_id: string; title: string; filename: string; format: string; sha256: string; metadata?: { author?: string; episode?: string; parser_version?: string }; bytes: number; characters: number; segment_count: number; created_at: string }
export interface WriterSegment { id: string; ordinal: number; text: string; start_line: number; end_line: number }
export interface WriterMessage { id: string; speaker: string; content: string; task_id?: string; created_at: string }
export interface WriterTask { id: string; role_id: string; project_id?: string; kind: string; base_version: number; status: string; agent_id?: string; lease_until?: number; error?: string; input: Record<string, unknown>; result?: { output: { candidate_version?: number }; agent_result: Record<string, unknown> }; created_at: string }
export interface WriterProject { id: string; role_id: string; role_version: number; name: string; format: string; brief: string; canon: Record<string, unknown>; revision: number; created_at: string }
export interface WriterDraft { id: string; project_id: string; revision: number; role_version: number; kind: string; title: string; content: string; created_at: string }
export interface WriterReview { id: string; draft_id: string; draft_revision: number; summary: string; findings: { severity: string; quote: string; issue: string; suggestion: string }[]; created_at: string }

async function response<T>(res: Response): Promise<T> {
    if (!res.ok) {
        const error = await res.json().catch(() => ({}));
        throw new Error(error.error ?? error.message ?? `请求失败（${res.status}）`);
    }
    return res.json();
}
export async function writer<T>(tool: string, arguments_: Record<string, unknown>): Promise<T> {
    return response<T>(await fetch(`/api/writer/${tool}`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(arguments_) }));
}
export async function uploadScript(roleId: string, file: File, metadata: { author?: string; episode?: string; title?: string } = {}): Promise<{ source: WriterSource; duplicate: boolean }> {
    if (file.size > 50 * 1024 * 1024) throw new Error('单个文件不能超过 50 MiB，请分卷上传。');
    const query = new URLSearchParams({ role_id: roleId, filename: file.name, title: metadata.title?.trim() || file.name.replace(/\.[^.]+$/, ''), author: metadata.author ?? '', episode: metadata.episode ?? '' });
    return response(await fetch(`/api/writer/upload?${query}`, { method: 'POST', headers: { 'Content-Type': 'application/octet-stream' }, body: file }));
}
export async function downloadScript(roleId: string, source: WriterSource): Promise<void> {
    const res = await fetch('/api/writer/download', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ role_id: roleId, source_id: source.id }) });
    if (!res.ok) { await response(res); return; }
    downloadBlob(await res.blob(), source.filename);
}
export function downloadBlob(blob: Blob, filename: string): void {
    const url = URL.createObjectURL(blob); const link = document.createElement('a'); link.href = url; link.download = filename; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
}
export const taskNames: Record<string, string> = { extract: '作品提炼', role_chat: '角色调校', write: '剧本创作', review: '剧本审稿' };
export const taskStates: Record<string, string> = { queued: '等待 Agent', leased: 'Agent 处理中', completed: '已完成', failed: '处理失败', cancelled: '已取消' };
export const formatNames: Record<string, string> = { short_drama: '短剧', web_series: '网剧', film: '电影', tv_series: '长篇电视剧' };
export const draftNames: Record<string, string> = { outline: '故事大纲', episode: '分集剧本', scene: '场景剧本' };
export const categoryNames: Record<string, string> = { premise: '故事前提', character: '人物动机', conflict: '冲突', pacing: '节奏', foreshadowing: '伏笔', dialogue: '台词', reversal: '反转', cliffhanger: '集尾悬念', ending: '结局', format: '格式', preference: '用户偏好' };
export const dateLabel = (value: string) => new Date(value).toLocaleString('zh-CN', { month: 'numeric', day: 'numeric', hour: '2-digit', minute: '2-digit', hour12: false });
