use super::provider::Identity;
use super::{Error, Grant, Hub, Result, hash, now, random};
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce, aead::Aead};
use rusqlite::{OptionalExtension, params};
use serde_json::{Value, json};

impl Hub {
    pub fn local_mode(&self) -> bool {
        self.local_mcp
    }

    /// Bind the current browser login to the loopback MCP bridge. The bridge
    /// has no bearer token by design; it is only reachable on localhost and
    /// only while this expiring, server-validated browser session exists.
    pub fn bind_local(&self, grant: &Grant) -> Result<()> {
        if !self.local_mode() {
            return Ok(());
        }
        self.db(|c| {
            c.execute(
                "INSERT INTO local_bindings(singleton,credential,updated) VALUES(1,?1,?2)
                 ON CONFLICT(singleton) DO UPDATE SET credential=excluded.credential, updated=excluded.updated",
                params![grant.credential, now()],
            )?;
            Ok(())
        })
    }

    pub fn unbind_local(&self, credential: &str) -> Result<()> {
        self.db(|c| {
            c.execute(
                "DELETE FROM local_bindings WHERE singleton=1 AND credential=?1",
                [credential],
            )?;
            Ok(())
        })
    }

    /// Resolve the currently logged-in browser identity for a local Agent.
    /// No token is accepted from the request. Membership, expiry, and the
    /// selected workspace are checked again for every MCP request.
    pub async fn local_grant(&self) -> Result<Grant> {
        let grant = self.db(|c| {
            c.query_row(
                "SELECT t.id,t.user_id,t.workspace,m.role,t.permission,t.kind
                 FROM local_bindings b JOIN credentials t ON t.id=b.credential
                 JOIN members m ON m.workspace=t.workspace AND m.user_id=t.user_id
                 WHERE b.singleton=1 AND t.kind='session' AND t.expires>?1",
                [now()],
                |r| {
                    let role: String = r.get(3)?;
                    let permission: String = r.get(4)?;
                    Ok(Grant {
                        credential: r.get(0)?,
                        user: r.get(1)?,
                        workspace: r.get(2)?,
                        role: lesser(&role, &permission).into(),
                        kind: r.get(5)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| Error::unauthorized("请先在本地客户端登录"))
        })?;
        self.check(&grant)?;
        Ok(grant)
    }

    pub fn db<T>(&self, f: impl FnOnce(&mut rusqlite::Connection) -> Result<T>) -> Result<T> {
        let mut connection = self.db.lock().map_err(|_| Error::internal())?;
        f(&mut connection)
    }
    pub fn encrypt(&self, s: &str) -> Result<Vec<u8>> {
        let nonce_id = uuid::Uuid::new_v4();
        let nonce = &nonce_id.as_bytes()[..12];
        let cipher = ChaCha20Poly1305::new_from_slice(&self.key).map_err(|_| Error::internal())?;
        let mut out = nonce.to_vec();
        out.extend(
            cipher
                .encrypt(Nonce::from_slice(nonce), s.as_bytes())
                .map_err(|_| Error::internal())?,
        );
        Ok(out)
    }
    pub fn decrypt(&self, b: &[u8]) -> Result<String> {
        if b.len() < 28 {
            return Err(Error::internal());
        }
        let cipher = ChaCha20Poly1305::new_from_slice(&self.key).map_err(|_| Error::internal())?;
        let raw = cipher
            .decrypt(Nonce::from_slice(&b[..12]), &b[12..])
            .map_err(|_| Error::internal())?;
        String::from_utf8(raw).map_err(|_| Error::internal())
    }
    pub fn new_session(
        &self,
        identity: Identity,
        provider: &str,
        upstream: &str,
        expiry: i64,
    ) -> Result<String> {
        let token = random();
        let encrypted = self.encrypt(upstream)?;
        self.db(|c| {
            let tx = c.transaction()?;
            let uid = tx.query_row("SELECT id FROM users WHERE issuer=?1 AND subject=?2", params![identity.issuer,identity.subject], |r|r.get::<_,String>(0)).optional()?.unwrap_or_else(random);
            tx.execute("INSERT INTO users(id,issuer,subject,name) VALUES(?1,?2,?3,?4) ON CONFLICT(issuer,subject) DO UPDATE SET name=excluded.name",params![uid,identity.issuer,identity.subject,identity.name])?;
            let workspace = tx.query_row("SELECT id FROM workspaces WHERE owner=?1 AND kind='personal'", [&uid], |r|r.get::<_,String>(0)).optional()?.unwrap_or_else(random);
            tx.execute("INSERT OR IGNORE INTO workspaces(id,name,kind,owner) VALUES(?1,'我的私人空间','personal',?2)",params![workspace,uid])?;
            tx.execute("INSERT OR IGNORE INTO members(workspace,user_id,role) VALUES(?1,?2,'owner')",params![workspace,uid])?;
            if provider == "kx" && self.config.legacy_owner_subject.as_deref() == Some(&identity.subject) {
                tx.execute("INSERT INTO workspaces(id,name,kind,owner) VALUES('legacy','原有本地资料','legacy',?1) ON CONFLICT(id) DO UPDATE SET owner=excluded.owner",[&uid])?;
                tx.execute("INSERT OR IGNORE INTO members(workspace,user_id,role) VALUES('legacy',?1,'owner')",[&uid])?;
            }
            tx.execute("DELETE FROM credentials WHERE expires<=?1",[now()])?;
            let expiry=expiry.min(now()+8*3600);
            if expiry <= now() { return Err(Error::unauthorized("上游凭据已过期")); }
            tx.execute("INSERT INTO credentials VALUES(?1,?2,?3,?4,'session','owner','网页登录',?5,?6,?7,?8)",params![random(),hash(&token),uid,workspace,provider,encrypted,expiry,now()])?;
            tx.commit()?;
            Ok(token)
        })
    }
    pub async fn authenticate(&self, token: &str, kind: &str) -> Result<Grant> {
        let digest = hash(token);
        let (grant,provider,encrypted,checked,issuer,subject) = self.db(|c| {
            c.query_row("SELECT t.id,t.user_id,t.workspace,m.role,t.permission,t.provider,t.upstream,t.checked,u.issuer,u.subject,t.kind FROM credentials t JOIN members m ON m.workspace=t.workspace AND m.user_id=t.user_id JOIN users u ON u.id=t.user_id WHERE t.hash=?1 AND t.kind=?2 AND t.expires>?3",params![digest,kind,now()],|r| {
                let role:String=r.get(3)?; let permission:String=r.get(4)?;
                Ok((Grant {credential:r.get(0)?,user:r.get(1)?,workspace:r.get(2)?,role: lesser(&role,&permission).into(),kind:r.get(10)?},r.get::<_,String>(5)?,r.get::<_,Vec<u8>>(6)?,r.get::<_,i64>(7)?,r.get::<_,String>(8)?,r.get::<_,String>(9)?))
            }).optional()?.ok_or_else(||Error::unauthorized("请重新登录或更新 Agent 凭据"))
        })?;
        if now() - checked >= 60 {
            let identity = self
                .provider
                .verify(&provider, &self.decrypt(&encrypted)?)
                .await?;
            if identity.issuer != issuer || identity.subject != subject {
                return Err(Error::unauthorized("身份已改变"));
            }
            self.db(|c| {
                c.execute(
                    "UPDATE credentials SET checked=?1 WHERE id=?2",
                    params![now(), grant.credential],
                )?;
                Ok(())
            })?;
        }
        // Recheck after the network await so a concurrent revoke cannot grant a new request.
        self.check(&grant)?;
        Ok(grant)
    }
    pub fn check(&self, g: &Grant) -> Result<()> {
        self.db(|c| {
            let row=c.query_row("SELECT m.role,t.permission,t.workspace FROM credentials t JOIN members m ON m.workspace=t.workspace AND m.user_id=t.user_id WHERE t.id=?1 AND t.expires>?2",params![g.credential,now()],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).optional()?;
            match row { Some((role,perm,workspace)) if workspace==g.workspace && lesser(&role,&perm)==g.role => Ok(()), _=>Err(Error::unauthorized("会话或成员权限已改变")) }
        })
    }
    pub fn me(&self, g: &Grant) -> Result<Value> {
        self.db(|c| {
            let user=c.query_row("SELECT id,name,subject FROM users WHERE id=?1",[&g.user],|r|Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"subject":r.get::<_,String>(2)?})))?;
            let spaces=c.prepare("SELECT w.id,w.name,w.kind,m.role FROM workspaces w JOIN members m ON w.id=m.workspace WHERE m.user_id=?1 ORDER BY w.kind,w.name")?.query_map([&g.user],|r|Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"kind":r.get::<_,String>(2)?,"role":r.get::<_,String>(3)?})))?.collect::<std::result::Result<Vec<_>,_>>()?;
            Ok(json!({"enabled":true,"user":user,"workspace":g.workspace,"role":g.role,"workspaces":spaces}))
        })
    }
    pub fn account_action(&self, g: &Grant, a: &Value) -> Result<Value> {
        if g.kind != "session" {
            return Err(Error::forbidden());
        }
        let action = a["action"].as_str().unwrap_or("");
        let result = self.db(|c| {
            let tx=c.transaction()?;
            let output=match action {
                "switch" => {
                    let id=field(a,"workspace",100)?;
                    let exists:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM members WHERE workspace=?1 AND user_id=?2)",params![id,g.user],|r|r.get(0))?;
                    if !exists {return Err(Error::forbidden());}
                    tx.execute("UPDATE credentials SET workspace=?1 WHERE id=?2",params![id,g.credential])?; json!({"ok":true})
                }
                "create_workspace" => {
                    let name=field(a,"name",120)?;
                    let count:i64=tx.query_row("SELECT count(*) FROM workspaces WHERE owner=?1",[&g.user],|r|r.get(0))?;
                    if count>=30{return Err(Error::bad("每个用户最多创建 30 个空间"));}
                    let id=random();
                    tx.execute("INSERT INTO workspaces VALUES(?1,?2,'shared',?3)",params![id,name,g.user])?;
                    tx.execute("INSERT INTO members VALUES(?1,?2,'owner')",params![id,g.user])?;
                    json!({"workspace":id})
                }
                "invite" => {
                    owner(g)?;
                    let kind:String=tx.query_row("SELECT kind FROM workspaces WHERE id=?1",[&g.workspace],|r|r.get(0))?;
                    if kind!="shared" {return Err(Error::bad("仅共享空间可以邀请成员"));}
                    let role=member_role(a)?;let token=random();
                    tx.execute("DELETE FROM invitations WHERE expires<=?1",[now()])?;
                    tx.execute("INSERT INTO invitations VALUES(?1,?2,?3,?4,?5)",params![hash(&token),g.workspace,role,now()+86400,g.user])?;
                    json!({"invitation":token,"expires_at":now()+86400})
                }
                "join" => {
                    let token=field(a,"invitation",200)?;
                    let (workspace,role)=tx.query_row("DELETE FROM invitations WHERE hash=?1 AND expires>?2 RETURNING workspace,role",params![hash(token),now()],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).optional()?.ok_or_else(||Error::bad("邀请已使用或过期"))?;
                    tx.execute("INSERT OR IGNORE INTO members VALUES(?1,?2,?3)",params![workspace,g.user,role])?;
                    json!({"workspace":workspace})
                }
                "members" => {
                    let rows=tx.prepare("SELECT u.id,u.name,m.role FROM members m JOIN users u ON u.id=m.user_id WHERE m.workspace=?1 ORDER BY m.role,u.name")?.query_map([&g.workspace],|r|Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"role":r.get::<_,String>(2)?})))?.collect::<std::result::Result<Vec<_>,_>>()?;
                    json!({"members":rows})
                }
                "set_member" | "remove_member" => {
                    owner(g)?; let uid=field(a,"user_id",100)?;
                    let old:String=tx.query_row("SELECT role FROM members WHERE workspace=?1 AND user_id=?2",params![g.workspace,uid],|r|r.get(0)).optional()?.ok_or_else(||Error::bad("成员不存在"))?;
                    if old=="owner" {return Err(Error::bad("不能移除或降级空间所有者"));}
                    if action=="remove_member" {
                        tx.execute("DELETE FROM members WHERE workspace=?1 AND user_id=?2",params![g.workspace,uid])?;
                        tx.execute("DELETE FROM credentials WHERE workspace=?1 AND user_id=?2 AND kind='agent'",params![g.workspace,uid])?;
                        tx.execute("UPDATE credentials SET workspace=(SELECT id FROM workspaces WHERE owner=?1 AND kind='personal') WHERE workspace=?2 AND user_id=?1 AND kind='session'",params![uid,g.workspace])?;
                    } else {tx.execute("UPDATE members SET role=?1 WHERE workspace=?2 AND user_id=?3",params![member_role(a)?,g.workspace,uid])?;}
                    json!({"ok":true})
                }
                "tokens" => {
                    let rows=tx.prepare("SELECT id,label,workspace,permission,expires FROM credentials WHERE user_id=?1 AND kind='agent' AND expires>?2")?.query_map(params![g.user,now()],|r|Ok(json!({"id":r.get::<_,String>(0)?,"label":r.get::<_,String>(1)?,"workspace":r.get::<_,String>(2)?,"permission":r.get::<_,String>(3)?,"expires_at":r.get::<_,i64>(4)?})))?.collect::<std::result::Result<Vec<_>,_>>()?;
                    json!({"tokens":rows})
                }
                "create_token" => {
                    let label=field(a,"label",120)?;
                    let permission=member_role(a)?;
                    if rank(permission)>rank(&g.role) {return Err(Error::forbidden());}
                    let count:i64=tx.query_row("SELECT count(*) FROM credentials WHERE user_id=?1 AND kind='agent' AND expires>?2",params![g.user,now()],|r|r.get(0))?;
                    if count>=30{return Err(Error::bad("有效 Agent 凭据最多 30 个"));}
                    let token=random();let id=random();
                    // Agent credentials cannot outlive their verified upstream session.
                    tx.execute("INSERT INTO credentials SELECT ?1,?2,user_id,workspace,'agent',?3,?4,provider,upstream,expires,checked FROM credentials WHERE id=?5",params![id,hash(&token),permission,label,g.credential])?;
                    json!({"id":id,"token":token,"workspace":g.workspace})
                }
                "revoke_token" => {let id=field(a,"id",100)?;tx.execute("DELETE FROM credentials WHERE id=?1 AND user_id=?2 AND kind='agent'",params![id,g.user])?;json!({"ok":true})}
                "logout" => {tx.execute("DELETE FROM credentials WHERE id=?1",[&g.credential])?;json!({"ok":true})}
                _=>return Err(Error::bad("未知账户操作"))
            };
            tx.execute("INSERT INTO audit VALUES(?1,?2,?3,?4,?5)",params![random(),g.user,g.workspace,action,now()])?;
            tx.commit()?;Ok(output)
        })?;
        Ok(result)
    }
    pub fn audit(&self, g: &Grant, action: &str) -> Result<()> {
        self.db(|c| {
            c.execute(
                "INSERT INTO audit VALUES(?1,?2,?3,?4,?5)",
                params![random(), g.user, g.workspace, action, now()],
            )?;
            Ok(())
        })
    }
}
pub fn field<'a>(v: &'a Value, k: &str, max: usize) -> Result<&'a str> {
    v[k].as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= max)
        .ok_or_else(|| Error::bad(format!("{k} 缺失或过长")))
}
pub fn rank(s: &str) -> u8 {
    match s {
        "owner" => 3,
        "editor" => 2,
        "viewer" => 1,
        _ => 0,
    }
}
fn lesser<'a>(a: &'a str, b: &'a str) -> &'a str {
    if rank(a) < rank(b) { a } else { b }
}
fn member_role(v: &Value) -> Result<&str> {
    let role = field(v, "role", 20)?;
    if !matches!(role, "editor" | "viewer") {
        return Err(Error::bad("权限必须是 editor 或 viewer"));
    }
    Ok(role)
}
fn owner(g: &Grant) -> Result<()> {
    if g.role == "owner" {
        Ok(())
    } else {
        Err(Error::forbidden())
    }
}
