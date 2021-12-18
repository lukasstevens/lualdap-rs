extern crate ldap3;
extern crate mlua;

use mlua::prelude::*;
use std::time::Duration;

fn hello(_: &Lua, name: String) -> LuaResult<()> {
    println!("hello, {}!", name);
    Ok(())
}

fn connect(_: &Lua, uri: String, timeout: u64) -> LuaResult<()> {
    let max_duration = Duration::new(u64::MAX, 1_000_000_000 - 1);
    let duration = if timeout == 0 { max_duration } else { Duration::from_secs(timeout) };
    ldap3::LdapConn::with_settings(ldap3::LdapConnSettings::new().set_conn_timeout(duration), &uri);
    Ok(())
}

#[mlua::lua_module]
fn lualdap_rs(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("hello", lua.create_function(hello)?)?;
    Ok(exports)
}
