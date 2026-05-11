declare function tool<T extends Record<string, any>>(options: ToolOptions<T>): any[];

declare namespace tool {
    export function tool<T extends Record<string, any>>(options: ToolOptions<T>): any[];
    export function tool<T extends any[]>(options: T): T;
}

interface ToolOptions<T = Record<string, any>> {
    name: string;
    description: string;
    callback: (args: T) => any;
    parameters?: ParametersDefinition;
    timeoutSecs?: number;
    maxRetries?: number;
    isIdempotent?: boolean;
}

type ParametersDefinition = {
    [key: string]:
        | string
        | string[]
        | ParameterProperty;
};

interface ParameterProperty {
    type?: string;
    enum?: string[];
    description?: string;
    required?: boolean;

    [key: string]: any;
}

export = tool;