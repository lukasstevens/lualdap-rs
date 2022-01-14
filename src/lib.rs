extern crate ldap3;
extern crate mlua;

use mlua::prelude::*;
use std::convert::From;
use std::time::Duration;

#[derive(Clone)]
struct SearchEntry(ldap3::SearchEntry);

impl From<ldap3::SearchEntry> for SearchEntry {
    fn from(item: ldap3::SearchEntry) -> Self {
        SearchEntry(item)
    }
}

impl mlua::UserData for SearchEntry {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("dn", |_, this| Ok(this.0.dn.clone()));
        fields.add_field_method_get("attrs", |_, this| Ok(this.0.attrs.clone()));
        fields.add_field_method_get("bin_attrs", |_, this| Ok(this.0.bin_attrs.clone()));
    }
}

struct LdapConn(ldap3::LdapConn);

fn connect(_: &Lua, uri: String, timeout: u64) -> LuaResult<LdapConn> {
    let max_duration = Duration::new(u64::MAX, 1_000_000_000 - 1);
    let duration = if timeout == 0 { max_duration } else { Duration::from_secs(timeout) };
    let ldap_settings = ldap3::LdapConnSettings::new().set_conn_timeout(duration);
    let conn =
        ldap3::LdapConn::with_settings(ldap_settings, &uri).unwrap();
    Ok(LdapConn(conn))
}

fn to_scope(scope: &str) -> Option<ldap3::Scope> {
   match scope.to_lowercase().as_str() {
    "base" => Some(ldap3::Scope::Base),
    "onelevel" => Some(ldap3::Scope::OneLevel),
    "subtree" => Some(ldap3::Scope::Subtree),
    _ => None,
   } 
}
impl mlua::UserData for LdapConn {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("simple_bind", |_, this, (bind_dn, bind_pw) : (String, String)| {
            this.0.simple_bind(&bind_dn, &bind_pw);
            Ok(())
        });
        methods.add_method_mut("search", |lua, this, (base, scope, filter, attrs) : (String, String, String, Vec<String>)| {
            let scope = to_scope(&scope).unwrap();
            let (results, _) = this.0.search(&base, scope, &filter, attrs).unwrap().success().unwrap();
            let entries = results.iter()
                                                    .map(|re| ldap3::SearchEntry::construct(re.clone()))
                                                    .map(|re| SearchEntry::from(re))
                                                    .enumerate();
            let table = lua.create_table_from(entries).unwrap();
            Ok(table)
        });
        methods.add_method_mut("unbind", |_, this, ()| {
            this.0.unbind();
            Ok(())
        });
    }
}

fn hello(_: &Lua, name: String) -> LuaResult<()> {
    println!("hello, {}!", name);
    Ok(())
}

#[mlua::lua_module]
fn lualdap_rs(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("hello", lua.create_function(hello)?)?;
    Ok(exports)
}
