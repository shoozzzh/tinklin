local path = jit.util.funcinfo(setfenv(1, getfenv())).source:match "^.*/"
local function require(module_name)
    return dofile_once(path .. module_name .. ".lua")
end

local npair = require "npair"

local xform = {}

---@class xform
---@field pos vector
---@field scale vector
---@field rot number

local entity_xform_fields = {}

function entity_xform_fields.pos:get()
    local x, y, _, _, _ = EntityGetTransform(self.entity_id)
    return npair.new { x, y }
end

function entity_xform_fields.pos:set(value)
    local _, _, rot, scale_x, scale_y = EntityGetTransform(self.entity_id)
    EntitySetTransform(self.entity_id, value[1], value[2], rot, scale_x, scale_y)
end

function entity_xform_fields.rot:get()
    local _, _, rot, _, _ = EntityGetTransform(self.entity_id)
    return rot
end

function entity_xform_fields.rot:set(value)
    local x, y, _, scale_x, scale_y = EntityGetTransform(self.entity_id)
    EntitySetTransform(self.entity_id, x, y, value, scale_x, scale_y)
end

function entity_xform_fields.scale:get()
    local _, _, _, scale_x, scale_y = EntityGetTransform(self.entity_id)
    return npair.new { scale_x, scale_y }
end

function entity_xform_fields.scale:set(value)
    local x, y, rot, _, _ = EntityGetTransform(self.entity_id)
    EntitySetTransform(self.entity_id, nil, nil, nil, value.x, value.y)
end

---@class entity_xform: xform
---@field entity_id entity_id
local entity_xform_mt = {
    __index = function(t, k)
        local field = entity_xform_fields[k]
        if field then
            local getter = field.get
            if getter then
                return getter(t)
            end
        end
    end,
    __newindex = function(t, k, v)
        local field = entity_xform_fields[k]
        if field then
            local setter = field.set
            if setter then
                return setter(t, v)
            end
        end
    end,
}

local comp_xform_fields = {}

function comp_xform_fields.pos:get()
    local x, y, _, _, _ = ComponentGetValue2(self.comp_id, "Transform")
    return npair.new { x, y }
end

function comp_xform_fields.pos:set(value)
    local _, _, scale_x, scale_y, rot = ComponentGetValue2(self.comp_id, "Transform")
    ComponentSetValue2(self.comp_id, "Transform", value[1], value[2], scale_x, scale_y, rot)
end

function comp_xform_fields.rot:get()
    local _, _, _, _, rot = ComponentGetValue2(self.comp_id, "Transform")
    return rot
end

function comp_xform_fields.rot:set(value)
    local x, y, scale_x, scale_y, _ = ComponentGetValue2(self.comp_id, "Transform")
    ComponentSetValue2(self.comp_id, "Transform", x, y, scale_x, scale_y, value)
end

function comp_xform_fields.scale:get()
    local _, _, scale_x, scale_y, _ = ComponentGetValue2(self.comp_id, "Transform")
    return npair.new { scale_x, scale_y }
end

function comp_xform_fields.scale:set(value)
    local x, y, _, _, rot = ComponentGetValue2(self.comp_id, "Transform")
    ComponentSetValue2(self.comp_id, "Transform", nil, nil, value.x, value.y, nil)
end

---@class comp_xform: xform
---@field entity_id entity_id
local comp_xform_mt = {
    __index = function(t, k)
        local field = comp_xform_fields[k]
        if field then
            local getter = field.get
            if getter then
                return getter(t)
            end
        end
    end,
    __newindex = function(t, k, v)
        local field = comp_xform_fields[k]
        if field then
            local setter = field.set
            if setter then
                return setter(t, v)
            end
        end
    end,
}

---@param entity_id entity_id
---@return entity_xform
---@deprecated
function xform.from_entity(entity_id)
    return xform.of_entity(entity_id)
end

---@param entity_id entity_id
---@return entity_xform
function xform.of_entity(entity_id)
    return setmetatable({ entity_id = entity_id }, entity_xform_mt)
end

---@param comp_id comp_id
---@return comp_xform
---@deprecated
function xform.from_comp(comp_id)
    return xform.of_comp(comp_id)
end

---View the "Transform" property of a comp(if it has).
---@param comp_id comp_id
---@return comp_xform
function xform.of_comp(comp_id)
    return setmetatable({ comp_id = comp_id }, comp_xform_mt)
end

return xform
