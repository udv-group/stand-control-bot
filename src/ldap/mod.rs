use ldap3::Ldap;
use ldap3::SearchEntry;
use secrecy::ExposeSecret;
use secrecy::Secret;
use std::collections::HashSet;

use anyhow::Result;

#[derive(Clone)]
pub struct UsersInfo {
    ldap: Ldap,
    users_query: String,
}

#[derive(Debug)]
pub struct AdUserInfo {
    pub dn: String,
    pub email: String,
    #[allow(dead_code)]
    pub groups: HashSet<String>,
}

impl UsersInfo {
    pub async fn new(
        mut ldap: Ldap,
        login: &str,
        password: &Secret<String>,
        users_query: String,
    ) -> Result<Self> {
        ldap.simple_bind(login, password.expose_secret())
            .await?
            .success()?;

        Ok(Self { ldap, users_query })
    }

    pub async fn find_user_info(&self, login_or_mail: String) -> Result<Option<AdUserInfo>> {
        let (rs, _res) = self
            .ldap
            .clone()
            .search(
                &self.users_query,
                ldap3::Scope::Subtree,
                &format!(
                    "(|(mail={})(sAMAccountName={}))",
                    login_or_mail, login_or_mail
                ),
                vec!["memberOf", "mail", "sAMAccountName"],
            )
            .await?
            .success()?;

        if let Some(entry) = rs
            .first()
            .map(|entry| SearchEntry::construct(entry.clone()))
        {
            let mail = entry
                .attrs
                .get("mail")
                .map(|mails| mails.first().cloned())
                .unwrap_or(None);

            let groups = entry.attrs.get("memberOf").map(|groups| {
                HashSet::<String>::from_iter(groups.iter().filter_map(|group| {
                    group.split(",").find_map(|item| {
                        item.starts_with("CN=")
                            .then(|| item.split_once("=").unwrap().1.to_string())
                    })
                }))
            });

            if let (Some(mail), Some(groups)) = (mail, groups) {
                return Ok(Some(AdUserInfo {
                    email: mail,
                    dn: entry.dn,
                    groups,
                }));
            }
        }

        Ok(None)
    }
}
