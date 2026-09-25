# 登录、用户隔离与共同培养编剧

## 配置统一登录

在服务数据目录中创建私有 `auth.json`，或通过 `VESTIGE_AUTH_CONFIG` 指定配置文件。不要提交真实配置或凭据。

```json
{
  "public_origin": "http://127.0.0.1:3927",
  "kx_api": "https://identity.example.com/api",
  "kx_app_id": "admin"
}
```

使用 `vestige-mcp --daemon --http` 启动。浏览器进入 `/dashboard/login`。KX 账号密码和钉钉组织列表直接复用后台，钉钉跳转经 KX 回调后以一次性票据返回 Vestige。无需复制 KX 用户表、钉钉密钥或后台业务。身份接口使用 KX 的 KxEd 传输格式；保密性由 HTTPS 提供，KxEd 不是加密安全边界。

地址必须与浏览器使用的 origin 一致。公网使用 HTTPS 反向代理转发到本机监听器；不通过任意 Host/X-Forwarded-Host 生成登录回调。配置错误时服务拒绝启动，不降级成匿名模式。

未配置 `auth.json` 时仍为兼容的本地单用户模式。登录模式禁止无用户身份的 stdio MCP 与旧全局 Bearer 凭据。操作系统本地 CLI 仍属于拥有文件访问权限的管理员工具，不是租户入口。

KX 的 LoginType 枚举不代表所有登录方式已经实现。本次直接集成已存在的账号密码和钉钉流程；需要其他身份服务时可配置标准 OAuth2 授权码接入：

```json
{
  "public_origin": "https://writer.example.com",
  "kx_api": "https://identity.example.com/api",
  "kx_app_id": "admin",
  "oauth": {
    "name": "企业统一登录",
    "issuer": "https://sso.example.com",
    "authorize_url": "https://sso.example.com/oauth/authorize",
    "token_url": "https://sso.example.com/oauth/token",
    "userinfo_url": "https://sso.example.com/oauth/userinfo",
    "client_id": "vestige-example",
    "client_secret": "",
    "scope": "profile"
  }
}
```

在授权服务登记精确回调地址 `https://writer.example.com/api/auth/callback`。服务须支持 S256 PKCE、表单令牌交换，用户信息返回稳定 `sub` 和可选 `name`。这是 OAuth2 用户信息集成，不宣称实现完整 OIDC ID Token/JWKS 验证或发现协议。只有授权页面地址、不提供后端令牌交换和可信用户信息的站点，不能用于安全登录。

## 使用流程

1. 登录后自动创建私人空间。该空间内的记忆、角色、作品、项目和草稿仅本人可访问。
2. 打开“账户与空间”创建共享空间；所有者生成 24 小时有效、一次使用的邀请码。
3. 其他用户先登录，在“接受邀请”中粘贴邀请码。邀请不会自动发送给任何人。
4. 切换到同一个共享空间后，可向同一个角色上传不同作品或知识，通过对话共同调整规则，交由 Agent 提炼候选。
5. 所有者检查证据后发布。已有项目继续固定在原角色版本，需显式采用新版本。

| 能力 | 所有者 | 编辑者 | 查看者 |
| --- | --- | --- | --- |
| 查看、下载、检索空间资料 | 是 | 是 | 是 |
| 上传作品、对话、创建提炼任务、编辑草稿 | 是 | 是 | 否 |
| 发布/回滚/删除编剧角色 | 是 | 否 | 否 |
| 邀请、移除成员及调整权限 | 是 | 否 | 否 |

共享空间内全部资料对该空间成员开放。不同团队、不同合作角色需要独立边界时，创建不同共享空间。私人空间不能邀请其他用户。空间所有者不能被移除或降级。

## Agent 连接

在“账户与空间”中创建凭据。完整凭据只显示一次，绑定创建时所在的空间。以 `Authorization: Bearer <个人凭据>` 连接 `/mcp`（支持仪表盘同端口或配置的独立 MCP 端口）。配置文件使用环境变量引用个人凭据，不提交真实 token。

Agent 的编辑凭据可上传文字、领取提炼任务、提交候选、写草稿与审稿；发布仍由所有者在界面操作。只读凭据可读取与检索。凭据有效期不超过上游身份和当前会话（最长 8 小时），到期后登录并创建新凭据。退出只结束当前网页会话；独立 Agent 凭据可显式撤销。成员移除会撤销其该空间的 Agent 凭据。

认证模式不开放服务器本地文件索引、连接器密钥、全局验证器遥测、模型切换及路径恢复等管理员功能。正常记忆、检索、图谱、编剧工作流在空间内运行。原本配置到全局 MCP 的 Agent 必须更换为个人凭据。

## 原有数据与备份

旧 `vestige.db` 保持原位，不自动归给首个登录用户。需要认领时，在私有配置中设置 `legacy_owner_subject` 为 KX `/auth/user/user_info` 返回的用户 ID 字符串，然后重启并重新登录；指定用户获得“原有本地资料”空间。账号页面展示 KX 用户 ID。不要凭显示名称匹配身份。

身份信息在 `identity/identity.db`；会话令牌只存摘要，上游令牌用 `identity/session.key` 加密。每个新空间的数据在 `identity/spaces/<随机空间 ID>/vestige.db`，独立检索索引与事件通道。模型可复用已经验证的本机运行时，记忆、查询缓存和候选数据不共享。非空空间不自动更换向量模型。

完整恢复必须保存旧库、身份数据库、`session.key`、私有 `auth.json` 和所有空间数据库。每个 SQLite 使用在线 backup API 做一致性快照；跨数据库快照不等同于单一事务，要达到同一时刻快照请暂停写入/停止服务。恢复后应主动撤销会话与 Agent 凭据，避免备份中的旧会话再次可用。

## 验证

- 身份、权限、票据重放、双用户隔离及 MCP 会话测试：`cargo test -p vestige-mcp --lib identity::tests`。
- 前端：`pnpm --filter @vestige/dashboard check`、`pnpm --filter @vestige/dashboard test`。
- 本地模拟身份服务仅用于隔离验收：`python3 tests/identity/mock_kx.py --allow-test-login`。测试账号为 alice/bob/carol，口令为 test-only-password；生产配置不可指向模拟服务。

真实钉钉扫码和真实账号密码验证由账号持有人在界面完成；模拟账户测试不代表已经以真实用户登录。

身份与空间的在线备份：`python3 scripts/backup-identity.py <服务数据目录> <不存在的备份目录>`。此脚本不覆盖原有主库备份；需同时保留 `vestige.db` 的常规一致性备份。身份快照会移除会话、Agent 凭据和待使用邀请，恢复后所有用户重新登录。若 `VESTIGE_AUTH_CONFIG` 指向数据目录外的配置，管理员须单独备份该配置。

登录模式每隔最多 60 秒向上游复核身份；上游不可用时拒绝继续访问，不回退为匿名模式。
