use ldap3::Ldap;
use ldap3::{ResultEntry, SearchEntry};
use secrecy::ExposeSecret;
use secrecy::Secret;

use anyhow::anyhow;
use anyhow::Result;

#[derive(Clone)]
pub struct UsersInfo {
    pub ldap: Ldap,
    pub users_query: String,
}

#[derive(Debug)]
pub struct AdUserInfo {
    pub dn: String,
    pub email: String,
    pub groups: Vec<String>,
}

impl TryFrom<SearchEntry> for AdUserInfo {
    type Error = anyhow::Error;

    fn try_from(value: SearchEntry) -> std::result::Result<Self, Self::Error> {
        let mail = value
            .attrs
            .get("mail")
            .map(|mails| mails.first().cloned())
            .unwrap_or(None);

        let groups = value.attrs.get("memberOf").map(|groups| {
            groups
                .iter()
                .filter_map(|group| {
                    group.split(",").find_map(|item| {
                        item.starts_with("CN=")
                            .then(|| item.split_once("=").unwrap().1.to_string())
                    })
                })
                .collect()
        });

        if let (Some(mail), Some(groups)) = (mail, groups) {
            return Ok(AdUserInfo {
                email: mail,
                dn: value.dn,
                groups,
            });
        };

        Err(anyhow!("Unable to parse SearchEntry"))
    }
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

    pub async fn do_ldap_request(&self, query: &str, filter: &str) -> Result<Vec<ResultEntry>> {
        let res = self
            .ldap
            .clone()
            .search(
                query,
                ldap3::Scope::Subtree,
                filter,
                vec!["memberOf", "mail", "sAMAccountName"],
            )
            .await?;

        match res.success() {
            Ok((rs, _res)) => Ok(rs),
            Err(err) => Err(anyhow!("Failed ldap request: {err}")),
        }
    }

    pub async fn get_user_info(&self, user_dn: &str) -> Result<Option<AdUserInfo>> {
        let rs = self.do_ldap_request(user_dn, "(mail=*)").await?;
        if let Some(entry) = rs.first() {
            return Ok(Some(AdUserInfo::try_from(SearchEntry::construct(
                entry.clone(),
            ))?));
        }
        Ok(None)
    }

    pub async fn find_user_info(&self, login_or_mail: String) -> Result<Option<AdUserInfo>> {
        let rs = self
            .do_ldap_request(
                &self.users_query,
                &format!(
                    "(|(mail={})(sAMAccountName={}))",
                    login_or_mail, login_or_mail
                ),
            )
            .await?;

        if let Some(entry) = rs.first() {
            return Ok(Some(AdUserInfo::try_from(SearchEntry::construct(
                entry.clone(),
            ))?));
        }
        Ok(None)
    }
}
