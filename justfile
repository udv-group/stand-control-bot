fmt:
    cargo clippy --fix --allow-dirty
    cargo fmt

check:
    cargo check
    cargo fmt --check
    cargo clippy -- -D warnings

build-css:
    npm run build:css

ldap-setup:
    mkdir -p .ldap/db .ldap/config .ldap/run
    slapadd -n 0 -F .ldap/config -l ldap/config.ldif
    slapd -h ldapi://ldapi -F .ldap/config
    sleep 1
    ldapadd -x -D 'cn=Manager,dc=example,dc=org' -w test -H ldapi://ldapi -f ldap/init.ldif
    ldapadd -x -D 'cn=Manager,dc=example,dc=org' -w test -H ldapi://ldapi -f ldap/overlay.ldif
    ldapadd -x -D 'cn=Manager,dc=example,dc=org' -w test -H ldapi://ldapi -f ldap/refint.ldif
    ldapadd -x -D 'cn=Manager,dc=example,dc=org' -w test -H ldapi://ldapi -f ldap/refint2.ldif
    ldapadd -x -D 'cn=Manager,dc=example,dc=org' -w test -H ldapi://ldapi -f ldap/user.ldif
    kill $(cat .ldap/run/slapd.pid)

ldap-clean:
    kill $(cat .ldap/run/slapd.pid) || echo "slapd not running"
    rm -rf .ldap

ldap-start:
    slapd -h "ldapi://ldapi ldap://localhost:2389 ldaps://localhost:2636" -F .ldap/config
