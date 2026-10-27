-- this file is meant to test if the definitions works
local tl = require "tinklin.main"

local e = tl.entity_wrap(1)
local ab_comp = e:comp_new "AbilityComponent" {
    mana = 1,
}
ab_comp:set {
    add_these_child_actions = "BULLET",
}

local expl_comp = e:comp_new "ExplosionComponent" {}
expl_comp:object "config_explosion".audio_event_name = ""

local it_comp = e:comp_first "InheritTransformComponent"
if it_comp then
    it_comp.Transform.pos = tl.npair.vec(1, 2)
end

local comp = tl.comp_cur() --[[@as LuaComponent]]

---@type iterator<string>
local iter = {}
local x1, x2, x3 = iter
    :map_multi(function(n)
        return 1, 2, 3
    end)
    :map_multi(function(n)
        return 3, 2, 1
    end)
    :map(function(a, b, c)
        return 1
    end)
    :nth(1)

---@type multi_iterator
local miter = {}

local tru = miter
    :take_while(function(a, b)
        ---@cast a string
        local t = a:sub(1, 1)
        return true
    end)
    :map(function(a, b)
        return 1
    end)
    :all(function(n)
        return n == 1
    end)
