# Testing

### LDAP/AD server

It is preferable to use real AD server to run auth tests, since auth is designed to
work with AD specifically.

It is possible to use LDAP-server instead, though you will have to use fully qualified names as logins.
To set up a local LDAP-server consult [ldap3 repo](https://github.com/inejge/ldap3/tree/00a513ece4ffa9a9782860c285f4c4c12bc07552/data).
