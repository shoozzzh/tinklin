-- run this file in game

local file_content = {}

local function writeln(s)
    file_content[#file_content + 1] = s
end

---@diagnostic disable-next-line: undefined-global
for _, data in ipairs(objects) do
    local comp_type, object_name, object_type = data[1], data[2], data[3]
    local entity_id = EntityCreateNew("")
    local comp_id = EntityAddComponent2(entity_id, comp_type, {})
    local object_fields = ComponentObjectGetMembers(comp_id, object_name)
    writeln(("[%s__%s]"):format(comp_type, object_name))
    writeln(("_type = \"%s\""):format(object_type))
    for key, value in pairs(object_fields) do
        if value == "" then
            local values = { ComponentObjectGetValue2(comp_id, object_name, key) }
            if values[1] == "" then
                value = "\\\"\\\""
            elseif #values > 0 then
                value = "{" .. table.concat(values, ",") .. "}"
            else
                value = "[Unsupported]"
            end
        end
        writeln(("%s = \"%s\""):format(key, value))
    end
    writeln("")
end

print(table.concat(file_content, "\n") .. "\n")
