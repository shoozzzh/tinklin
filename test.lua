-- this file is meant to test auto completion stuff
local tl = require "tinklin.main"

local e = tl.entity_wrap(1)
local ab_comp = e:comp_new "AbilityComponent" {
    mana = 1,
}
ab_comp:set {
    add_these_child_actions = "BULLET",
}

local comp = tl.comp_cur() --[[@as LuaComponent]]
