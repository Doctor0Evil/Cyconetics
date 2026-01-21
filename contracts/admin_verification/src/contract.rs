#[cfg(target_arch = "wasm32")]
use cosmwasm_std::{
    entry_point, DepsMut, Env, MessageInfo, Response, Deps, QueryRequest, to_binary, Binary, StdResult,
};
use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct QueryMsg {
    pub is_admin: Addr,
}

pub const ADMINS: Item<Vec<Addr>> = Item::new("admins");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let validated: Vec<Addr> = msg.admins.into_iter().map(|a| deps.api.addr_validate(&a).unwrap()).collect();
    ADMINS.save(deps.storage, &validated)?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg { is_admin } => {
            let admins = ADMINS.load(deps.storage)?;
            let ok = admins.contains(&is_admin);
            to_binary(&ok)
        }
    }
}
