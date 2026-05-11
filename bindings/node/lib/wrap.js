function tool(options) {
    if (Array.isArray(options)) {
        return options
    }
    const {name, description, callback, timeoutSecs, maxRetries, isIdempotent} = options
    const raw = options.parameters || {}
    const required = []
    const properties = {}
    for (const [key, val] of Object.entries(raw)) {
        if (Array.isArray(val)) {
            properties[key] = {type: 'string', enum: val, description: key}
            required.push(key)
        } else if (typeof val === 'string') {
            properties[key] = {type: val, description: key}
            required.push(key)
        } else {
            properties[key] = val
            if (val.required !== false) required.push(key)
        }
    }
    const paramsJson = JSON.stringify({
        type: 'object',
        properties,
        required: required.length > 0 ? required : undefined,
    })
    const wrapped = (_err, argsJson) => {
        const args = JSON.parse(argsJson)
        const result = callback(args)
        return typeof result === 'string' ? result : JSON.stringify(result)
    }
    return [name, description, paramsJson, wrapped, timeoutSecs, maxRetries, isIdempotent].filter(
        (v) => v !== undefined,
    )
}


module.exports = tool