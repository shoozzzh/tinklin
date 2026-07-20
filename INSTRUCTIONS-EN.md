[简体中文](INSTRUCTIONS.md) | English
## Install
[Download the newest release](https://github.com/shoozzzh/tinklin/releases/latest) and unzip it to `mods/YOUR_MOD/libs/tinklin` or anywhere in your mod folder.
## Usage
### Introduce
```lua
local tl = dofile_once("path/to/tinklin/main.lua")
```
### Wrap
```lua
local entity = tl.entity_wrap(GetUpdatedEntityID())
local comp = tl.comp_wrap(GetUpdatedComponentID()) --[[@as LuaComponent]]
```
In the above case you can also use convenient functions：
```lua
local entity = tl.entity_cur()
local comp = tl.comp_cur() --[[@as LuaComponent]]
```
### Get/Set component fields
```lua
local proj_comp = entity:comp_first "ProjectileComponent"
if not proj_comp then return end

-- get/set component fields
proj_comp.lifetime = proj_comp.lifetime + 60
proj_comp.damage = proj_comp.damage - 0.4

-- set multiple fields
proj_comp.set{
    lifetime = 100,
    damage = 4,
}

local vel_comp = entity:comp_first "VelocityComponent"
if vel_comp then
    -- get/set vector
    if vel_comp.mVelocity:norm() > 400 then
        vel_comp.mVelocity = tl.vec(0, 0)
    end
    -- unpack a vector(not recommended)
    local vel_x, vel_y = unpack(vel_comp.mVelocity)
end

-- get an component object
local cfg_exp = proj_comp:object "config_explosion"

-- get/set component object fields
cfg_exp.explosion_radius = cfg_exp.explosion_radius + 20
```
### Get component(s) from an entity
```lua
-- get the first component with a certain type
local first_lua_comp = entity:comp_first "LuaComponent"

-- get the first enabled component with a certain type
local first_enabled_lua_comp = entity:comp_first_enabled "LuaComponent"

-- optional tag param
local first_lua_comp_with_tag = entity:comp_first("LuaComponent", "enabled_in_world")

-- an iterator of all components that meet the requirement(s)
entity:comps "LuaComponent":each( function(comp) comp.meta:remove() end )

-- if the given component type is nil, return components of any type
entity:comps_enabled(nil, "enabled_in_hand"):each( function(comp) comp.meta.enabled = false end )
```
### Component operations
```lua
-- create a new component
local comp = entity:comp_new "LifetimeComponent" { lifetime = 1 }

-- enable/disable component
comp.meta.enabled = false

-- remove a component
comp.meta:remove()
```
### Tags
```lua
-- toggle a tag from component
comp.meta.tag.enabled_in_world = true

-- check a tag from component
if comp.meta.tag.enabled_in_hand then
    -- ...
end

-- same for entity tags
entity.tag.invincible = true

if entity.tag.invincible then
    -- ...
end
```
