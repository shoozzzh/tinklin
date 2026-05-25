---@meta

---@class entity
local entity_methods = {}

--@overload fun(self, comp_type: "{}", tag: string?): {}?
---@param comp_type component_type
---@param tag string
---@return component?
function entity_methods:comp_first(comp_type, tag) end

--@overload fun(self, comp_type: "{}", tag: string?): {}?
---@param comp_type component_type
---@param tag string
---@return component?
function entity_methods:comp_first_enabled(comp_type, tag) end

--@overload fun(self, comp_type: "{}", tag: string?): iterator<{}>
---@param comp_type nil
---@param tag string?
---@return iterator<component>
function entity_methods:comps(comp_type, tag) end

--@overload fun(self, comp_type: "{}", tag: string?): iterator<{}>
---@param comp_type nil
---@param tag string?
---@return iterator<component>
function entity_methods:comps_enabled(comp_type, tag) end

--@overload fun(self, comp_type: "{}"): fun(fields: {}.set): {}
---@param comp_type component_type
---@return component
function entity_methods:comp_new(comp_type) end
